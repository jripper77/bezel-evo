# Phase 22: Review  (slug: display-idle-saver)

**Verdict:** APPROVED_PENDING_MANUAL

Verifica del 7 novembre 2026, branch `windows-improvements`, HEAD `7ee11094`. Suite completa, copertura ed e2e eseguiti ora (non presi da SUMMARY.md). Nessun test hardware (`--ignored`) eseguito.

## Gates
| Gate | Status | Details |
|---|---|---|
| Build | PASS | `cargo build --workspace --locked` ok (50,6 s); `Cargo.lock` non toccato dalla fase |
| Tests | PASS | 1541 passed, 0 failed, 22 ignored (hardware) su 49 binari; tutti i 40+ test `idle_*` del piano presenti e verdi |
| Coverage | PASS | 89.59% lines (TOTAL `cargo llvm-cov`, 89589 righe, 9327 mancate); exit 0 con `--fail-under-lines 80` |
| Lint | PASS | `cargo fmt --all --check` ok; `cargo clippy --workspace --all-targets --locked -- -D warnings` ok; i nuovi `#[allow(unsafe_code)]` di `display_state.rs` portano `reason = "..."` |
| Hexagonal/Safety/Protocol/Hygiene | PASS (con avvisi) | 5.2, 5.3, 5.4, 5.5, 5.7, 5.8, 5.9, 5.10 puliti per il codice della fase; 5.1 e 5.11 con rilievi preesistenti (vedi Warnings) |
| Consistency | PASS | 16 commit con scope `display-idle-saver`; 8/8 task con test; deviazioni dai `files_modified` documentate in SUMMARY; D-2026-11-06-display-idle-saver-1..5 rispettate |
| UI Validation | PASS (con avviso) | `npm run test:unit` 715/715, linee 99.19%; Playwright 1279/1280 al primo giro, l'unico fallimento (`inspector-redesign.spec.mjs:3`, dark-pt, `net::ERR_NO_BUFFER_SPACE`) ripassa da solo (2/2) |
| DoD | PENDING_MANUAL | Auto tutti PASS; 2 Manual di fase + 2 Manual di progetto in attesa |

## Blockers
Nessuno.

## Warnings
1. **5.1 (preesistente, fuori fase):** `crates/bezel-core/Cargo.toml` dipende da `unicode-segmentation = "1.13.3"`, fuori dalla whitelist (`thiserror`). Introdotto da `a6bccfff` (18 ottobre, `feat(text)`), non da questa fase, e nessuna D-XX lo ratifica. Alla lettera la regola 5.1 bloccherebbe; non lo conto come blocco di questa fase perché la fase non lo tocca. Serve una D-XX che allarghi la whitelist (crate puro, senza I/O) oppure spostare la segmentazione in un adattatore.
2. **Copertura dei file nuovi sotto l'80%** (TOTAL ok): `crates/bezel-power/src/display_state.rs` 70.25% (callback FFI e `register` raggiungibili solo con eventi reali di Windows), `apps/bezel-studio/src-tauri/src/power.rs` 73.87% (`watch` e il ponte del `Suspend` non testati), `crates/bezel-cli/src/light_tray.rs` 19.65% (file del tray in gran parte preesistente; `watch_power` non testato).
3. **DRY:** il ponte evento → thread con attesa sincrona del `Suspend` (`sync_channel` + `recv_timeout(SUSPEND_WAIT)`, budget 1,8 s su 2 s) è scritto due volte: `crates/bezel-cli/src/light_tray.rs:1061-1100` e `apps/bezel-studio/src-tauri/src/power.rs` (`watch`, `SUSPEND_WAIT`/`SUSPEND_BUDGET`). Candidato a un helper in `bezel-power::display_state`, insieme alle due costanti.
4. **Errore silenzioso:** `power::watch` in Studio restituisce `None` se lo spawn del thread o la registrazione falliscono, senza traccia (`.ok()?`). Light invece lo scrive su stderr. Il test d'audit vieta log fuori da `diag.rs`: valutare un `DiagCode` dedicato.
5. **5.11 supply chain (preesistente, `Cargo.lock` invariato nella fase):** `cargo audit` segnala 6 vulnerabilità: `rustls-webpki` 0.103.15 (RUSTSEC-2026-0382), `webpki-roots` 1.0.9 (RUSTSEC-2026-0381), `zip` 4.6.1 (RUSTSEC-2026-0367/0368/0369/0429). Avvisi: `zbus_names`/`zvariant`/`zvariant_derive` non mantenuti, `tokio` 1.53.1 unsound (RUSTSEC-2026-0386), `async-executor` 1.14.0 yanked.
6. **Comando DoD TODO di PROJECT.md:** dà un falso positivo su parole portoghesi preesistenti (`apps/bezel-studio/src/i18n/pt-BR.js:101` «todo o PC», `:109` «Todo o PC», del 16 e 18 ottobre). Non ci sono TODO veri: il criterio è soddisfatto, ma la regex andrebbe corretta (per esempio escludendo `src/i18n/pt-BR.js` oppure cercando solo `TODO`/`FIXME` maiuscoli).
7. **E2E instabile:** `tests/e2e/inspector-redesign.spec.mjs:3` [dark-pt] fallito una volta per `Failed to load resource: net::ERR_NO_BUFFER_SPACE` (problema di rete o del carico locale, area non toccata dalla fase); ripassa da solo.
8. **Limiti noti dichiarati in SUMMARY (T-7):** un link Studio fuori sessione al momento dell'evento (frame in corso, prestato a un job, in riconnessione) non viene abbassato né ripristinato. Senza motore (solo demo/test) Studio continua a disegnare frame del tema in idle. Da tenere presente nella prova manuale.

## DoD Checklist (gate 8)
| # | Criterion | Source | Type | Status | Evidence |
|---|---|---|---|---|---|
| 1 | Transizioni monitor spento/acceso e sospensione/risveglio portano l'idle dentro e fuori | CONTEXT | Auto | PASS | `display_state::tests::idle_*` (5) + `domain::idle::tests::idle_*` (11) verdi nella suite completa |
| 2 | In idle la cattura loopback è ferma e alla riaccensione riparte | CONTEXT (spostato in `bezel-runtime` dal planner) | Auto | PASS | `tests::idle_releases_audio_capture_and_leaving_reopens_it` ok |
| 3 | In idle `set_brightness` al 10% e logo ai fake, nessun frame di tema; ripristino della luminosità | CONTEXT | Auto | PASS | `crates/bezel-core/tests/idle.rs` (4) + `tests::idle_shows_the_moving_logo_without_theme_frames`, `idle_opens_no_new_connection` ok |
| 4 | Parità chiavi i18n en/pt-BR/it della nuova opzione | CONTEXT | Auto | PASS | `npm run test:unit` 715/715 (comprende `i18n.test.mjs`); `prefs.idleTitle/idleSaver/idleHint` nelle tre lingue |
| 5 | Opzione attiva di default; disattivata, l'idle non scatta | CONTEXT | Auto | PASS | `settings::tests::idle_saver_is_on_unless_turned_off`, `backend::tests::idle_saver_reaches_the_engine_options`, `power::tests::idle_option_off_ignores_power_events`, `light_tray::tests::idle_option_off_leaves_power_events_alone` ok |
| 6 | `cargo test --workspace` exit 0 | PROJECT | Auto | PASS | 1541 passed, 0 failed |
| 7 | Copertura >= 80% linee | PROJECT | Auto | PASS | 89.59% TOTAL, exit 0 |
| 8 | Nessun TODO/FIXME senza issue | PROJECT | Auto | PASS (falso positivo del comando) | Unici match: «todo» portoghese in `pt-BR.js:101,109` (vedi Warning 6) |
| 9 | Con tema con spettro e monitor spenti, `powercfg /requests` non mostra SYSTEM per Bezel | CONTEXT | Manual | MANUAL_REQUIRED | Incollare l'output di `powercfg /requests` (da amministratore) a monitor spenti con un tema spettro attivo in Light e in Studio |
| 10 | Sull'8,8" il logo si sposta ogni 30 s e la luminosità cala | CONTEXT | Manual | MANUAL_REQUIRED | Foto/video dell'8,8" (COM6) per almeno 60 s a monitor spenti, poi alla riaccensione (tema e luminosità di prima) |
| 11 | CHANGELOG aggiornato | PROJECT | Manual | MANUAL_REQUIRED | Voce «Screens» in Unreleased presente; manca la sezione `## [version]` finché non si pubblica |
| 12 | README descrive il comportamento attuale | PROJECT | Manual | MANUAL_REQUIRED | Controllare se il README va aggiornato con l'idle a monitor spenti |

## Recommendation
Nessun blocco: la fase si può chiudere con `/jdi-ship` dopo le due prove manuali (9 e 10), da fare sull'hardware reale con una build locale (`scripts/windows/build-evo.ps1`). Da pianificare a parte: D-XX o rifattorizzazione per `unicode-segmentation` nel core (Warning 1), aggiornamento di `zip`/`rustls-webpki`/`webpki-roots` (Warning 5), helper condiviso per il ponte del `Suspend` (Warning 3) e correzione della regex TODO di PROJECT.md (Warning 6).
