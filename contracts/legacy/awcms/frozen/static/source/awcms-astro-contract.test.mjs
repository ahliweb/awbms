/**
 * Gerbang kontrak terhadap awcms.
 *
 * Dua kelas cacat yang diuji di sini sama-sama **tidak menggagalkan apa pun**
 * saat terjadi, dan itulah sebabnya keduanya butuh tes alih-alih review:
 *
 *   1. **Token tenant lain terpasang di situs ini.** Build tetap hijau dan
 *      situs terisi penuh — dengan artikel milik tenant lain, di domain ini.
 *      Rantai resolusi lama tidak bisa melihatnya sama sekali: ia memilih
 *      tenant dari konfigurasi, sementara yang menentukan isi respons adalah
 *      token.
 *   2. **Halaman berhenti di batas satu permintaan.** Situs terbit dengan
 *      sebagian artikel hilang, tanpa satu pun error, dan yang hilang justru
 *      yang paling baru.
 *
 * Jalankan dengan `bun test`.
 */
import { test, describe, beforeEach, afterEach } from "bun:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";

import { resolveTenant, TenantNotConfiguredError } from "../src/lib/awcms/tenant.ts";

const TENANT_UUID = "3f2b1c0d-4e5f-6071-8293-a4b5c6d7e8f9";
const TENANT_HEX = "3f2b1c0d4e5f60718293a4b5c6d7e8f9";
const RAHASIA = "cPI7PnFNbwACjb7UhqEhl8huKM7Lw9FY40yuj4AQYa4";
const TOKEN = `awcmsm_${TENANT_HEX}_${RAHASIA}`;

describe("tenant diturunkan dari token", () => {
  test("token mesin menentukan tenant tanpa variabel lain", () => {
    const hasil = resolveTenant({ AWCMS_API_TOKEN: TOKEN });
    assert.equal(hasil.tenantId, TENANT_UUID);
    assert.equal(hasil.asserted, false);
  });

  test("AWCMS_TENANT_ID yang cocok menandai hasil sebagai terverifikasi", () => {
    const hasil = resolveTenant({
      AWCMS_API_TOKEN: TOKEN,
      AWCMS_TENANT_ID: TENANT_UUID.toUpperCase()
    });
    assert.equal(hasil.tenantId, TENANT_UUID);
    assert.equal(hasil.asserted, true);
  });

  test("AWCMS_TENANT_ID yang berbeda MENGGAGALKAN build", () => {
    // Inilah alasan berkas itu berhenti menjadi rantai dan menjadi assertion.
    // Tanpa tes ini, satu-satunya yang menjaganya adalah kalimat di komentar.
    assert.throws(
      () =>
        resolveTenant({
          AWCMS_API_TOKEN: TOKEN,
          AWCMS_TENANT_ID: "11111111-2222-4333-8444-555555555555"
        }),
      (error) =>
        error instanceof TenantNotConfiguredError &&
        /Tenant mismatch/.test(error.message)
    );
  });

  test("token yang hilang, bukan-mesin, atau cacat ditolak masing-masing dengan sebabnya", () => {
    assert.throws(
      () => resolveTenant({}),
      (e) => /AWCMS_API_TOKEN is not set/.test(e.message)
    );

    // Token sesi manusia: bentuknya sah sebagai bearer, tetapi ia mati sendiri
    // pada reset password atau step-up MFA — kegagalan yang datang berbulan
    // kemudian, di build yang tidak mengubah apa pun.
    assert.throws(
      () => resolveTenant({ AWCMS_API_TOKEN: "sesi-manusia-yang-terlihat-sah" }),
      (e) => /not a machine credential/.test(e.message)
    );

    assert.throws(
      () => resolveTenant({ AWCMS_API_TOKEN: `awcmsm_${TENANT_HEX}_pendek` }),
      (e) => /malformed/.test(e.message)
    );
  });

  test("variabel rantai lama ditolak, bukan diabaikan", () => {
    // Diabaikan diam-diam adalah keadaan terburuk: operator memasang
    // AWCMS_TENANT_CODE=produksi, mendapat isi tenant staging, dan tidak ada
    // apa pun di build yang menyebut mana dari keduanya yang dipercaya.
    for (const nama of ["AWCMS_TENANT_CODE", "AWCMS_DEFAULT_TENANT_CODE"]) {
      assert.throws(
        () => resolveTenant({ AWCMS_API_TOKEN: TOKEN, [nama]: "produksi" }),
        (e) => new RegExp(`${nama}.*no longer decide`).test(e.message),
        `${nama} seharusnya ditolak`
      );
    }
  });

  test("tempat build TIDAK meneruskan variabel yang sudah ditolak", () => {
    // Pasangan wajib dari tes di atas, dan kelas cacat yang berbeda.
    //
    // Menolak sebuah variabel hanya bermanfaat bila tidak ada yang mengirimkan
    // variabel itu. Sepanjang `ci.yml` dan `Dockerfile` masih meneruskan
    // `AWCMS_TENANT_CODE` dari repository/build variable — nilai yang
    // dokumentasi versi sebelumnya justru menyuruh mengisinya — sebuah situs
    // yang belum membersihkan konfigurasinya mendapat build yang GAGAL di CI,
    // dengan pesan yang tidak menyebut langkah mana yang mengirimkannya. Itu
    // bukan gerbang yang bekerja; itu gerbang yang menembak pemakainya sendiri.
    //
    // Arah sebaliknya ikut dijaga di sini: `AWCMS_TENANT_ID` HARUS diteruskan.
    // Ia assertion yang menangkap token tenant lain, dan tanpa baris itu ia
    // tidak pernah berjalan di dua tempat yang benar-benar membangun situs.
    const berkas = [".github/workflows/ci.yml", "Dockerfile"];

    for (const jalur of berkas) {
      const isi = readFileSync(jalur, "utf8");

      for (const pensiun of ["AWCMS_TENANT_CODE", "AWCMS_DEFAULT_TENANT_CODE"]) {
        // Komentar boleh menyebut namanya — justru di sanalah alasannya
        // ditulis. Yang dilarang adalah baris yang benar-benar meneruskannya.
        const meneruskan = isi
          .split("\n")
          .filter((baris) => !baris.trim().startsWith("#"))
          .some((baris) => baris.includes(pensiun));

        assert.equal(meneruskan, false, `${jalur} masih meneruskan ${pensiun}`);
      }

      assert.ok(
        isi.includes("AWCMS_TENANT_ID"),
        `${jalur} tidak meneruskan AWCMS_TENANT_ID, jadi assertion tenant tidak pernah jalan di sana`
      );
    }
  });
});

// ---------------------------------------------------------------------------
// Traversal daftar + hidrasi, dengan `fetch` ditiru.
// ---------------------------------------------------------------------------

const fetchAsli = globalThis.fetch;

/** Membangun satu post penuh; `ringkas()` memangkasnya seperti awcms memangkasnya. */
function buatPost(
  index,
  {
    locale = "id",
    status = "published",
    visibility = "public",
    grup,
    // Kedua stempel bisa disetel karena keduanya sekarang KEPUTUSAN, bukan
    // hiasan fixture: `publishedAt` menentukan apakah artikelnya terbit sama
    // sekali dan di urutan berapa ia muncul di seksi berita, `updatedAt`
    // menentukan `dateModified`. Nilai bawaannya tetap seperti sebelumnya
    // supaya setiap tes lama menguji hal yang sama persis.
    publishedAt = "2026-07-01T00:00:00.000Z",
    updatedAt = "2026-07-02T00:00:00.000Z",
    // Byline opt-in penulis (`awcms` ADR-0109). Dihilangkan seluruhnya bila
    // tidak disetel — bukan disetel ke `null` — supaya fixture bawaannya
    // menirukan sebuah awcms yang MENDAHULUI ADR itu dan tidak mengirim field
    // ini sama sekali. Itu keadaan yang harus tetap terbangun, dan sebuah
    // fixture yang selalu membawa field-nya tidak akan pernah mengujinya.
    authorByline,
    // Seksi artikel ini, dan bagaimana ia dinyatakan.
    //
    // `kategori` bawaannya `"panduan"` sehingga setiap kasus lama di berkas ini
    // menguji hal yang sama persis. `null` menghilangkan sidecar `awcmsAstro`
    // SELURUHNYA — bukan menyetelnya kosong — dan itulah bentuk sebuah post
    // yang ditulis editor di layar admin `awcms`, satu-satunya bentuk yang tak
    // pernah dilihat suite ini sampai sekarang.
    kategori = "panduan",
    termIds
  } = {}
) {
  return {
    id: `00000000-0000-4000-8000-${String(index).padStart(12, "0")}`,
    title: `Artikel ${index}`,
    slug: `artikel-${index}`,
    excerpt: null,
    contentJson:
      kategori === null
        ? { blocks: [] }
        : { awcmsAstro: { schemaVersion: 1, urutan: index, kategori } },
    status,
    visibility,
    metaDescription: `Meta ${index}`,
    canonicalUrl: null,
    locale,
    ...(grup === undefined ? {} : { translationGroupId: grup }),
    ...(authorByline === undefined ? {} : { authorByline }),
    ...(termIds === undefined ? {} : { termIds }),
    publishedAt,
    updatedAt,
    createdAt: `2026-07-01T00:00:${String(index).padStart(2, "0")}.000Z`
  };
}

/** ISO untuk `n` menit dari sekarang — dihitung, tidak pernah ditulis harfiah. */
function menitDariSekarang(n) {
  return new Date(Date.now() + n * 60 * 1000).toISOString();
}

function ringkas(post) {
  const { id, title, slug, status, visibility, locale, publishedAt, updatedAt, createdAt } = post;
  return { id, title, slug, status, visibility, locale, publishedAt, updatedAt, createdAt };
}

/**
 * `fetch` tiruan yang meniru kontrak awcms.
 *
 * Dua hal ditiru dengan sengaja, karena keduanya adalah PENOLAKAN di sisi awcms
 * dan diam-diam berperilaku salah bila ditiru longgar:
 *
 *   - tanpa `view=full`, daftar memberi RINGKASAN — tanpa `contentJson`,
 *     tanpa `metaDescription`, tanpa `translationGroupId`. Adapter yang lupa
 *     memintanya akan membangun situs kosong tanpa satu pun error, jadi tiruan
 *     ini menolak memberi baris penuh kecuali diminta;
 *   - `view=full` tanpa `order=created_at` dijawab 400.
 *
 * `jejak` mencatat setiap permintaan supaya tes bisa membuktikan tidak ada
 * lagi satu permintaan per post (N+1 yang dulu ada).
 */
function pasangFetchTiruan(
  posts,
  { ukuranHalaman = 100, jejak = [], media = new Map(), term = [] } = {}
) {
  globalThis.fetch = async (input) => {
    const url = new URL(String(input));
    jejak.push(url.pathname + url.search);

    // Kosakata taksonomi. Sejak seksi sebuah artikel bisa datang dari term
    // `awcms` — bukan hanya dari sidecar milik repo ini — `content.ts`
    // membaca endpoint ini di setiap build, jadi double ini harus tahu ia ada.
    //
    // Bawaannya larik KOSONG dan itu bukan kemalasan: sebuah tenant yang tidak
    // memakai kategori sama sekali adalah keadaan yang sah, dan seluruh kasus
    // lama di berkas ini menempatkan artikelnya lewat sidecar. Bawaan yang
    // berisi term akan membuat kasus-kasus itu lulus lewat jalur yang bukan
    // jalur yang mereka uji.
    if (url.pathname === "/api/v1/blog/terms") {
      return Response.json({
        success: true,
        data: { terms: term, nextCursor: null }
      });
    }

    if (url.pathname === "/api/v1/media/objects") {
      const ids = (url.searchParams.get("ids") ?? "").split(",").filter(Boolean);

      // Kontrak awcms: id yang tidak resolve DILAPORKAN di `unresolved`, tidak
      // dibuang diam-diam — itu yang membuat pemanggil bisa membedakan "tidak
      // punya gambar" dari "gambarnya hilang".
      return Response.json({
        success: true,
        data: {
          items: ids.filter((id) => media.has(id)).map((id) => ({ id, ...media.get(id) })),
          unresolved: ids.filter((id) => !media.has(id))
        }
      });
    }

    const detail = url.pathname.match(/^\/api\/v1\/blog\/posts\/(.+)$/);
    if (detail) {
      const post = posts.find((p) => p.id === detail[1]);
      return Response.json({ success: true, data: { ...post, termIds: [] } });
    }

    const view = url.searchParams.get("view") ?? "summary";

    if (view === "full" && url.searchParams.get("order") !== "created_at") {
      return Response.json(
        {
          success: false,
          error: {
            code: "VALIDATION_ERROR",
            message: "view=full requires order=created_at"
          }
        },
        { status: 400 }
      );
    }

    const cursor = url.searchParams.get("cursor");
    const mulai = cursor ? Number(atob(cursor)) : 0;
    const irisan = posts.slice(mulai, mulai + ukuranHalaman);
    const nextCursor =
      irisan.length === ukuranHalaman && mulai + ukuranHalaman < posts.length
        ? btoa(String(mulai + ukuranHalaman))
        : null;

    return Response.json({
      success: true,
      data: {
        posts: view === "full" ? irisan : irisan.map(ringkas),
        nextCursor
      }
    });
  };

  return jejak;
}

/**
 * Permukaan `awcms` yang benar-benar dipanggil build ini — diekstrak dari KODE,
 * bukan dipercaya dari dokumen.
 *
 * ## Kenapa gerbang ini ada, dan kenapa ia lintas-repo
 *
 * `ahliweb/awcms` menilai kesiapannya sebagian dari daftar permukaan yang
 * dikonsumsi repo ini. Pada 4 Agustus 2026 penilaian itu
 * (`docs/awcms/repo-assessment-2026-08-04.md`) mencatat **enam**, dan menyusun
 * rencana snapshot kontrak konsumen di atas angka tersebut. Tiga di antaranya
 * tidak pernah dipanggil build ini:
 *
 *   - `GET /api/v1/blog/posts/{id}` — DIHAPUS ADR-0018 (dulu N+1 per build);
 *   - `GET /api/v1/auth/session` — milik BFF portal yang belum ada;
 *   - `POST /api/v1/access/machine-credentials` — cara MANUSIA menerbitkan
 *     token, bukan panggilan build.
 *
 * Selisih itu bukan sekadar angka: sebuah kontrak konsumen yang membekukan tiga
 * permukaan yang tidak dikonsumsi akan mengikat repo SANA pada bentuk yang repo
 * SINI tidak pernah butuh, sambil membuat "kontraknya terjaga" terasa lebih
 * lengkap daripada kenyataannya.
 *
 * Yang bisa dilakukan repo ini adalah membuat daftarnya **tidak bisa salah**:
 * diekstrak dari sumber, dibandingkan dua arah dengan tabel di skill, dan merah
 * saat keduanya menyimpang. Permukaan keempat karena itu tidak bisa mendarat
 * diam-diam.
 */
describe("permukaan awcms yang dipanggil build", () => {
  /**
   * KEDUA berkas skill, masing-masing digerbangi sendiri.
   *
   * `bun run audit:translation` menjaga cermin tetap SEUSIA sumbernya — hash
   * yang cocok membuktikan ia diterjemahkan ulang saat sumbernya berubah, bukan
   * bahwa tabelnya memuat baris yang sama. Sebuah baris yang hilang dari cermin
   * karena itu lolos kedua gerbang terjemahan, dan pembaca Indonesianya melihat
   * daftar permukaan yang berbeda dari yang dipanggil kode. Penanda blok dan
   * kolom pertamanya berupa jalur `/api/v1/…`, jadi keduanya diperiksa dengan
   * pemeriksa yang sama meski prosanya berbeda bahasa.
   */
  const SKILL_BERKAS = [
    { berkas: ".claude/skills/awcms-astro-integrasi/SKILL.md", bahasa: "Inggris" },
    { berkas: ".claude/skills/awcms-astro-integrasi/SKILL.id.md", bahasa: "Indonesia" }
  ];

  /** Jalur `/api/v1/…` di dalam string literal `src/`, tanpa komentar. */
  function permukaanDiSumber() {
    const ditemukan = new Set();

    for (const nama of new Bun.Glob("**/*.{ts,astro}").scanSync("src")) {
      const isi = readFileSync(`src/${nama}`, "utf8")
        // Komentar dibuang lebih dulu: berkas di sini MEMERIKAN permukaan yang
        // tidak dipanggil jauh lebih sering daripada memanggilnya, dan sebuah
        // gerbang yang menghitung docblock akan melaporkan permukaan yang justru
        // sudah dihapus.
        .replace(/\/\*[\s\S]*?\*\//g, "")
        .replace(/^\s*\/\/.*$/gm, "");

      for (const [, jalur] of isi.matchAll(/["'`](\/api\/v1\/[^"'`\s?]*)["'`]/g)) {
        ditemukan.add(jalur);
      }
    }

    return ditemukan;
  }

  /**
   * Jalur di kolom pertama tabel bertanda di sebuah berkas skill.
   *
   * @param {string} berkas
   */
  function permukaanDiSkill(berkas) {
    const isi = readFileSync(berkas, "utf8");
    const blok = isi.split("<!-- permukaan:dipanggil:mulai -->")[1]?.split(
      "<!-- permukaan:dipanggil:selesai -->"
    )[0];

    assert.ok(blok, `penanda permukaan tidak ditemukan di ${berkas}`);

    return new Set(
      [...blok.matchAll(/^\|\s*`(\/api\/v1\/[^`]+)`\s*\|/gm)].map((m) => m[1])
    );
  }

  /**
   * Jalur di blok "dijanjikan" — permukaan yang ADA di `src/` dan TIDAK dipanggil.
   *
   * Kontrak `awcms` sendiri memisahkan CONSUMED dari COMMITTED, dan alasannya
   * berpindah persis: sebuah janji dan sebuah ketergantungan sama-sama layak
   * stabil tetapi gagal dengan cara berbeda. Tanpa daftar kedua ini, sebuah
   * jalur yang ditulis di belakang fitur yang dimatikan hanya punya dua nasib,
   * dan keduanya buruk: masuk ke tabel "dipanggil" — yang menjadikan tabel itu
   * berbohong tentang apa yang dilakukan build — atau disembunyikan dari
   * gerbang dengan menyusun jalurnya dari potongan string, yang persis bypass
   * yang disebut ADR-0038 §4 sebagai batas yang diketahui.
   *
   * @param {string} berkas
   */
  function permukaanDijanjikan(berkas) {
    const isi = readFileSync(berkas, "utf8");
    const blok = isi.split("<!-- permukaan:dijanjikan:mulai -->")[1]?.split(
      "<!-- permukaan:dijanjikan:selesai -->"
    )[0];

    assert.ok(blok, `penanda permukaan dijanjikan tidak ditemukan di ${berkas}`);

    return new Set(
      [...blok.matchAll(/^\|\s*`(\/api\/v1\/[^`]+)`\s*\|/gm)].map((m) => m[1])
    );
  }

  test("kode sumber memanggil tepat tiga belas permukaan", () => {
    // Daftarnya ditulis eksplisit supaya permukaan BERIKUTNYA memerahkan gerbang
    // ini meskipun penulisnya ingat memperbarui skill — dua pemeriksaan yang bisa
    // salah bersama bukan dua pemeriksaan.
    //
    // Yang keempat, `/api/v1/site-profile/composed`, mendarat 22 Agustus 2026
    // (`awcms` #596): identitas situs — masthead, footer, kontak, tautan sosial,
    // simpul `Organization`. Ia dibekukan di `awcms` LEBIH DULU sebagai path
    // COMMITTED, baru dipanggil dari sini; Definition of Done repo ini menuntut
    // urutan itu, dan urutan sebaliknya berarti build di sini bersandar pada
    // bentuk yang belum disanggupi repo sebelah.
    //
    // Yang kelima, `/api/v1/blog/terms`, mendarat dengan urutan yang sama
    // (`awcms` #597 butir 1, ADR-0104): kosakata tenant, yang tanpa dia arsip
    // kategori maupun tag tidak bisa dibangun sama sekali.
    //
    // Keenam dan ketujuh, `/api/v1/blog/menus` dan `/api/v1/blog/widgets`
    // (`awcms` #597 butir 6, ADR-0105 di sana). Keduanya baru bisa dibekukan
    // SETELAH `awcms` #652 memberi responsnya skema sungguhan: sebelum itu
    // keduanya array `object` telanjang, dan membekukannya sama dengan
    // membekukan janji yang tidak bisa gagal terhadap apa pun.
    //
    // Kedelapan dan kesembilan, `/api/v1/site-search/query` dan `/suggest`
    // (`awcms` #607 + #597 butir 3, ADR-0107 di sana; ADR-0043 di sini).
    // Keduanya BERBEDA KELAS dari tujuh di atasnya dan itu pantas dibaca
    // sebelum yang kesepuluh datang: yang memanggilnya adalah peramban
    // PEMBACA saat runtime, bukan build ini. Konsekuensinya bagi gerbang ini
    // nol — ia mengekstrak string literal dari `src/`, dan siapa yang
    // mengeksekusinya bukan sesuatu yang bisa dilihat sebuah regex.
    // Konsekuensinya bagi yang membacanya tidak nol: sebuah perubahan bentuk
    // pada kedua permukaan itu gagal di peramban orang asing, bukan di build
    // yang bisa dilihat siapa pun.
    //
    // Yang kesepuluh, `/api/v1/analytics/collect` (`awcms` #597 butir 9,
    // ADR-0044 di sini), termasuk kelas yang sama — peramban pembaca — dengan
    // satu perbedaan yang PENTING dan berlawanan arah: ia satu-satunya
    // permintaan di repo ini yang membawa header, dan HARUS membawanya.
    // `security.checkOrigin` di `awcms` menolak POST lintas-origin yang tipe
    // isinya mirip form, jadi hanya `application/json` yang lolos — dan handler
    // `OPTIONS` di sana ada justru untuk preflight yang ditimbulkannya.
    // Menyamakan aturannya dengan dua permukaan pencarian, ke arah mana pun,
    // mematikan salah satunya di peramban.
    //
    // Yang kesebelas sampai ketiga belas — `/api/v1/newsletter/subscribe`,
    // `/confirm`, `/unsubscribe` (ADR-0049 di sini, `awcms` ADR-0103/0118) —
    // kelas peramban-pembaca yang sama dengan satu perbedaan yang belum pernah
    // ada di daftar ini: mereka MENULIS. Sebuah kiriman membuat `awcms`
    // mengirim surat ke alamat yang diketik seseorang, jadi bentuk yang salah
    // di sini tidak sekadar membuat halaman kosong — ia mengirimkan sesuatu,
    // atau diam-diam berhenti mengirim, kepada orang yang memintanya.
    //
    // Ketiganya duduk di blok `dijanjikan` selama satu hari. Empat hal terukur
    // di `awcms` membuat endpoint-nya tak terjangkau dari situs statis — tanpa
    // `OPTIONS`, tanpa `Access-Control-Allow-Origin`, tenant diselesaikan dari
    // HOST yang adalah CMS itu sendiri, dan tautan konfirmasi dibangun di
    // origin CMS tempat halamannya tidak ada — dan ADR-0118 di sana menutup
    // keempatnya lalu membekukan ketiga jalurnya sebagai COMMITTED. Urutan itu
    // yang membuat baris ini boleh ada.
    const sumber = permukaanDiSumber();
    const dijanjikan = permukaanDijanjikan(SKILL_BERKAS[0].berkas);

    // Yang DIPANGGIL adalah sumber dikurangi yang dijanjikan. Pengurangannya
    // ditulis di sini, bukan disembunyikan di dalam `permukaanDiSumber`, supaya
    // sebuah jalur yang dipindahkan dari "dijanjikan" ke "dipanggil" tetap
    // harus melewati daftar eksplisit di bawah.
    const dipanggil = [...sumber].filter((jalur) => !dijanjikan.has(jalur));

    assert.deepEqual(
      dipanggil.sort(),
      [
        "/api/v1/analytics/collect",
        "/api/v1/blog/menus",
        "/api/v1/blog/posts",
        "/api/v1/blog/terms",
        "/api/v1/blog/widgets",
        "/api/v1/media/objects",
        "/api/v1/media/public-origin",
        "/api/v1/newsletter/confirm",
        "/api/v1/newsletter/subscribe",
        "/api/v1/newsletter/unsubscribe",
        "/api/v1/site-profile/composed",
        "/api/v1/site-search/query",
        "/api/v1/site-search/suggest"
      ],
      "permukaan awcms yang dipanggil src/ berubah — bila ini disengaja, " +
        "perbarui tabel bertanda di skill integrasi DAN beri tahu `awcms`: " +
        "repo itu menyusun kontrak konsumennya dari daftar ini"
    );
  });

  for (const { berkas, bahasa } of SKILL_BERKAS) {
    test(`tabel di ${berkas} sama persis dengan kode, dua arah`, () => {
      // Dua arah, bukan satu: sebuah baris yang TERTINGGAL di skill setelah
      // permukaannya dihapus adalah cacat yang sama dengan permukaan baru yang
      // tidak dicatat — dan yang pertama justru yang sudah pernah terjadi di sini
      // (`/posts/{id}` bertahan di dokumen berbulan-bulan setelah ADR-0018
      // menghapus panggilannya).
      const dijanjikan = permukaanDijanjikan(berkas);

      assert.deepEqual(
        [...permukaanDiSkill(berkas)].sort(),
        [...permukaanDiSumber()].filter((jalur) => !dijanjikan.has(jalur)).sort(),
        `tabel bertanda di ${berkas} (${bahasa}) menyimpang dari permukaan yang dipanggil src/`
      );
    });

    test(`blok dijanjikan di ${berkas} tidak tumpang tindih dengan yang dipanggil`, () => {
      // Sebuah jalur di KEDUA blok adalah jalur yang salah satunya tidak pernah
      // dibaca lagi — dan yang tidak dibaca selalu yang mengatakan kebenaran
      // yang tidak nyaman.
      const dipanggil = permukaanDiSkill(berkas);
      const bertumpuk = [...permukaanDijanjikan(berkas)].filter((j) => dipanggil.has(j));

      assert.deepEqual(bertumpuk, [], `${berkas} (${bahasa})`);
    });

    test(`setiap permukaan dijanjikan di ${berkas} ADA di sumber`, () => {
      // Arah kedua, dan ia yang membuat daftar ini tidak membusuk: sebuah janji
      // yang kodenya sudah dihapus adalah baris yang terus menjanjikan sesuatu
      // yang tidak ada lagi.
      const sumber = permukaanDiSumber();
      const hantu = [...permukaanDijanjikan(berkas)].filter((j) => !sumber.has(j));

      assert.deepEqual(hantu, [], `${berkas} (${bahasa}) menjanjikan jalur yang tidak ada di src/`);
    });
  }

  test("tidak ada permukaan yang DIJANJIKAN sekaligus sudah bisa dipanggil", async () => {
    // Yang dibeli blok kedua itu. Sebuah jalur boleh duduk di sana hanya
    // selama fiturnya mati; begitu flag-nya menyala, ia permukaan yang
    // dipanggil dan harus pindah ke tabel pertama beserta seluruh kewajibannya.
    //
    // Blok itu KOSONG sejak 28 Agustus 2026, dan kosong adalah keadaan yang sah
    // — bukan alasan untuk menghapus pemeriksaan ini. Yang diperiksa bukan
    // "ada berapa", melainkan bahwa tidak satu pun yang tersisa di sana sambil
    // fiturnya sudah menyala. Menuntut isinya tidak kosong akan memerahkan repo
    // ini setiap kali sebuah janji akhirnya ditepati, yang justru saat tidak
    // ada yang berbuat salah.
    const dijanjikan = permukaanDijanjikan(SKILL_BERKAS[0].berkas);
    const { newsletterAktif } = await import("../src/config/site.ts");

    if (dijanjikan.has("/api/v1/newsletter/subscribe")) {
      assert.equal(
        newsletterAktif,
        false,
        "newsletterAktif menyala sementara /api/v1/newsletter/subscribe masih di " +
          "blok `dijanjikan`. Pindahkan barisnya ke tabel permukaan yang dipanggil."
      );
    }

    // Repo TEMPLATE ini sendiri tidak menyatakan `SITE_NEWSLETTER`, jadi
    // formulirnya tidak terbit di sini apa pun keadaan blok di atas. Sebuah
    // SITUS turunan yang menyatakannya adalah yang menerbitkannya — dan di sana
    // ketiga jalurnya sudah ada di tabel yang dipanggil, tempat kewajibannya
    // ikut terbaca.
    assert.equal(
      newsletterAktif,
      false,
      "template ini menerbitkan formulir buletin tanpa SITE_NEWSLETTER dinyatakan"
    );
  });
});

describe("traversal build feed", () => {
  let getArticles;
  let resetContentCacheForTests;

  beforeEach(async () => {
    process.env.AWCMS_API_URL = "http://awcms.uji";
    process.env.AWCMS_API_TOKEN = TOKEN;
    delete process.env.AWCMS_TENANT_CODE;
    delete process.env.AWCMS_DEFAULT_TENANT_CODE;

    ({ getArticles, resetContentCacheForTests } = await import("../src/lib/content.ts"));
    resetContentCacheForTests();
  });

  afterEach(() => {
    globalThis.fetch = fetchAsli;
  });

  test("melewati batas satu halaman tanpa kehilangan satu artikel pun", async () => {
    // 250 post, halaman 100: sebuah build yang berhenti di halaman pertama
    // menerbitkan 100 artikel dan tidak melaporkan apa pun tentang 150 sisanya.
    const posts = Array.from({ length: 250 }, (_, i) => buatPost(i));
    pasangFetchTiruan(posts);

    const artikel = await getArticles("panduan", "id");
    assert.equal(artikel.length, 250);
  });

  test("draft dan post non-publik tidak pernah terbit", async () => {
    const posts = [
      buatPost(0),
      buatPost(1, { status: "draft" }),
      buatPost(2, { visibility: "private" })
    ];
    pasangFetchTiruan(posts);

    const artikel = await getArticles("panduan", "id");
    assert.equal(artikel.length, 1);
  });

  test("satu traversal, bukan satu permintaan per post", async () => {
    // Adapter ini pernah menyusuri daftar lalu mengambil ULANG setiap post
    // lewat /posts/{id} — N+1 permintaan ke endpoint admin pada setiap publish.
    // Sejak awcms punya `view=full`, permintaan per-id itu tidak boleh muncul
    // lagi; kalau ia kembali, ia kembali diam-diam.
    const posts = Array.from({ length: 5 }, (_, i) => buatPost(i));
    const jejak = pasangFetchTiruan(posts);

    await getArticles("panduan", "id");

    const perId = jejak.filter((j) => /\/api\/v1\/blog\/posts\/[^?]/.test(j));
    assert.deepEqual(perId, [], "tidak boleh ada permintaan per post");

    // Dihitung PER ENDPOINT, bukan sebagai total.
    //
    // Asersi ini semula berbunyi `jejak.length === 1`, dan itu berhenti benar
    // saat seksi artikel mulai dibaca dari taksonomi: build kini juga memanggil
    // `/blog/terms` sekali. Melonggarkannya menjadi "dua" akan membuat ia
    // berhenti menjaga apa pun — angka yang naik satu setiap kali seseorang
    // menambah permintaan.
    //
    // Yang dijaga adalah bentuknya: daftar post ditelusuri sekali per halaman,
    // dan kosakata diambil sekali untuk seluruh build betapa pun banyak tab
    // yang meminta artikel. Keduanya BUKAN per-post, dan itulah yang tidak
    // boleh kembali.
    const daftarPost = jejak.filter((j) => j.startsWith("/api/v1/blog/posts?"));
    const daftarTerm = jejak.filter((j) => j.startsWith("/api/v1/blog/terms"));

    assert.equal(daftarPost.length, 1, "satu halaman post = satu permintaan");
    assert.equal(daftarTerm.length, 1, "kosakata diambil sekali per build");
    assert.equal(
      jejak.length,
      daftarPost.length + daftarTerm.length,
      `tidak ada permintaan lain yang diharapkan; jejak: ${jejak.join(", ")}`
    );
  });

  test("adapter meminta view=full atas urutan yang stabil", async () => {
    // Dua parameter ini yang membedakan "situs terbit" dari "situs kosong":
    // tanpa view=full daftar tidak memuat contentJson sama sekali, dan awcms
    // menolak view=full di atas urutan mutable.
    const jejak = pasangFetchTiruan([buatPost(0)]);

    await getArticles("panduan", "id");

    assert.match(jejak[0], /view=full/);
    assert.match(jejak[0], /order=created_at/);
    assert.match(jejak[0], /status=published/);
  });

  test("isi artikel benar-benar sampai ke kontrak komponen", async () => {
    // Regresi yang dibayar mahal: adapter ini pernah membaca `contentJson` dari
    // respons yang tidak pernah memuatnya. Hasilnya build hijau dengan seluruh
    // seksi kosong — `kategori` juga tinggal di dalam `contentJson`.
    const posts = [buatPost(7)];
    pasangFetchTiruan(posts);

    const [artikel] = await getArticles("panduan", "id");
    assert.equal(artikel.entry.data.kategori, "panduan");
    assert.equal(artikel.entry.data.description, "Meta 7");
  });

  test("awcms yang MENGABAIKAN view=full menggagalkan build", async () => {
    // awcms lama tidak menolak parameter yang tidak dikenalnya — ia
    // mengabaikannya dan menjawab ringkasan. Tanpa gerbang ini, situs terbit
    // dengan setiap badan artikel kosong dan setiap seksi kosong, hijau.
    const posts = [buatPost(0), buatPost(1)];
    globalThis.fetch = async () =>
      Response.json({
        success: true,
        data: { posts: posts.map(ringkas), nextCursor: null }
      });

    await assert.rejects(
      () => getArticles("panduan", "id"),
      (e) => /ignored \?view=full/.test(e.message)
    );
  });

  test("terjemahan yang tidak bisa dipasangkan menggagalkan build", async () => {
    const posts = [buatPost(0), buatPost(1, { locale: "en" })];
    pasangFetchTiruan(posts);

    await assert.rejects(
      () => getArticles("panduan", "id"),
      (e) => /translationGroupId/.test(e.message)
    );
  });

  test("awcms yang menerima koneksi lalu DIAM menggagalkan build", async () => {
    // Kelas cacat yang berbeda dari setiap gerbang lain di berkas ini, dan
    // satu-satunya yang tidak menghasilkan keluaran salah: ia tidak
    // menghasilkan keluaran sama sekali.
    //
    // Sebuah awcms yang menerima koneksi lalu tidak pernah menjawab bukan
    // keadaan hipotetis — ia bentuk kegagalan paling umum dari basis data yang
    // kehabisan koneksi, dan `fetch` tidak punya batas waktu bawaan. Tanpa
    // deadline, build menggantung sampai batas job CI membunuhnya (15 menit,
    // dengan pesan yang menyebut nama job alih-alih awcms) atau, di mesin
    // lokal, selamanya.
    process.env.AWCMS_API_TIMEOUT_MS = "80";

    // Tiruan yang meniru `fetch` sungguhan: ia menghormati sinyal dan menolak
    // dengan `TimeoutError`. Tiruan yang langsung menolak akan menghijaukan
    // kode yang tidak pernah memasang sinyalnya sama sekali.
    globalThis.fetch = (_url, init) =>
      new Promise((_resolve, reject) => {
        init?.signal?.addEventListener("abort", () =>
          reject(init.signal.reason ?? new Error("aborted"))
        );
      });

    try {
      await assert.rejects(
        () => getArticles("panduan", "id"),
        (e) => e.code === "TIMEOUT" && /within 80 ms/.test(e.message)
      );
    } finally {
      delete process.env.AWCMS_API_TIMEOUT_MS;
    }
  });

  test("batas waktu yang tidak bisa dibaca DITOLAK, bukan diabaikan", async () => {
    // Nilai yang terbaca seperti konfigurasi dan tidak memutuskan apa pun
    // adalah kelas cacat yang repo ini paling sering menulis aturan
    // terhadapnya. `0` disebut khusus karena ia terlihat seperti "tanpa batas"
    // dan justru mengembalikan keadaan yang gerbang di atas ada untuk mencegah.
    for (const nilai of ["nanti", "0", "-1"]) {
      process.env.AWCMS_API_TIMEOUT_MS = nilai;
      resetContentCacheForTests();
      globalThis.fetch = async () => {
        throw new Error("tidak boleh sampai ke jaringan");
      };

      try {
        await assert.rejects(
          () => getArticles("panduan", "id"),
          (e) => e.code === "CONFIG_INVALID",
          nilai
        );
      } finally {
        delete process.env.AWCMS_API_TIMEOUT_MS;
      }
    }
  });

  test("terjemahan yang membawa grup dipasangkan, bukan ditolak", async () => {
    // Gerbang di atas adalah assertion atas DATA, bukan pemeriksaan versi
    // awcms: begitu field-nya benar-benar dikembalikan, jalur ini lewat tanpa
    // ada yang perlu diubah di sini.
    const posts = [
      buatPost(0, { grup: "grup-1" }),
      buatPost(1, { locale: "en", grup: "grup-1" })
    ];
    pasangFetchTiruan(posts);

    const artikel = await getArticles("panduan", "en");
    assert.equal(artikel.length, 1);
    assert.equal(artikel[0].isFallback, false);
    assert.equal(artikel[0].slug, "artikel-0");
  });
});

// ---------------------------------------------------------------------------
// Byline penulis — `awcms` ADR-0109, #597 butir 4.
//
// Empat keadaan, dan tidak satu pun bisa dilihat typecheck: ketiganya bertipe
// `string | undefined` sesudah adapter, dan yang membedakannya adalah BARIS
// MANA yang dibaca dan apa yang terjadi pada nilai kosong. Yang membuatnya
// pantas diuji: setiap kesalahan di sini menerbitkan nama seseorang di halaman
// publik — bukan halaman yang rusak, melainkan halaman yang tampak benar dan
// mengkredit orang yang salah.
// ---------------------------------------------------------------------------
describe("byline penulis (awcms ADR-0109)", () => {
  let getArticles;
  let resetContentCacheForTests;

  beforeEach(async () => {
    process.env.AWCMS_API_URL = "http://awcms.uji";
    process.env.AWCMS_API_TOKEN = TOKEN;
    delete process.env.AWCMS_TENANT_CODE;
    delete process.env.AWCMS_DEFAULT_TENANT_CODE;

    ({ getArticles, resetContentCacheForTests } = await import("../src/lib/content.ts"));
    resetContentCacheForTests();
  });

  afterEach(() => {
    globalThis.fetch = fetchAsli;
  });

  test("awcms yang tidak mengirim field ini menghasilkan artikel tanpa byline, bukan build gagal", async () => {
    pasangFetchTiruan([buatPost(0)]);

    const artikel = await getArticles("panduan", "id");
    assert.equal(artikel[0].authorByline, undefined);
  });

  test("null — keadaan setiap baris sebelum ADR-0109 — berarti atribusi organisasi", async () => {
    // `null` dan "field tidak ada" harus tiba di komponen sebagai SATU keadaan.
    // Dua ejaan untuk satu keadaan adalah bagaimana salah satunya berakhir tidak
    // ditangani — di sini akibatnya adalah `null` yang tercetak sebagai kata.
    pasangFetchTiruan([buatPost(0, { authorByline: null })]);

    const artikel = await getArticles("panduan", "id");
    assert.equal(artikel[0].authorByline, undefined);
  });

  test("byline yang hanya spasi diperlakukan sebagai tidak ada", async () => {
    // Kalau ia lolos, halamannya memasang baris "Ditulis oleh" dan tidak
    // menyebut siapa pun — sebuah klaim kepenulisan tanpa penulis.
    pasangFetchTiruan([buatPost(0, { authorByline: "   " })]);

    const artikel = await getArticles("panduan", "id");
    assert.equal(artikel[0].authorByline, undefined);
  });

  test("byline dibaca dari baris TERJEMAHAN, bukan dari baris sumber", async () => {
    // Ini keputusan yang membedakannya dari `termIds`/`urutan`/`kategori`, yang
    // semuanya dibaca dari sumber supaya klasifikasi tidak berbeda antar bahasa.
    // Kepenulisan bukan klasifikasi: terjemahan sering ditulis orang lain, dan
    // mengambil nama penulis sumber untuknya mengkredit seseorang atas teks yang
    // tidak ia tulis. Membaliknya lolos typecheck dan lolos setiap gerbang lain.
    pasangFetchTiruan([
      buatPost(0, { grup: "grup-1", authorByline: "Penulis Sumber" }),
      buatPost(1, { locale: "en", grup: "grup-1", authorByline: "Translating Author" })
    ]);

    const [artikel] = await getArticles("panduan", "en");
    assert.equal(artikel.isFallback, false);
    assert.equal(artikel.authorByline, "Translating Author");
  });

  test("artikel fallback membawa byline penulis SUMBER, karena kata-katanya memang miliknya", async () => {
    // Sisi lain dari tes di atas, dan bukan pengecualian terhadapnya: sebuah
    // artikel fallback ADALAH post sumber, ditampilkan di locale lain. Nama yang
    // benar untuknya adalah nama orang yang menulis kata-kata di layar.
    pasangFetchTiruan([buatPost(0, { authorByline: "Penulis Sumber" })]);

    const [artikel] = await getArticles("panduan", "en");
    assert.equal(artikel.isFallback, true);
    assert.equal(artikel.authorByline, "Penulis Sumber");
  });
});

// ---------------------------------------------------------------------------
// Predikat terbit — paritas dengan rute publik awcms sendiri.
//
// `awcms` menyajikan `/news/**` dengan `published_at IS NOT NULL AND
// published_at <= now()` di atas `status`/`visibility`. Build feed yang dipanggil
// repo ini TIDAK menerapkan predikat itu — `listBlogPostsFullPage` menyaring
// tenant, `deleted_at`, `status`, dan `locale` saja. Tanpa gerbang di bawah,
// aturan itu bisa dicabut dari adapter tanpa satu tes pun berubah warna, karena
// setiap fixture di berkas ini bertanggal masa lalu dan akan selamanya begitu.
// ---------------------------------------------------------------------------

describe("predikat terbit", () => {
  let getArticles;
  let resetContentCacheForTests;

  beforeEach(async () => {
    process.env.AWCMS_API_URL = "http://awcms.uji";
    process.env.AWCMS_API_TOKEN = TOKEN;
    ({ getArticles, resetContentCacheForTests } = await import("../src/lib/content.ts"));
    resetContentCacheForTests();
  });

  afterEach(() => {
    globalThis.fetch = fetchAsli;
  });

  test("post berstatus published tanpa publishedAt tidak ikut terbit", async () => {
    // awcms akan menjawab 404 untuk post seperti ini di `/news/{slug}`-nya
    // sendiri. Menerbitkannya di sini berarti situs statis dan CMS-nya tidak
    // sepakat tentang apa yang sudah tayang.
    const posts = [buatPost(0), buatPost(1, { publishedAt: null })];
    pasangFetchTiruan(posts);

    const artikel = await getArticles("panduan", "id");
    assert.equal(artikel.length, 1);
    assert.equal(artikel[0].slug, "artikel-0");
  });

  test("post terjadwal jauh di masa depan tidak ikut terbit", async () => {
    const posts = [buatPost(0), buatPost(1, { publishedAt: menitDariSekarang(60 * 24 * 30) })];
    pasangFetchTiruan(posts);

    const artikel = await getArticles("panduan", "id");
    assert.equal(artikel.length, 1);
  });

  test("condong jam beberapa menit TIDAK membuang artikel yang baru terbit", async () => {
    // Jalur konten normal adalah terbit → webhook → build, berjarak detik, dan
    // kedua stempel yang dibandingkan datang dari dua mesin: awcms menstempel
    // `published_at` dari jam BASIS DATA, perbandingan ini berjalan di jam
    // BUILDER. Tanpa toleransi, builder yang tertinggal semenit membuang artikel
    // yang baru saja diterbitkan — dan di seksi berita itu kartu PERTAMA.
    const posts = [buatPost(0, { publishedAt: menitDariSekarang(3) })];
    pasangFetchTiruan(posts);

    const artikel = await getArticles("panduan", "id");
    assert.equal(artikel.length, 1);
  });

  test("NOL post yang bisa terbit MENGGAGALKAN build", async () => {
    // Penyaring yang hanya bisa mengurangi butuh lantai, alasan yang sama
    // dengan gerbang nol-dari-N pada media. Tanpa ini: setiap seksi kosong,
    // beranda mencetak "0 artikel", dan tidak ada yang merah di mana pun.
    const posts = [buatPost(0, { publishedAt: null }), buatPost(1, { publishedAt: null })];
    pasangFetchTiruan(posts);

    await assert.rejects(
      () => getArticles("panduan", "id"),
      (e) => /NOT ONE of them carries a usable publishedAt/.test(e.message)
    );
  });

  test("publishedAt yang tidak bisa diurai MENGGAGALKAN build, bukan menjadi daftar kosong", async () => {
    // Bedanya menentukan: `false` berarti "belum waktunya terbit", keadaan
    // normal yang membuang satu artikel. Tanggal yang bukan tanggal berarti
    // bentuk respons berubah — dan karena setiap perbandingan terhadap `NaN`
    // bernilai false, memperlakukannya sebagai "belum terbit" akan membuang
    // SETIAP artikel sekaligus dan terbaca seperti CMS yang masih kosong.
    const posts = [buatPost(0, { publishedAt: "1785000000" })];
    pasangFetchTiruan(posts);

    await assert.rejects(
      () => getArticles("panduan", "id"),
      (e) => /not a date this build can parse/.test(e.message)
    );
  });

  test("kedua tanggal datang dari BARIS yang sama, bukan dari dua baris", async () => {
    // Pasangan yang salah menghasilkan `dateModified` MENDAHULUI
    // `datePublished` pada konten yang sepenuhnya normal: artikel sumber yang
    // baru diterbitkan bulan ini, terjemahannya tidak disentuh sejak bulan lalu.
    // awcms membaca keduanya dari satu baris; ini membuktikan repo ini ikut.
    const posts = [
      buatPost(0, {
        grup: "grup-1",
        publishedAt: "2026-08-01T00:00:00.000Z",
        updatedAt: "2026-08-03T00:00:00.000Z"
      }),
      buatPost(1, {
        locale: "en",
        grup: "grup-1",
        publishedAt: "2026-07-10T00:00:00.000Z",
        updatedAt: "2026-07-12T00:00:00.000Z"
      })
    ];
    pasangFetchTiruan(posts);

    const [en] = await getArticles("panduan", "en");

    assert.equal(en.entry.data.publishedDate.toISOString(), "2026-07-10T00:00:00.000Z");
    assert.equal(en.entry.data.updatedDate.toISOString(), "2026-07-12T00:00:00.000Z");
    assert.ok(
      en.entry.data.publishedDate <= en.entry.data.updatedDate,
      "dateModified tidak boleh mendahului datePublished"
    );
  });

  test("artikel fallback memakai tanggal post sumber, bukan tanggal kosong", async () => {
    const posts = [
      buatPost(0, { publishedAt: "2026-08-01T00:00:00.000Z", updatedAt: "2026-08-05T00:00:00.000Z" })
    ];
    pasangFetchTiruan(posts);

    const [en] = await getArticles("panduan", "en");

    assert.equal(en.isFallback, true);
    assert.equal(en.entry.data.publishedDate.toISOString(), "2026-08-01T00:00:00.000Z");
    assert.equal(en.entry.data.updatedDate.toISOString(), "2026-08-05T00:00:00.000Z");
  });
});

// ---------------------------------------------------------------------------
// Urutan seksi.
//
// Diuji lewat fungsi murni `urutkanArtikel`, bukan lewat `getArticles`, dan itu
// disengaja: cabang yang dipilih `getArticles` datang dari `siteConfig.tabs`,
// setiap tab yang dibawa template ini bernilai "manual", dan repo template tidak
// punya instans awcms untuk membangun apa pun. Diuji lewat `getArticles`, cabang
// "terbaru" akan menjadi kode yang tidak pernah dieksekusi di repo yang
// memilikinya — eksekusi pertamanya terjadi di build produksi sebuah situs.
// ---------------------------------------------------------------------------

describe("urutan seksi", () => {
  let urutkanArtikel;

  beforeEach(async () => {
    ({ urutkanArtikel } = await import("../src/lib/content.ts"));
  });

  /** Kunci urut minimal; `judul` sengaja dibuat berlawanan arah dengan `terbitSumber`. */
  const CONTOH = [
    { urutan: 2, judul: "Anggur", terbit: Date.parse("2026-01-01"), slugSumber: "anggur" },
    { urutan: 1, judul: "Ceri", terbit: Date.parse("2026-03-01"), slugSumber: "ceri" },
    { urutan: 3, judul: "Belimbing", terbit: Date.parse("2026-02-01"), slugSumber: "belimbing" }
  ];

  test('"manual" mengurutkan dari urutan redaksi, bukan dari tanggal', () => {
    assert.deepEqual(
      urutkanArtikel(CONTOH, "manual").map((x) => x.slugSumber),
      ["ceri", "anggur", "belimbing"]
    );
  });

  test('"terbaru" mengurutkan dari publishedAt sumber, terbaru lebih dulu', () => {
    assert.deepEqual(
      urutkanArtikel(CONTOH, "terbaru").map((x) => x.slugSumber),
      ["ceri", "belimbing", "anggur"]
    );
  });

  test('"terbaru" MENGABAIKAN urutan redaksi sepenuhnya', () => {
    // Kalau `urutan` masih ikut menentukan, seksi berita akan berhenti berurutan
    // begitu satu artikel kebetulan membawa nilai `urutan` — dan nilai bawaannya
    // 99 untuk setiap artikel yang tidak pernah dinomori siapa pun.
    const dinomori = CONTOH.map((x, i) => ({ ...x, urutan: 10 - i }));
    assert.deepEqual(
      urutkanArtikel(dinomori, "terbaru").map((x) => x.slugSumber),
      ["ceri", "belimbing", "anggur"]
    );
  });

  test("seri dipecah secara deterministik di KEDUA cabang", () => {
    // `Array#sort` stabil, jadi comparator yang mengembalikan 0 menyerahkan
    // pasangannya pada urutan yang kebetulan dikembalikan API — persis yang
    // aturan ke-3 larang. Diuji dari dua masukan dengan urutan awal berlawanan:
    // hasilnya harus sama.
    const seri = [
      { urutan: 5, judul: "Sama", terbit: 1000, slugSumber: "beta" },
      { urutan: 5, judul: "Sama", terbit: 1000, slugSumber: "alfa" }
    ];

    for (const mode of ["manual", "terbaru"]) {
      assert.deepEqual(
        urutkanArtikel(seri, mode).map((x) => x.slugSumber),
        urutkanArtikel([...seri].reverse(), mode).map((x) => x.slugSumber),
        `cabang ${mode} tidak deterministik pada seri`
      );
    }
  });

  test('"terbaru" memecah seri dengan slug SUMBER, sehingga setiap locale sama urutannya', () => {
    // Judul adalah milik terjemahan; slug sumber identik di setiap bahasa.
    // Memecah seri dengan judul akan menjalankan seksi berita dalam urutan yang
    // berbeda di Bahasa Indonesia dan di English, tanpa satu pun gerbang melihat.
    const id = [
      { urutan: 9, judul: "Zebra", terbit: 2000, slugSumber: "alfa" },
      { urutan: 9, judul: "Apel", terbit: 2000, slugSumber: "beta" }
    ];
    const en = [
      { urutan: 9, judul: "Apple", terbit: 2000, slugSumber: "alfa" },
      { urutan: 9, judul: "Zebra", terbit: 2000, slugSumber: "beta" }
    ];

    assert.deepEqual(
      urutkanArtikel(id, "terbaru").map((x) => x.slugSumber),
      urutkanArtikel(en, "terbaru").map((x) => x.slugSumber)
    );
  });

  test("masukan tidak dimutasi", () => {
    const asli = [...CONTOH];
    urutkanArtikel(CONTOH, "terbaru");
    assert.deepEqual(CONTOH, asli);
  });
});

describe("urutan seksi tersambung ke getArticles", () => {
  // `urutkanArtikel` di atas diuji sebagai fungsi murni, dan itu meninggalkan
  // satu celah: SAMBUNGANNYA. `getArticles` boleh berhenti memanggilnya sama
  // sekali — atau memanggilnya dengan kunci yang salah — dan setiap tes di atas
  // tetap hijau. Blok ini menutup celah itu lewat jalur yang benar-benar
  // dipakai halaman.
  let getArticles;
  let resetContentCacheForTests;

  beforeEach(async () => {
    process.env.AWCMS_API_URL = "http://awcms.uji";
    process.env.AWCMS_API_TOKEN = TOKEN;
    ({ getArticles, resetContentCacheForTests } = await import("../src/lib/content.ts"));
    resetContentCacheForTests();
  });

  afterEach(() => {
    globalThis.fetch = fetchAsli;
  });

  test('tab "manual" benar-benar terurut dari `urutan`, bukan dari urutan respons', async () => {
    // `buatPost(i)` menulis `urutan: i`, jadi feed yang dikembalikan terbalik
    // harus keluar dalam urutan menaik. Tanpa pengurutan sama sekali, hasilnya
    // persis urutan respons — dan itu yang tes ini bunuh.
    const posts = [buatPost(3), buatPost(1), buatPost(2)];
    pasangFetchTiruan(posts);

    const artikel = await getArticles("panduan", "id");
    assert.deepEqual(
      artikel.map((a) => a.entry.data.urutan),
      [1, 2, 3]
    );
  });

  test("tanggal yang DITAMPILKAN kartu ikut menurun di seksi yang diurutkan tanggal", async () => {
    // Kunci urut dan kolom yang dirender kartu harus kolom yang SAMA. Saat
    // keduanya berbeda — diurutkan dari tanggal post SUMBER, ditampilkan dari
    // tanggal TERJEMAHAN — halaman `/en/` menampilkan daftar yang tanggalnya
    // naik-turun tanpa pola, dan tidak satu pun gerbang bisa melihatnya.
    //
    // Diuji lewat `urutkanArtikel` atas kunci yang dirakit `getArticles`
    // sendiri, karena template ini tidak membawa satu pun tab "terbaru".
    const { urutkanArtikel } = await import("../src/lib/content.ts");

    const posts = [
      buatPost(0, { grup: "g0", publishedAt: "2026-08-01T00:00:00.000Z" }),
      buatPost(1, { locale: "en", grup: "g0", publishedAt: "2026-07-10T00:00:00.000Z" }),
      buatPost(2, { grup: "g2", publishedAt: "2026-07-20T00:00:00.000Z" }),
      buatPost(3, { locale: "en", grup: "g2", publishedAt: "2026-08-05T00:00:00.000Z" })
    ];
    pasangFetchTiruan(posts);

    const en = await getArticles("panduan", "en");

    const berkunci = en.map((a) => ({
      urutan: a.entry.data.urutan,
      judul: a.entry.data.title,
      terbit: a.entry.data.publishedDate.getTime(),
      slugSumber: a.slug
    }));

    const tampil = urutkanArtikel(berkunci, "terbaru").map((x) => x.terbit);

    for (let i = 1; i < tampil.length; i += 1) {
      assert.ok(
        tampil[i - 1] >= tampil[i],
        `tanggal kartu naik pada posisi ${i}: ${new Date(tampil[i - 1]).toISOString()} lalu ${new Date(tampil[i]).toISOString()}`
      );
    }
  });

  test("feed yang tidak memuat satu pun post publik MENGGAGALKAN build", async () => {
    const posts = [buatPost(0, { status: "draft" }), buatPost(1, { visibility: "private" })];
    pasangFetchTiruan(posts);

    await assert.rejects(
      () => getArticles("panduan", "id"),
      (e) => /NOT ONE of them is both published and public/.test(e.message)
    );
  });

  test("feed seksi dirakit dari kolom awcms yang benar (ADR-0035)", async () => {
    // `bangunFeedAtom` diuji sebagai fungsi murni di `tests/feed.test.mjs`, dan
    // itu meninggalkan celah yang persis sama dengan yang blok ini tutup untuk
    // `urutkanArtikel`: SAMBUNGANNYA. `isiFeed` boleh mengirim URL relatif,
    // mengirim `updatedAt` sebagai tanggal terbit, atau mengirim `title` mentah
    // alih-alih `description` — dan setiap tes di berkas itu tetap hijau.
    //
    // Dijalankan atas tab "panduan" karena template ini tidak membawa satu pun
    // tab "terbaru"; yang diuji di sini perakitannya, bukan syarat terbitnya —
    // syarat itu milik `seksiPunyaFeed`, yang diuji terpisah.
    const { isiFeed } = await import("../src/lib/feed-seksi.ts");

    pasangFetchTiruan([
      buatPost(1, {
        publishedAt: "2026-08-01T03:00:00.000Z",
        updatedAt: "2026-08-04T09:00:00.000Z"
      })
    ]);

    const xml = await isiFeed("id", "panduan");

    assert.match(xml, /<feed xmlns="http:\/\/www\.w3\.org\/2005\/Atom"/);
    // URL absolut, diturunkan dari SITE_URL — bukan path situs.
    assert.match(xml, /<id>https?:\/\/[^<]*\/panduan\/artikel-1\/<\/id>/);
    // Kedua stempel dibawa TERPISAH, yang merupakan ADR-0033 dilihat dari sini.
    assert.ok(xml.includes("<published>2026-08-01T03:00:00.000Z</published>"));
    assert.ok(xml.includes("<updated>2026-08-04T09:00:00.000Z</updated>"));
    // Ringkasan datang dari `metaDescription`, bukan dari judul.
    assert.ok(xml.includes('<summary type="text">Meta 1</summary>'));
  });

  test("rute feed benar-benar mengeluarkan feed itu, dengan tipe isinya", async () => {
    // `isiFeed` diuji di atas, dan itu masih meninggalkan enam baris perekat
    // yang tidak pernah dieksekusi di repo ini: `getStaticPaths` dan `GET` pada
    // kedua rute. Keduanya hanya berjalan saat `astro build` menemukan seksi
    // berita — yang tidak pernah terjadi di sini. Dipanggil langsung.
    const akar = await import("../src/pages/[tab]/feed.xml.ts");
    const berprefiks = await import("../src/pages/[lang]/[tab]/feed.xml.ts");

    pasangFetchTiruan([buatPost(1)]);

    // Ketiga tab template `"manual"`, jadi kedua rute menerbitkan nol berkas.
    assert.deepEqual(await akar.getStaticPaths(), []);
    assert.deepEqual(await berprefiks.getStaticPaths(), []);

    const respons = await akar.GET({ props: { tab: "panduan" } });
    assert.equal(respons.headers.get("content-type"), "application/atom+xml; charset=utf-8");
    assert.match(await respons.text(), /<feed xmlns="http:\/\/www\.w3\.org\/2005\/Atom"/);

    const responsEn = await berprefiks.GET({ props: { lang: "en", tab: "panduan" } });
    assert.match(await responsEn.text(), /xml:lang="en-US"/);
  });

  test("template ini menerbitkan NOL feed, dan itu keadaan yang ditegakkan", async () => {
    // Ketiga tab template `"manual"`. Yang dijaga di sini bukan kekosongannya
    // melainkan bahwa kedua rute feed membaca daftar yang SAMA: sebuah tab yang
    // kelak dinyatakan `"terbaru"` harus menerbitkan feed di seluruh locale
    // sekaligus, atau tidak sama sekali.
    const { daftarFeed } = await import("../src/lib/feed-seksi.ts");

    pasangFetchTiruan([buatPost(1)]);

    assert.deepEqual(await daftarFeed(), []);
  });

  test("gerbang view=full berbicara lebih dulu daripada lantai tanggal", async () => {
    // Urutan operasi yang ADR-0033 sebut dikunci, dikunci di sini. Bila
    // assertion `view=full` dipindah ke SESUDAH penyaring tanggal, ia akan
    // lolos hampa atas daftar kosong dan pesan yang muncul menjadi pesan
    // lantai — menyalahkan tanggal untuk awcms yang menjawab ringkasan.
    const posts = [buatPost(0, { publishedAt: null }), buatPost(1, { publishedAt: null })];
    globalThis.fetch = async () =>
      Response.json({
        success: true,
        data: { posts: posts.map(ringkas), nextCursor: null }
      });

    await assert.rejects(
      () => getArticles("panduan", "id"),
      (e) => /ignored \?view=full/.test(e.message)
    );
  });
});

describe("urutanSeksiTab", () => {
  test("slug yang tidak menamai tab mana pun jatuh ke manual", async () => {
    // Ini yang dijadikan alasan bahwa sebuah artikel tidak pernah diam-diam
    // menjadi NewsArticle saat tab-nya diganti nama atau dihapus.
    const { urutanSeksiTab } = await import("../src/config/site.ts");

    assert.equal(urutanSeksiTab("tab-yang-tidak-pernah-ada"), "manual");
    assert.equal(urutanSeksiTab(""), "manual");
    assert.equal(urutanSeksiTab("panduan"), "manual");
  });
});

// ---------------------------------------------------------------------------
// Gambar artikel dari media awcms.
//
// Kelas cacat yang dijaga di sini sama dengan yang membuat berkas ini ada:
// keluaran yang SALAH tanpa satu pun kegagalan. Sebuah build yang kehilangan
// seluruh gambarnya — token tanpa `media.read`, awcms yang lebih tua, media
// yang tidak dikonfigurasi — menerbitkan situs yang tampak sengaja tanpa
// ilustrasi, dan tidak ada yang bisa membedakannya dari situs yang memang
// begitu.
// ---------------------------------------------------------------------------

const MEDIA_ID = "aaaaaaaa-bbbb-4ccc-8ddd-eeeeeeeeeeee";

function buatObjek({ altText = "Foto loket layanan", width = 1600, height = 900 } = {}) {
  return {
    publicUrl: "https://media.contoh.test/news/foto.webp",
    altText,
    mimeType: "image/webp",
    width,
    height
  };
}

describe("gambar artikel dari media awcms", () => {
  let getArticles;
  let resetContentCacheForTests;

  beforeEach(async () => {
    process.env.AWCMS_API_URL = "http://awcms.uji";
    process.env.AWCMS_API_TOKEN = TOKEN;
    ({ getArticles, resetContentCacheForTests } = await import("../src/lib/content.ts"));
    resetContentCacheForTests();
  });

  afterEach(() => {
    globalThis.fetch = fetchAsli;
  });

  test("featuredMediaId menjadi gambar artikel, dengan altText dari awcms", async () => {
    const posts = [{ ...buatPost(0), featuredMediaId: MEDIA_ID }];
    pasangFetchTiruan(posts, { media: new Map([[MEDIA_ID, buatObjek()]]) });

    const [artikel] = await getArticles("panduan", "id");

    assert.equal(artikel.gambar.src, "https://media.contoh.test/news/foto.webp");
    assert.equal(artikel.gambar.alt, "Foto loket layanan");
    assert.equal(artikel.gambar.width, 1600);
    assert.equal(artikel.gambar.height, 900);
  });

  test("altText kosong jatuh ke judul artikel, bukan ke alt kosong", async () => {
    // `alt=""` memberi tahu pembaca layar bahwa gambarnya dekoratif. Gambar
    // kepala artikel bukan dekorasi, jadi kosong adalah jawaban yang salah —
    // judulnya setidaknya dalam bahasa yang sedang dibaca.
    const posts = [{ ...buatPost(0), featuredMediaId: MEDIA_ID }];
    pasangFetchTiruan(posts, {
      media: new Map([[MEDIA_ID, buatObjek({ altText: "   " })]])
    });

    const [artikel] = await getArticles("panduan", "id");
    assert.equal(artikel.gambar.alt, "Artikel 0");
  });

  test("artikel tanpa featuredMediaId TIDAK memicu satu permintaan media pun", async () => {
    const jejak = [];
    pasangFetchTiruan([buatPost(0)], { jejak });

    await getArticles("panduan", "id");

    assert.equal(jejak.filter((j) => j.startsWith("/api/v1/media/")).length, 0);
  });

  test("satu id yang hilang menjadi placeholder, sisanya tetap terbit", async () => {
    // awcms mengizinkan objek di-purge dan memutuskan rujukan yang menggantung
    // menjadi inert, bukan penghalang (ADR-0056 §B). Menggagalkan build di sini
    // berarti situs tidak bisa terbit karena satu gambar dihapus.
    const hilang = "ffffffff-bbbb-4ccc-8ddd-eeeeeeeeeeee";
    const posts = [
      { ...buatPost(0), featuredMediaId: MEDIA_ID },
      { ...buatPost(1), featuredMediaId: hilang }
    ];
    pasangFetchTiruan(posts, { media: new Map([[MEDIA_ID, buatObjek()]]) });

    const artikel = await getArticles("panduan", "id");

    assert.equal(artikel.length, 2);
    assert.ok(artikel[0].gambar);
    assert.equal(artikel[1].gambar, undefined);
  });

  test("SELURUH id yang hilang MENGGAGALKAN build", async () => {
    // Bentuk yang sama dengan cacat ADR-0018: build hijau, situs terbit, dan
    // setiap artikel kehilangan gambarnya sekaligus tanpa ada yang menyebutnya.
    const posts = [
      { ...buatPost(0), featuredMediaId: MEDIA_ID },
      { ...buatPost(1), featuredMediaId: "ffffffff-bbbb-4ccc-8ddd-eeeeeeeeeeee" }
    ];
    pasangFetchTiruan(posts, { media: new Map() });

    await assert.rejects(
      () => getArticles("panduan", "id"),
      (e) => /resolved NONE/.test(e.message)
    );
  });

  test("lebih dari 100 id dipecah menjadi beberapa permintaan, tanpa kehilangan satu pun", async () => {
    // `ids` dibatasi 100 di awcms dan kelebihannya dijawab 400 — bukan
    // dipotong. Tanpa pemecahan, situs dengan 150 artikel bergambar gagal
    // build; dengan pemecahan yang salah, sebagian gambar hilang diam-diam.
    const jejak = [];
    const media = new Map();
    const posts = Array.from({ length: 150 }, (_, i) => {
      const id = `aaaaaaaa-bbbb-4ccc-8ddd-${String(i).padStart(12, "0")}`;
      media.set(id, buatObjek());
      return { ...buatPost(i), featuredMediaId: id };
    });
    pasangFetchTiruan(posts, { media, jejak });

    const artikel = await getArticles("panduan", "id");

    assert.equal(artikel.length, 150);
    assert.equal(artikel.filter((a) => a.gambar).length, 150);

    const permintaanMedia = jejak.filter((j) => j.startsWith("/api/v1/media/objects"));
    assert.equal(permintaanMedia.length, 2);
    for (const permintaan of permintaanMedia) {
      const ids = new URL(`http://x${permintaan}`).searchParams.get("ids").split(",");
      assert.ok(ids.length <= 100, `satu permintaan membawa ${ids.length} id`);
    }
  });

  test("media di-resolve SEKALI per build, bukan sekali per tab atau per locale", async () => {
    const jejak = [];
    const posts = [
      { ...buatPost(0), featuredMediaId: MEDIA_ID, grup: "grup-1" },
      { ...buatPost(1, { locale: "en", grup: "grup-1" }), featuredMediaId: MEDIA_ID }
    ];
    pasangFetchTiruan(posts, { media: new Map([[MEDIA_ID, buatObjek()]]), jejak });

    await getArticles("panduan", "id");
    await getArticles("panduan", "en");

    assert.equal(jejak.filter((j) => j.startsWith("/api/v1/media/objects")).length, 1);
  });
});

// ---------------------------------------------------------------------------
// Kartu share per artikel.
//
// awcms menyatakan `seoImageMediaId` MENGALAHKAN `featuredMediaId` untuk
// pratinjau sosial, dan `seo-facts-port-adapter.ts` miliknya menyelesaikan
// persis `seo_image_media_id ?? featured_media_id`. Urutan itu dicerminkan di
// sini, bukan ditemukan ulang: situs yang kartunya berbeda dari permukaan SEO
// CMS-nya sendiri adalah dua jawaban untuk satu pertanyaan, dan hanya satu yang
// terlihat editor.
// ---------------------------------------------------------------------------

const KARTU_ID = "bbbbbbbb-cccc-4ddd-8eee-ffffffffffff";

describe("kartu share per artikel", () => {
  let getArticles;
  let resetContentCacheForTests;

  beforeEach(async () => {
    process.env.AWCMS_API_URL = "http://awcms.uji";
    process.env.AWCMS_API_TOKEN = TOKEN;
    ({ getArticles, resetContentCacheForTests } = await import("../src/lib/content.ts"));
    resetContentCacheForTests();
  });

  afterEach(() => {
    globalThis.fetch = fetchAsli;
  });

  test("seoImageMediaId mengalahkan featuredMediaId — urutan milik awcms", async () => {
    const posts = [{ ...buatPost(0), featuredMediaId: MEDIA_ID, seoImageMediaId: KARTU_ID }];
    pasangFetchTiruan(posts, {
      media: new Map([
        [MEDIA_ID, buatObjek()],
        [
          KARTU_ID,
          {
            publicUrl: "https://media.contoh.test/news/kartu.png",
            altText: "Kartu berbagi",
            mimeType: "image/png",
            width: 1200,
            height: 630
          }
        ]
      ])
    });

    const [artikel] = await getArticles("panduan", "id");

    assert.equal(artikel.kartuShare.src, "https://media.contoh.test/news/kartu.png");
    // Gambar HALAMAN tetap featuredMediaId: yang diprioritaskan awcms hanya
    // permukaan pratinjau, bukan ilustrasi di badan artikel.
    assert.equal(artikel.gambar.src, "https://media.contoh.test/news/foto.webp");
  });

  test("tanpa seoImageMediaId, featuredMediaId menjadi kartunya", async () => {
    const posts = [{ ...buatPost(0), featuredMediaId: MEDIA_ID }];
    pasangFetchTiruan(posts, { media: new Map([[MEDIA_ID, buatObjek()]]) });

    const [artikel] = await getArticles("panduan", "id");
    assert.equal(artikel.kartuShare.src, artikel.gambar.src);
  });

  test("kartu membawa MIME dan ukurannya sendiri, bukan konstanta kartu situs", async () => {
    // Inilah cacat yang menunggu bila keduanya tidak ikut: setiap halaman
    // artikel memasang `image/png` 1200×630 untuk berkas WebP 1600×900, dan
    // pengunduh pratinjau yang memercayainya melebarkan ke kotak yang salah
    // atau menolak kartunya — tanpa satu pun kegagalan di build.
    const posts = [{ ...buatPost(0), featuredMediaId: MEDIA_ID }];
    pasangFetchTiruan(posts, { media: new Map([[MEDIA_ID, buatObjek()]]) });

    const [artikel] = await getArticles("panduan", "id");

    assert.equal(artikel.kartuShare.type, "image/webp");
    assert.equal(artikel.kartuShare.width, 1600);
    assert.equal(artikel.kartuShare.height, 900);
  });

  test("artikel tanpa gambar apa pun tidak punya kartu — dan itu didukung", async () => {
    pasangFetchTiruan([buatPost(0)]);

    const [artikel] = await getArticles("panduan", "id");
    assert.equal(artikel.kartuShare, undefined);
  });

  test("kedua id diminta dalam SATU batch, dan yang sama tidak diminta dua kali", async () => {
    const jejak = [];
    const posts = [
      { ...buatPost(0), featuredMediaId: MEDIA_ID, seoImageMediaId: KARTU_ID },
      { ...buatPost(1), featuredMediaId: MEDIA_ID }
    ];
    pasangFetchTiruan(posts, {
      media: new Map([
        [MEDIA_ID, buatObjek()],
        [KARTU_ID, { ...buatObjek(), publicUrl: "https://media.contoh.test/news/kartu.png" }]
      ]),
      jejak
    });

    await getArticles("panduan", "id");

    const permintaan = jejak.filter((j) => j.startsWith("/api/v1/media/objects"));
    assert.equal(permintaan.length, 1);
    const ids = new URL(`http://x${permintaan[0]}`).searchParams.get("ids").split(",");
    assert.deepEqual([...new Set(ids)].sort(), ids.sort());
    assert.equal(ids.length, 2);
  });
});

// ---------------------------------------------------------------------------
// Kartu situs sebagai objek utuh.
//
// Yang dijaga di sini bukan nilainya melainkan BENTUKNYA: selama `src` dan
// `alt` adalah dua nilai terpisah, keduanya bisa datang dari dua gambar
// berbeda — dan itu yang terjadi sebelum ini, di halaman seksi maupun halaman
// artikel. Tipe `KartuShare` yang membuatnya berhenti bisa ditulis; tes ini
// menjaga bawaannya tetap benar.
// ---------------------------------------------------------------------------

describe("kartu situs", () => {
  test("membawa MIME dan ukuran yang dikontrakkan .env.example", async () => {
    const { kartuSitus, SOCIAL_IMAGE_WIDTH, SOCIAL_IMAGE_HEIGHT } = await import(
      "../src/lib/social-image.ts"
    );
    const kartu = kartuSitus("Kartu berbagi situs");

    // Template ini tidak mengisi SITE_SOCIAL_IMAGE, jadi bawaannya `undefined`
    // — dan itu keadaan yang DIDUKUNG: halaman tidak memasang tag gambar sama
    // sekali dan pratinjau jatuh ke kartu teks.
    if (kartu === undefined) {
      assert.equal(SOCIAL_IMAGE_WIDTH, 1200);
      assert.equal(SOCIAL_IMAGE_HEIGHT, 630);
      return;
    }

    assert.equal(kartu.type, "image/png");
    assert.equal(kartu.width, SOCIAL_IMAGE_WIDTH);
    assert.equal(kartu.height, SOCIAL_IMAGE_HEIGHT);
    assert.equal(kartu.alt, "Kartu berbagi situs");
  });
});

/**
 * Seksi sebuah artikel: dari mana ia datang, dan apa yang terjadi bila tidak
 * datang dari mana pun.
 *
 * Sampai perbaikan ini, seksi HANYA dibaca dari `contentJson.awcmsAstro.kategori`
 * — sidecar milik repo ini yang jalur authoring `awcms` tidak pernah menulisnya.
 * Setiap artikel yang ditulis editor di layar admin `awcms` tidak pernah terbit:
 * bukan salah render, tidak dibangun sama sekali, dengan build hijau.
 *
 * Suite ini tidak bisa melihatnya karena `buatPost` menulis sidecar pada SETIAP
 * baris fixture — sehingga satu-satunya bentuk yang gagal di produksi adalah
 * satu-satunya bentuk yang tidak pernah dihasilkan double-nya.
 */
describe("seksi artikel datang dari taksonomi, bukan hanya dari sidecar", () => {
  let getArticles;
  let resetContentCacheForTests;

  const TERM_PANDUAN = {
    id: "aaaaaaaa-0000-4000-8000-000000000001",
    taxonomyType: "category",
    name: "Panduan",
    slug: "panduan"
  };
  const TERM_ASING = {
    id: "aaaaaaaa-0000-4000-8000-000000000002",
    taxonomyType: "category",
    name: "Tak Dipetakan",
    slug: "tak-dipetakan"
  };

  beforeEach(async () => {
    process.env.AWCMS_API_URL = "http://awcms.uji";
    process.env.AWCMS_API_TOKEN = TOKEN;
    delete process.env.AWCMS_TENANT_CODE;
    delete process.env.AWCMS_DEFAULT_TENANT_CODE;

    ({ getArticles, resetContentCacheForTests } = await import("../src/lib/content.ts"));
    resetContentCacheForTests();
  });

  afterEach(() => {
    globalThis.fetch = fetchAsli;
  });

  test("artikel TANPA sidecar, hanya dengan term awcms, akhirnya terbit", async () => {
    // Inilah bentuk yang dihasilkan layar admin `awcms`, dan inilah kasus yang
    // sebelumnya membangun NOL halaman.
    const posts = [buatPost(0, { kategori: null, termIds: [TERM_PANDUAN.id] })];
    pasangFetchTiruan(posts, { term: [TERM_PANDUAN] });

    const artikel = await getArticles("panduan", "id");

    assert.equal(artikel.length, 1);
    assert.equal(artikel[0].slug, "artikel-0");
    // Seksinya harus ikut sampai ke kontrak komponen: `kategori` kosong akan
    // membuat breadcrumb tidak menyebut apa pun dan `urutanSeksiTab("")`
    // menjawab "manual", sehingga seksi berita diam-diam merender sebagai
    // seksi rujukan.
    assert.equal(artikel[0].entry.data.kategori, "panduan");
  });

  test("artikel DENGAN sidecar tetap berperilaku persis seperti sebelumnya", async () => {
    const posts = [buatPost(0)];
    pasangFetchTiruan(posts, { term: [TERM_PANDUAN] });

    const artikel = await getArticles("panduan", "id");

    assert.equal(artikel.length, 1);
    assert.equal(artikel[0].entry.data.kategori, "panduan");
  });

  test("sidecar MENANG atas term yang tidak sepakat dengannya", async () => {
    // `awcms` ADR-0115 §4 MENOLAK mengimpor baris yang tak bisa ditempatkan
    // `--section-map`-nya. Membiarkan taksonomi menimpa instruksi eksplisit itu
    // akan membuat sebuah migrasi mendarat di tempat yang bukan pilihan
    // operatornya.
    const posts = [
      buatPost(0, { kategori: "panduan", termIds: [TERM_ASING.id] })
    ];
    pasangFetchTiruan(posts, { term: [TERM_PANDUAN, TERM_ASING] });

    assert.equal((await getArticles("panduan", "id")).length, 1);
  });

  test("post yang tak tertempatkan DISEBUT namanya, dan build tetap lanjut", async () => {
    const peringatan = [];
    const warnAsli = console.warn;
    console.warn = (...args) => peringatan.push(args.join(" "));

    try {
      const posts = [
        buatPost(0, { kategori: null, termIds: [TERM_PANDUAN.id] }),
        buatPost(1, { kategori: null, termIds: [TERM_ASING.id] })
      ];
      pasangFetchTiruan(posts, { term: [TERM_PANDUAN, TERM_ASING] });

      const artikel = await getArticles("panduan", "id");

      // Satu artikel salah tempat tidak boleh menghentikan seluruh redaksi
      // menerbitkan.
      assert.equal(artikel.length, 1);

      const teks = peringatan.join("\n");
      assert.match(teks, /artikel-1/);
      assert.match(teks, /tak-dipetakan/);
      // Yang hilang harus terbaca sebagai HALAMAN yang tidak dibangun, bukan
      // sebagai catatan kecil tentang sebuah field.
      assert.match(teks, /no page is built/i);
    } finally {
      console.warn = warnAsli;
    }
  });

  test("NOL dari N tertempatkan menggagalkan build, alih-alih menerbitkan situs kosong", async () => {
    // Bukan kesalahan tingkat artikel. Penyebab yang lazim adalah `termSlugs`
    // menyebut kosakata yang tidak dipakai tenant ini, kredensial build tanpa
    // `blog_content.taxonomies.read`, atau tab yang diganti nama sementara
    // `site.ts` tidak. Ketiganya menerbitkan SITUS KOSONG dari build hijau.
    const posts = [
      buatPost(0, { kategori: null, termIds: [TERM_ASING.id] }),
      buatPost(1, { kategori: null, termIds: [TERM_ASING.id] })
    ];
    pasangFetchTiruan(posts, { term: [TERM_ASING] });

    await assert.rejects(
      () => getArticles("panduan", "id"),
      /All 2 published post\(s\) belong to no section/
    );
  });

  test("kosakata KOSONG bukan kegagalan — situs yang hanya memakai sidecar tetap terbangun", async () => {
    // Sebuah tenant yang tidak memakai kategori sama sekali adalah keadaan yang
    // sah, dan `taksonomi.ts` memperingatkan lalu mengembalikan `[]` pada 403
    // atau 404. Situs yang menempatkan artikelnya lewat sidecar harus tetap
    // bekerja persis seperti sebelum penempatan taksonomi ada.
    const posts = [buatPost(0)];
    pasangFetchTiruan(posts, { term: [] });

    assert.equal((await getArticles("panduan", "id")).length, 1);
  });

  test("respons /blog/terms tanpa field `terms` gagal dengan pesan yang menyebut endpoint-nya", async () => {
    // Sebelumnya ini meledak sebagai `Spread syntax requires ...iterable` dari
    // dalam `taksonomi.ts` — pesan yang tidak menyebut endpoint, tenant, maupun
    // apa yang harus diperbaiki.
    const posts = [buatPost(0)];
    pasangFetchTiruan(posts);
    const tiruan = globalThis.fetch;
    globalThis.fetch = async (input) => {
      if (String(input).includes("/api/v1/blog/terms")) {
        return Response.json({ success: true, data: { nextCursor: null } });
      }
      return tiruan(input);
    };

    await assert.rejects(
      () => getArticles("panduan", "id"),
      /GET \/api\/v1\/blog\/terms answered without a "terms" array/
    );
  });
});

/**
 * Badan artikel: dari kolom KANONIK, dengan proyeksi sebagai jatuhan.
 *
 * `awcms` ADR-0100 menjadikan Portable Text badan kanonik pada 19 Agustus 2026
 * dan mengapalkannya di v10.0.0. Union enam-tipe yang lama bertahan di sana
 * sebagai `content_json.blocks` — proyeksi TURUNAN yang sengaja dipertahankan
 * supaya repo ini tidak menjadi kosong pada hari cutover, dan **lossy secara
 * konstruksi**: kosakata lama tidak punya mark, jadi setiap tebal, miring,
 * kode, dan tautan dalam kalimat merata menjadi teks polos saat menyeberang.
 *
 * Proyeksi itulah yang dirender situs ini sampai sekarang. Setiap artikel yang
 * pernah diterbitkannya adalah prosa tanpa format, dan tidak ada editor yang
 * bisa mengubahnya.
 */
describe("badan artikel datang dari kolom kanonik", () => {
  let getArticles;
  let resetContentCacheForTests;

  const BADAN = [
    {
      _type: "block",
      _key: "b1",
      style: "normal",
      children: [
        { _type: "span", _key: "s1", text: "Angka ", marks: [] },
        { _type: "span", _key: "s2", text: "penting", marks: ["strong"] },
        { _type: "span", _key: "s3", text: " ada di ", marks: [] },
        { _type: "span", _key: "s4", text: "aturannya", marks: ["k1"] }
      ],
      markDefs: [{ _type: "link", _key: "k1", href: "https://contoh.test/aturan" }]
    }
  ];

  beforeEach(async () => {
    process.env.AWCMS_API_URL = "http://awcms.uji";
    process.env.AWCMS_API_TOKEN = TOKEN;
    delete process.env.AWCMS_TENANT_CODE;
    delete process.env.AWCMS_DEFAULT_TENANT_CODE;

    ({ getArticles, resetContentCacheForTests } = await import("../src/lib/content.ts"));
    resetContentCacheForTests();
  });

  afterEach(() => {
    globalThis.fetch = fetchAsli;
  });

  test("mark inline akhirnya sampai ke pembaca", async () => {
    const post = buatPost(0);
    post.bodyPortableText = BADAN;
    post.contentJson.blocks = [
      { type: "paragraph", text: "Angka penting ada di aturannya" }
    ];

    pasangFetchTiruan([post]);
    const [artikel] = await getArticles("panduan", "id");

    // Persis yang HILANG di proyeksi: proyeksinya membawa kata-kata yang sama
    // dan tidak satu pun mark-nya.
    assert.match(artikel.entry.bodyHtml, /<strong>penting<\/strong>/);
    assert.match(
      artikel.entry.bodyHtml,
      /<a href="https:\/\/contoh\.test\/aturan" rel="noopener noreferrer">aturannya<\/a>/
    );
  });

  test("baris yang belum di-backfill tetap terbaca lewat proyeksinya", async () => {
    // `bodyPortableText` tiba KOSONG dari baris yang belum disentuh
    // `blog:portable-text:backfill`. Menghapus jatuhan ini akan menerbitkan
    // artikel kosong untuk persis baris yang belum dimigrasikan.
    const post = buatPost(0);
    post.bodyPortableText = [];
    post.contentJson.blocks = [{ type: "paragraph", text: "Prosa lama" }];

    pasangFetchTiruan([post]);
    const [artikel] = await getArticles("panduan", "id");

    assert.match(artikel.entry.bodyHtml, /Prosa lama/);
  });

  test("awcms yang MENDAHULUI ADR-0100 tidak mengirim field ini sama sekali", async () => {
    // Absen dan kosong adalah dua hal berbeda, dan keduanya terjadi. Yang satu
    // berarti "awcms ini tidak punya kolomnya", yang lain "baris ini belum
    // di-backfill" — keduanya mengambil cabang yang sama karena kata-katanya
    // sama-sama ada di proyeksi.
    const post = buatPost(0);
    post.contentJson.blocks = [{ type: "paragraph", text: "Prosa lama" }];
    assert.equal(post.bodyPortableText, undefined);

    pasangFetchTiruan([post]);
    const [artikel] = await getArticles("panduan", "id");

    assert.match(artikel.entry.bodyHtml, /Prosa lama/);
  });

  test("kolom kanonik MENANG saat keduanya ada, bukan digabung", async () => {
    const post = buatPost(0);
    post.bodyPortableText = BADAN;
    post.contentJson.blocks = [{ type: "paragraph", text: "PROYEKSI BASI" }];

    pasangFetchTiruan([post]);
    const [artikel] = await getArticles("panduan", "id");

    assert.doesNotMatch(artikel.entry.bodyHtml, /PROYEKSI BASI/);
  });

  test("galeri di badan kanonik ikut terselesaikan di batch media yang SAMA", async () => {
    // Mengumpulkan id hanya dari proyeksi akan menyelesaikan setiap galeri milik
    // baris yang belum di-backfill dan tidak satu pun milik baris yang sudah —
    // situs yang galerinya bekerja sampai hari kontennya dimigrasikan.
    const post = buatPost(0);
    post.bodyPortableText = [
      { _type: "gallery", _key: "g1", items: [{ mediaObjectId: "media-kanonik" }] }
    ];

    const media = new Map([
      [
        "media-kanonik",
        {
          publicUrl: "https://media.uji/kanonik.webp",
          altText: "Foto kanonik",
          width: 800,
          height: 450
        }
      ]
    ]);

    pasangFetchTiruan([post], { media });
    const [artikel] = await getArticles("panduan", "id");

    assert.match(artikel.entry.bodyHtml, /media\.uji\/kanonik\.webp/);
  });
});

/**
 * Pengaman putaran liar, dan angka yang diukur alih-alih diasumsikan.
 *
 * Plafonnya dulu 400 halaman — 20.000 post — dengan komentar bahwa ia "duduk
 * jauh di atas situs mana pun yang masuk akal". Angka itu tebakan, dan pada 26
 * Agustus 2026 ia berhenti benar: `awcms` mengukur arsip rujukan keluarga ini
 * dan mendapat 25.029 artikel. Plafonnya ada DI BAWAH korpus yang sudah diukur
 * keluarga ini sendiri, sehingga kegagalannya — yang jujur, karena ia MELEMPAR
 * alih-alih memotong diam-diam — menyala pada situs yang sekadar besar.
 */
describe("plafon skala build", () => {
  let getArticles;
  let resetContentCacheForTests;

  beforeEach(async () => {
    process.env.AWCMS_API_URL = "http://awcms.uji";
    process.env.AWCMS_API_TOKEN = TOKEN;
    delete process.env.AWCMS_TENANT_CODE;
    delete process.env.AWCMS_DEFAULT_TENANT_CODE;

    ({ getArticles, resetContentCacheForTests } = await import("../src/lib/content.ts"));
    resetContentCacheForTests();
  });

  afterEach(() => {
    globalThis.fetch = fetchAsli;
  });

  test("korpus 25.029 artikel milik keluarga ini muat di bawah plafonnya", async () => {
    // Angka yang persis. Ini bukan uji beban — fixture-nya tidak membangun
    // 25.029 post — melainkan penegasan bahwa PLAFONNYA berada di atas korpus
    // yang sudah diukur, yang justru hal yang salah selama ini.
    const { MAX_PAGES_UNTUK_UJI, PAGE_SIZE_UNTUK_UJI } = await import(
      "../src/lib/content.ts"
    );

    assert.ok(
      MAX_PAGES_UNTUK_UJI * PAGE_SIZE_UNTUK_UJI > 25029,
      `plafon ${MAX_PAGES_UNTUK_UJI * PAGE_SIZE_UNTUK_UJI} post masih di bawah ` +
        `korpus 25.029 artikel yang diukur awcms ADR-0114`
    );
  });

  test("cursor yang tidak maju MELEMPAR, bukan menerbitkan daftar pendek", async () => {
    // Yang paling berbahaya bukan situs besar melainkan cursor yang macet:
    // awcms mengembalikan halaman yang sama selamanya, dan mengembalikan apa
    // yang sudah terkumpul akan menerbitkan situs yang kehilangan artikel tanpa
    // ada yang menghitungnya.
    const posts = Array.from({ length: 50 }, (_, i) => buatPost(i));

    globalThis.fetch = async (input) => {
      const url = new URL(String(input));

      if (url.pathname === "/api/v1/blog/terms") {
        return Response.json({ success: true, data: { terms: [], nextCursor: null } });
      }

      // Selalu halaman yang sama, selalu dengan cursor. Persis bentuk cursor
      // yang macet.
      return Response.json({
        success: true,
        data: { posts, nextCursor: "cursor-yang-tidak-pernah-maju" }
      });
    };

    await assert.rejects(
      () => getArticles("panduan", "id"),
      (galat) => {
        assert.match(galat.message, /still returned a cursor/);
        // Pesannya harus menyebut KEDUA sebabnya dan alat untuk memilih di
        // antaranya. Pesan lama hanya menyebut keduanya ada.
        assert.match(galat.message, /cursor is not advancing/);
        assert.match(galat.message, /ukur:skala/);
        // Dan harus menolak "kembalikan yang sudah ada" secara eksplisit.
        assert.match(galat.message, /short list that looks complete/);
        return true;
      }
    );
  });
});
