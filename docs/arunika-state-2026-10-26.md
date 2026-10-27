# Arunika — State Tersimpan (26 Okt 2026)

> Cadangan dokumentasi sebelum uninstall/reinstall. Data aplikasi utama
> tetap ada di `%APPDATA%\Arunika\` (uninstall sengaja TIDAK menghapusnya) —
> file ini pengaman + catatan keputusan desain terakhir. Salinan identik
> juga ada di Obsidian Vault (`50-Projects/Arunika — State Tersimpan 2026-10-26.md`).

## Settings aktif (settings.json)

```json
{ "theme": "classic", "idle_style": "eyes", "use_24h": true }
```

## Keputusan desain terakhir

- **Island idle:** animasi mata biru + nebula — **DIBUANG (tidak disukai)**.
  Akan diganti **animasi HANTU** (user sedang membuat, GIF + MP4; timbalangit
  memilih salah satu). Infrastruktur sprite siap: decoder PNG di
  `crates/poni/src/png.rs`, slot 168×44 (`EYES_W`/`EYES_H` di activities.rs),
  interaksi hover→jam / klik→tools dipertahankan.
- Sementara ini look "Eyes" = pill polos (label UI "Eyes" belum diganti,
  akan jadi "Ghost" saat aset hantu terpasang).
- Ukuran island: 168×44 (besar) — dipertahankan.

## Status repo & infrastruktur

- Branch `rebrand/arunika`; CI hijau beruntun (lihat Actions).
- Endpoint: feed `arunika.idewara.com/latest.json`, API `api.arunika.idewara.com`,
  drop `drop.arunika.idewara.com` (placeholder domain final user).
- Kunci update ECDSA P-256: id `0684111eef2eb31c`; private key di luar repo +
  secret GitHub `UPDATE_FEED_KEY` (environment `release`).
- Add-on pre-release: `addon-sholat-1`, `addon-ja-1` (hash pin cocok).
- Backend: repo private `Idewara/arunika-cloud` (Node+Hono+Postgres, 29/29 test),
  belum deploy — menunggu DNS + VPS user.
- Toolchain lokal: GNU jadi default, `rustc` jalan; EDR ManageEngine masih
  memblokir eksekusi exe hasil build (build script gagal) → build penuh tetap
  via CI. Draft tiket whitelist EDR tersedia di vault.

## Langkah berikutnya

1. Terima aset hantu dari user → ekstrak frame (background di-key transparan
   bila perlu) → pasang di pipeline sprite.
2. CI hijau → install → verifikasi visual.
3. Merge `rebrand/arunika` → `main`; rilis v1.0.0 (tulis `docs/release-notes/v1.0.0.md`).
4. Deploy server (Fase 5) saat DNS + VPS tersedia.
