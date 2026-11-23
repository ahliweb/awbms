#!/usr/bin/env bash
# VG-01 legacy-baseline delta report (issue #7, P0.3).
#
# Compares the frozen AWCMS / AWCMS-Astro SHAs (contracts/legacy/awcms/frozen/provenance.json)
# with intentionally observed upstream SHAs and classifies every changed path using
# tools/vg01_baseline_delta.rules. Read-only: it never touches the frozen corpus and
# never refreshes the baseline. A refresh needs an explicit decision, provenance update,
# regenerated hashes, parity review and affected-test execution.
#
# Exit codes:
#   0  nothing changed, or only CI/tooling/docs changed
#   1  tool error (bad arguments, API failure, unwritable output)
#   2  ALERT: contract / schema / security / consumer-surface change
#   3  REVIEW: only behaviour/parity or unclassified changes
set -euo pipefail
export LC_ALL=C

usage() {
  cat >&2 <<'USAGE'
Usage: tools/vg01_baseline_delta.sh --out DIR [options]
  --out DIR           output directory (baseline-delta.md and baseline-delta.json)
  --awcms-head REF    observed AWCMS ref or SHA (default: main)
  --astro-head REF    observed AWCMS-Astro ref or SHA (default: main)
  --provenance FILE   frozen provenance (default: contracts/legacy/awcms/frozen/provenance.json)
  --rules FILE        classification rules (default: tools/vg01_baseline_delta.rules)
  --owner NAME        GitHub owner of both repositories (default: ahliweb)
USAGE
  exit 1
}

die() {
  echo "vg01_baseline_delta: $*" >&2
  exit 1
}

script_dir=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
repo_root=$(cd "$script_dir/.." && pwd)

out=""
awcms_ref="main"
astro_ref="main"
provenance="$repo_root/contracts/legacy/awcms/frozen/provenance.json"
rules="$script_dir/vg01_baseline_delta.rules"
owner="ahliweb"

while (($# > 0)); do
  case "$1" in
    --out) out=${2:?}; shift 2 ;;
    --awcms-head) awcms_ref=${2:?}; shift 2 ;;
    --astro-head) astro_ref=${2:?}; shift 2 ;;
    --provenance) provenance=${2:?}; shift 2 ;;
    --rules) rules=${2:?}; shift 2 ;;
    --owner) owner=${2:?}; shift 2 ;;
    -h | --help) usage ;;
    *) usage ;;
  esac
done
[[ -n "$out" ]] || usage
[[ -f "$provenance" ]] || die "provenance not found: $provenance"
[[ -f "$rules" ]] || die "rules not found: $rules"
command -v gh >/dev/null || die "gh CLI is required"

# Never write inside the frozen corpus.
mkdir -p "$out"
out_abs=$(cd "$out" && pwd -P)
frozen_abs=""
if [[ -d "$repo_root/contracts/legacy" ]]; then frozen_abs=$(cd "$repo_root/contracts/legacy" && pwd -P); fi
if [[ -n "$frozen_abs" && ( "$out_abs" == "$frozen_abs" || "$out_abs" == "$frozen_abs"/* ) ]]; then
  rmdir "$out_abs" 2>/dev/null || true
  die "refusing to write inside contracts/legacy (frozen corpus is read-only)"
fi

json_field() { # json_field KEY FILE -> value of a flat string field
  sed -n "s/^[[:space:]]*\"$1\"[[:space:]]*:[[:space:]]*\"\\([^\"]*\\)\".*/\\1/p" "$2" | head -n 1
}
frozen_awcms=$(json_field awcms "$provenance")
frozen_astro=$(json_field awcmsAstro "$provenance")
is_sha() { [[ "$1" =~ ^[0-9a-f]{40}$ ]]; }
is_sha "$frozen_awcms" || die "bad frozen awcms SHA in provenance"
is_sha "$frozen_astro" || die "bad frozen awcms-astro SHA in provenance"

resolve_sha() { # resolve_sha REPO REF
  local sha
  sha=$(gh api "repos/$owner/$1/commits/$2" --jq .sha) || die "cannot resolve $1@$2"
  is_sha "$sha" || die "unexpected SHA for $1@$2"
  echo "$sha"
}
head_awcms=$(resolve_sha awcms "$awcms_ref")
head_astro=$(resolve_sha awcms-astro "$astro_ref")

# Load rules (first match wins).
rule_repo=()
rule_glob=()
rule_class=()
while read -r r g c extra; do
  [[ -z "${r:-}" || "$r" == \#* ]] && continue
  [[ -n "${extra:-}" || -z "${c:-}" ]] && die "malformed rule: $r $g $c ${extra:-}"
  case "$c" in
    api-contract | migration-schema | authorization-security | astro-consumer | behaviour-parity | ci-docs | unclassified) ;;
    *) die "unknown class in rules: $c" ;;
  esac
  rule_repo+=("$r")
  rule_glob+=("$g")
  rule_class+=("$c")
done <"$rules"

classify() { # classify SHORT_REPO PATH
  local i
  for i in "${!rule_glob[@]}"; do
    if [[ "${rule_repo[$i]}" == "*" || "${rule_repo[$i]}" == "$1" ]]; then
      # shellcheck disable=SC2254 # the glob is intentionally a pattern
      case "$2" in
        ${rule_glob[$i]}) echo "${rule_class[$i]}"; return ;;
      esac
    fi
  done
  echo unclassified
}

classes=(api-contract migration-schema authorization-security astro-consumer behaviour-parity ci-docs unclassified)

# Upstream paths pinned by the frozen corpus (frozen/static/manifest.json).
pinned_awcms=(src/modules/index.ts docs/awcms/repo-inventory.md openapi/awcms-public-api.openapi.yaml
  asyncapi/awcms-domain-events.asyncapi.yaml scripts/api-consumer-contract.ts package.json .github/workflows/ci.yml)
pinned_astro=(tests/kontrak-awcms.test.mjs package.json)

work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT

# list_changed SHORT BASE HEAD OUTFILE -> prints method; writes sorted unique paths to OUTFILE
list_changed() {
  local short=$1 base=$2 head=$3 file=$4 raw="$work/$1.raw" n
  if gh api --paginate "repos/$owner/$short/compare/$base...$head?per_page=100" \
    --jq '.files[]? | .filename, (.previous_filename // empty)' >"$raw" 2>/dev/null; then
    n=$(sort -u "$raw" | grep -c . || true)
    # The compare API caps the file list at 300 entries; treat that as possibly truncated.
    if ((n < 300)); then
      sort -u "$raw" | grep . >"$file" || : >"$file"
      echo "compare-api"
      return
    fi
  fi
  # Fallback: shallow blobless fetch of both commits, then git diff --name-only.
  local dir="$work/$short.git"
  git init -q --bare "$dir"
  git -C "$dir" fetch -q --depth=1 --filter=blob:none "https://github.com/$owner/$short.git" "$base" "$head" \
    || die "shallow fetch failed for $short"
  git -C "$dir" diff --name-only --no-renames "$base" "$head" | sort -u | grep . >"$file" || : >"$file"
  echo "git-diff"
}

json_escape() { sed -e 's/\\/\\\\/g' -e 's/"/\\"/g'; }

md="$out_abs/baseline-delta.md"
js="$out_abs/baseline-delta.json"
exit_code=0
repo_json=()
md_body="$work/body.md"
: >"$md_body"

process_repo() { # process_repo SHORT BASE HEAD
  local short=$1 base=$2 head=$3 changed="$work/$1.changed" tsv="$work/$1.tsv"
  local method status ahead behind total path cls c meta
  declare -A cnt=()
  for c in "${classes[@]}"; do cnt[$c]=0; done

  if [[ "$base" == "$head" ]]; then
    : >"$changed"
    method="none"; status="identical"; ahead=0; behind=0
  else
    method=$(list_changed "$short" "$base" "$head" "$changed")
    meta=$(gh api "repos/$owner/$short/compare/$base...$head?per_page=1" \
      --jq '[.status, .ahead_by, .behind_by] | @tsv') || die "compare metadata failed for $short"
    read -r status ahead behind <<<"$meta"
  fi

  : >"$tsv"
  while IFS= read -r path; do
    cls=$(classify "$short" "$path")
    cnt[$cls]=$((cnt[$cls] + 1))
    printf '%s\t%s\n' "$cls" "$path" >>"$tsv"
  done <"$changed"
  total=$(grep -c . "$changed" || true)

  for c in api-contract migration-schema authorization-security astro-consumer; do
    if ((cnt[$c] > 0)) && ((exit_code < 2)); then exit_code=2; fi
  done
  if ((exit_code == 0)) && ((cnt[behaviour-parity] + cnt[unclassified] > 0)); then exit_code=3; fi

  local -a pinned
  if [[ "$short" == "awcms" ]]; then pinned=("${pinned_awcms[@]}"); else pinned=("${pinned_astro[@]}"); fi
  local pinned_hit=() p
  for p in "${pinned[@]}"; do
    if grep -Fxq "$p" "$changed"; then pinned_hit+=("$p"); fi
  done

  {
    printf '## %s/%s\n\n' "$owner" "$short"
    # shellcheck disable=SC2016 # backticks are literal Markdown
    printf -- '- Frozen SHA: `%s`\n- Observed SHA: `%s`\n' "$base" "$head"
    printf -- '- Compare status: %s (ahead %s, behind %s)\n' "$status" "$ahead" "$behind"
    printf -- '- File-list method: %s\n- Changed paths: %s\n\n' "$method" "$total"
    printf '| Class | Count |\n| --- | ---: |\n'
    for c in "${classes[@]}"; do printf '| %s | %s |\n' "$c" "${cnt[$c]}"; done
    printf '\nChanged files pinned by the frozen corpus: '
    if ((${#pinned_hit[@]} == 0)); then
      printf 'none\n\n'
    else
      printf '\n\n'
      # shellcheck disable=SC2016 # backticks are literal Markdown
      for p in "${pinned_hit[@]}"; do printf -- '- `%s`\n' "$p"; done
      printf '\n'
    fi
    for c in "${classes[@]}"; do
      ((cnt[$c] > 0)) || continue
      [[ "$c" == "ci-docs" ]] && continue
      printf '### %s (%s)\n\n' "$c" "${cnt[$c]}"
      awk -F'\t' -v c="$c" -v max=40 '$1 == c { n++; if (n <= max) printf "- `%s`\n", $2 }
        END { if (n > max) printf "- ... %d more (see the JSON report)\n", n - max }' <(sort -t$'\t' -k2,2 "$tsv")
      printf '\n'
    done
    printf '### ci-docs (%s)\n\nPaths omitted from the Markdown body; see the JSON report.\n\n' "${cnt[ci-docs]}"
  } >>"$md_body"

  local counts_json files_json
  counts_json=""
  for c in "${classes[@]}"; do
    counts_json+="${counts_json:+,}\"$c\":${cnt[$c]}"
  done
  files_json=""
  while IFS=$'\t' read -r cls path; do
    files_json+="${files_json:+,}{\"path\":\"$(printf '%s' "$path" | json_escape)\",\"class\":\"$cls\"}"
  done < <(sort -t$'\t' -k2,2 "$tsv")
  local pinned_json=""
  for p in "${pinned_hit[@]}"; do pinned_json+="${pinned_json:+,}\"$p\""; done
  repo_json+=("{\"repository\":\"$owner/$short\",\"frozen\":\"$base\",\"observed\":\"$head\",\"status\":\"$status\",\"aheadBy\":$ahead,\"behindBy\":$behind,\"method\":\"$method\",\"changedPaths\":$total,\"counts\":{$counts_json},\"pinnedFilesChanged\":[$pinned_json],\"files\":[$files_json]}")
}

process_repo awcms "$frozen_awcms" "$head_awcms"
process_repo awcms-astro "$frozen_astro" "$head_astro"

verdict="none-or-ci-docs-only"
case "$exit_code" in
  2) verdict="alert" ;;
  3) verdict="review" ;;
esac

{
  printf '# VG-01 legacy baseline delta\n\n'
  printf 'Verdict: **%s** (exit code %s)\n\n' "$verdict" "$exit_code"
  printf 'The frozen baseline was NOT refreshed by this report. A refresh requires an explicit decision, a provenance update, regenerated hashes, parity review and execution of the affected tests.\n\n'
  cat "$md_body"
} >"$md"

{
  printf '{"gate":"VG-01","report":"legacy-baseline-delta","baselineRefreshed":false,"verdict":"%s","exitCode":%s,"repositories":[' "$verdict" "$exit_code"
  sep=""
  for r in "${repo_json[@]}"; do printf '%s%s' "$sep" "$r"; sep=","; done
  printf ']}\n'
} >"$js"

echo "VG-01 baseline delta: verdict=$verdict exit=$exit_code report=$out_abs"
exit "$exit_code"
