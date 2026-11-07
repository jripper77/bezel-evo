# Phase 22: Review  (slug: display-idle-saver)

**Verdict:** APPROVED_PENDING_MANUAL

Seconda verifica del 7 novembre 2026, branch `windows-improvements`, HEAD `b6bee983`, dopo le correzioni `78bb2431..b6bee983` (SUMMARY.md, «Correzioni dopo la verifica»; la prima verifica è in `41cc54ce`). Suite completa, copertura, test UI ed e2e eseguiti ora, non presi da SUMMARY.md. Nessun test hardware (`--ignored`) eseguito.

## Gates
| Gate | Status | Details |
|---|---|---|
| Build | PASS | `cargo build --workspace --locked` ok; il nuovo `Cargo.lock` (`964e66b4`) è accettato da `--locked` |
| Tests | PASS | 1554 passed, 0 failed, 22 ignored (hardware) su 49 binari (erano 1541: +13, i test nuovi di `display_state`, `light_tray` e `power::`) |
| Coverage | PASS | 89.84% lines (TOTAL `cargo llvm-cov`: 89812 righe, 9123 mancate); exit 0 con `--fail-under-lines 80`. File della fase: `display_state.rs` 93.93%, `src-tauri/src/power.rs` 87.62%, `light_tray.rs` 39.68% (era 19.65%) |
| Lint | PASS | `cargo fmt --all --check` ok; `cargo clippy --workspace --all-targets --locked -- -D warnings` ok; ogni `#[allow(unsafe_code)]` di `display_state.rs` porta un `reason` |
| Hexagonal/Safety/Protocol/Hygiene | PASS (con avvisi) | 5.1 ora pulito (D-2026-11-07-display-idle-saver-1); 5.2, 5.3, 5.5, 5.7, 5.8, 5.9 puliti; 5.4 e 5.10 solo riscontri preesistenti fuori fase; 5.6 nessun `unwrap`/`expect`/`panic!` nuovo fuori dai test; 5.11 avvisi residui (vedi Warnings) |
| Consistency | PASS | 7 commit di correzione con scope `display-idle-saver` e trailer Claude; D-2026-11-06-display-idle-saver-1..5 rispettate; D-2026-11-07-display-idle-saver-1 presente in `.jdi/decisions/` e in `DECISIONS.md:506` |
| UI Validation | PASS | `npm run test:unit` 715/715, linee 99.19%; Playwright 1280/1280 (7,5 min), nessun flaky in questo giro; nessun file UI cambiato dalle correzioni |
| DoD | PENDING_MANUAL | Auto tutti PASS; 2 Manual di fase + 2 Manual di progetto in attesa |

## Esito delle correzioni (rispetto a 41cc54ce)
| Warning precedente | Esito |
|---|---|
| 1. `unicode-segmentation` nel core (5.1) | Risolto: D-2026-11-07-display-idle-saver-1 ratifica il crate (`no_std`, senza dipendenze, senza `unsafe` né I/O); il comando 5.1 non dà output |
| 2. Copertura dei file nuovi | Risolto per i file della fase (`display_state.rs` 93.93%, `power.rs` 87.62%); `light_tray.rs` resta basso nel totale (vedi Warning 3) |
| 3. DRY del ponte `Suspend` | Risolto: `forward`/`forwarder`, `SUSPEND_WAIT`, `SUSPEND_BUDGET`, `SuspendDone` una volta sola in `crates/bezel-power/src/display_state.rs:86-147`, usati da Light e Studio |
| 4. Errore silenzioso di `power::watch` | Risolto: `DiagCode::PowerWatchNotStarted` / `PowerEventsUnavailable` via `diag::report` (`apps/bezel-studio/src-tauri/src/power.rs:174-211`) |
| 5. `cargo audit` | In parte: `rustls-webpki`, `webpki-roots`, `tokio`, `async-executor` aggiornati; restano `zip` e `zbus*` (vedi Warnings 1-2) |
| 6. Regex TODO del DoD | Risolto: il comando di PROJECT.md stampa `OK` |

## Blockers
Nessuno.

## Warnings
1. **5.11 `zip` 4.6.1** (RUSTSEC-2026-0367/0368/0369/0429, correzione solo da 9.0.0): tutti e quattro riguardano `ZipArchive::extract` / `ZipStreamReader::extract`. Verificato: `crates/bezel-themes/src/native.rs` usa solo `ZipArchive`/`ZipWriter` e `by_name`, nessuna chiamata a `extract` nel workspace. Non raggiungibile oggi, ma i `.bezeltheme` arrivano da fuori: pianificare il salto di major (a parte, fuori fase) e non introdurre mai `extract` finché resta 4.x.
2. **5.11 `zbus_names`/`zvariant`/`zvariant_derive` non mantenuti**: confermato con `cargo tree -i zbus_names --target all` che arrivano solo da `tauri-plugin-opener` e `tauri-plugin-single-instance` (ramo Linux). Piattaforma non supportata (D-2026-10-16-studio-redesign-fidelity-27), non compilati per Windows: solo informativo, si risolve con gli aggiornamenti dei plugin Tauri.
3. **Copertura di `crates/bezel-cli/src/light_tray.rs` 39.68%**: il file è in gran parte il tray preesistente; della parte di questa fase restano scoperte le stampe d'errore e i rami di fallimento di spawn/registrazione (come dichiarato in SUMMARY). TOTAL ben sopra la soglia.
4. **Esito del ponte ignorato**: `forwarder` scarta il `Forwarded` (`let _ = forward(...)`, `crates/bezel-power/src/display_state.rs:145`), quindi un `Suspend` andato in `TimedOut` o `Dropped` non lascia traccia né in Light né in Studio. Comportamento identico a prima della correzione; minore, valutare un callback o un `DiagCode` se servirà diagnosticare sospensioni lente.
5. **Test che si registrano davvero con Windows**: `light_tray::tests::idle_watch_power_feeds_the_worker` e il test di `power::watch` di Studio aprono una `Watch` reale (non hardware Bezel, consentito); in una sessione senza notifiche di alimentazione (servizio, CI futuro) potrebbero fallire. Accettabile con la sola piattaforma Windows desktop.
6. **Limiti noti (T-7, invariati)**: un link Studio fuori sessione al momento dell'evento non viene abbassato né ripristinato; senza motore (demo/test) Studio continua a disegnare il tema in idle. Da tenere presente nella prova manuale.

## DoD Checklist (gate 8)
| # | Criterion | Source | Type | Status | Evidence |
|---|---|---|---|---|---|
| 1 | Transizioni monitor spento/acceso e sospensione/risveglio portano l'idle dentro e fuori | CONTEXT | Auto | PASS | `display_state::tests::idle_*` (11) + `domain::idle::tests::idle_*` verdi nella suite completa |
| 2 | In idle la cattura loopback è ferma e alla riaccensione riparte | CONTEXT | Auto | PASS | `bezel_runtime tests::idle_releases_audio_capture_and_leaving_reopens_it` ok |
| 3 | In idle `set_brightness` al 10% e logo ai fake, nessun frame di tema; ripristino della luminosità | CONTEXT | Auto | PASS | `crates/bezel-core/tests/idle.rs` + `tests::idle_shows_the_moving_logo_without_theme_frames`, `idle_opens_no_new_connection`, `light_tray::tests::idle_light_targets_drive_the_runtime` ok |
| 4 | Parità chiavi i18n en/pt-BR/it della nuova opzione | CONTEXT | Auto | PASS | `npm run test:unit` 715/715 (comprende `i18n.test.mjs`) |
| 5 | Opzione attiva di default; disattivata, l'idle non scatta | CONTEXT | Auto | PASS | `settings::tests::idle_saver_is_on_unless_turned_off`, `power::tests::idle_option_off_ignores_power_events`, `light_tray::tests::idle_option_off_leaves_power_events_alone` ok |
| 6 | `cargo test --workspace` exit 0 | PROJECT | Auto | PASS | 1554 passed, 0 failed |
| 7 | Copertura >= 80% linee | PROJECT | Auto | PASS | 89.84% TOTAL, exit 0 |
| 8 | Nessun TODO/FIXME senza issue | PROJECT | Auto | PASS | Comando di PROJECT.md (corretto in `c26e5442`) stampa `OK` |
| 9 | Con tema con spettro e monitor spenti, `powercfg /requests` non mostra SYSTEM per Bezel | CONTEXT | Manual | MANUAL_REQUIRED | Incollare l'output di `powercfg /requests` (da amministratore) a monitor spenti con un tema spettro attivo, in Light e in Studio |
| 10 | Sull'8,8 pollici il logo si sposta ogni 30 s e la luminosità cala | CONTEXT | Manual | MANUAL_REQUIRED | Foto/video dell'8,8 pollici (COM6) per almeno 60 s a monitor spenti, poi alla riaccensione (tema e luminosità di prima) |
| 11 | CHANGELOG aggiornato | PROJECT | Manual | MANUAL_REQUIRED | Voce «Screens» in Unreleased presente; la sezione `## [version]` arriva alla pubblicazione |
| 12 | README descrive il comportamento attuale | PROJECT | Manual | MANUAL_REQUIRED | Decidere se il README va aggiornato con l'idle a monitor spenti |

## Recommendation
Nessun blocco e tutti gli avvisi della prima verifica che chiedevano correzioni sono chiusi. La fase si può chiudere con `/jdi-ship` dopo le due prove manuali (9 e 10) sull'8,8 pollici reale con una build locale (`scripts/windows/build-evo.ps1`). Da pianificare a parte: aggiornamento di `zip` a 9.x (Warning 1); facoltativo un segnale per i `Suspend` scaduti (Warning 4).
