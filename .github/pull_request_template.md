## Objective

<!-- What this change achieves and why. One or two sentences. -->

## Linked issue

Closes #

<!-- Use "Part of #n" instead of "Closes #n" if this PR does not finish the issue. -->

## Acceptance criteria

- [ ]

## Security and privacy

- [ ] No secret, credential, personal data or private repository detail is added.
- [ ] Security impact reviewed (authN/authZ, input handling, outbound calls, supply chain). Notes:

## Tenant and RLS impact

- [ ] No tenant-scoped table, policy, role or query is changed.
- [ ] If changed: FORCE RLS is kept, the runtime role stays non-owner and `NOBYPASSRLS`, and tests cover cross-tenant denial. Notes:

## Migration and compatibility

- [ ] No migration, contract, API or fixture change.
- [ ] If changed: migration is immutable once merged, compatibility impact and consumers are listed. Notes:

## Tests and evidence

<!-- Commands run and their results, or links to CI runs. -->

- [ ] `scripts/check-docs.sh` passes
- [ ] Other checks relevant to this change ran (state which):

## Rollback

<!-- How to revert safely. "Revert this commit" is enough for documentation-only changes. -->

## Changeset

- [ ] A changeset is added under `.changeset/`, or this change does not need one (state why).
