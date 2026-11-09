# update-check — piano

1. `bezel-core/src/domain/update.rs`: `Version`, `newer`, `due`, `CHECK_EVERY`, `LATEST_RELEASE_API`, `RELEASES_PAGE`.
2. `bezel-runtime/src/update.rs`: `Record`, `read`, `write`, `available`.
3. `bezel-cli/src/update_check.rs` (ureq): `checking`, `step`, `fetch`, `spawn`, `open_releases`; voce nel menu in `light_tray.rs`.
4. Studio: `settings.update_check`, `PreferencesDto.update_check`/`update_available`, comando `set_update_check` (build.rs, capability), link `releases`; bridge, demo, `preferences.js`, i18n.
5. Test: unità Rust e JS, e2e `update-check.spec.mjs`.
