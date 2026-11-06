# Phase 22: Display idle saver — Plan  (slug: display-idle-saver)

## Goal
Quando Windows spegne i monitor Bezel ferma la cattura audio, abbassa la luminosità, riduce il traffico e mostra un logo mobile anti burn-in; alla riaccensione riprende. La sospensione/risveglio del PC applica lo stesso stato.

## Locked decisions (from CONTEXT.md)
- D-2026-11-06-display-idle-saver-1: luminosità idle fissa 10%, ripristino della precedente alla riaccensione.
- D-2026-11-06-display-idle-saver-2: logo Pixel che si sposta ogni 30 s, tutti gli schermi; rilevamento GUID_CONSOLE_DISPLAY_STATE in `crates/bezel-power`; idle = caso d'uso del core.
- D-2026-11-06-display-idle-saver-3: standby rev C solo alla sospensione del PC (ripresa al risveglio); a monitor spenti tutti mostrano il logo; schermi già spenti/addormentati non si svegliano.
- D-2026-11-06-display-idle-saver-4: tutti i modelli, Bezel Light, sospensione/risveglio; opzione attiva di default, disattivabile in Preferenze; i18n en/pt-BR/it.
- D-2026-11-06-display-idle-saver-5: loopback fermo in idle e in sospensione, riparte al ritorno; stessa opzione di D-4.
- D-2026-10-03-power-off-standby-2/3/6: `app::standby::at_shutdown` è l'unico modo di applicare la scelta rev C (riusato alla sospensione).

## Tasks

### Wave 1

#### T-1: caso d'uso idle nel core (stato, luminosità, posizione del logo)
- **Specialist:** jdi-doer-bezel
- **Files modified:** `crates/bezel-core/src/domain/idle.rs` (nuovo), `crates/bezel-core/src/domain/mod.rs`, `crates/bezel-core/src/app/idle.rs` (nuovo), `crates/bezel-core/src/app/mod.rs`
- **Acceptance:**
  - `domain::idle`: `PowerEvent { DisplayOff, DisplayOn, DisplayDimmed, Suspend, Resume }` e macchina `IdleState` con `enabled`; produce `EnterIdle / LeaveIdle / Suspend / Resume / Nothing`; `DisplayDimmed` = nessun cambio; opzione spenta = sempre `Nothing`; eventi ripetuti idempotenti; `Resume` dopo `Suspend` con monitor ancora spenti torna in idle.
  - `IDLE_BRIGHTNESS = 10`; `interval(Family)` = 30 s, 60 s per `TuringRevA` e `KipyeRevD` (banda bassa); `logo_box(canvas, slot)` deterministico, sempre dentro il canvas, slot consecutivi in posizioni diverse; `IdleBrightness` ricorda l'ultimo livello impostato e restituisce quello da ripristinare (fallback passato dall'adattatore).
  - `app::idle::{dim, restore}` su `&mut dyn ScreenLink`: `dim` invia solo `set_brightness(10)` e solo se `model.brightness`; `restore` rimanda il livello ricordato; nessun altro comando.
- **Dependencies:** none
- **Test:** `cargo test -p bezel-core idle` (almeno 8 test, nomi `idle_*`; un log ordinato unico nel fake per i comandi inviati)
- **Status:** pending

### Wave 2 (parallel-eligible)

#### T-2: rilevamento monitor spenti/sospensione in `bezel-power`
- **Specialist:** jdi-doer-bezel
- **Files modified:** `crates/bezel-power/src/display_state.rs` (nuovo), `crates/bezel-power/src/lib.rs`, `crates/bezel-power/Cargo.toml`
- **Acceptance:**
  - `display_state::watch(handler: impl Fn(PowerEvent) + Send + Sync + 'static) -> io::Result<Watch>` senza finestra: `PowerSettingRegisterNotification(GUID_CONSOLE_DISPLAY_STATE, DEVICE_NOTIFY_CALLBACK)` + `PowerRegisterSuspendResumeNotification(DEVICE_NOTIFY_CALLBACK)`; `Watch` deregistra entrambe nel `Drop`. Feature `windows-sys`: `Win32_System_Power`, `Win32_System_SystemServices` (verificare i nomi esatti su docs.rs 0.61).
  - `unsafe` solo nel modulo `imp`, `#[allow(unsafe_code, reason = ...)]` e `// SAFETY:` su ogni blocco, come `session.rs`; la mappatura pura `event_of(kind, data) -> Option<PowerEvent>` (0 off, 1 on, 2 dimmed; `PBT_APMSUSPEND`, `PBT_APMRESUMEAUTOMATIC`) è testata senza FFI.
  - Il callback di `Suspend` è sincrono (Windows aspetta il ritorno): documentato un budget di 2 s per l'handler; gli altri eventi non bloccano.
- **Dependencies:** T-1
- **Test:** `cargo test -p bezel-power idle` (`display_state::tests::idle_event_maps_display_states`, `display_state::tests::idle_event_maps_suspend_and_resume`, `display_state::tests::idle_watch_registers_and_unregisters`)
- **Status:** pending

#### T-3: luminosità idle nell'Owner (ricorda l'ultimo livello, abbassa e ripristina)
- **Specialist:** jdi-doer-bezel
- **Files modified:** `crates/bezel-power/src/owner.rs`, `crates/bezel-power/tests/screen_owner.rs`
- **Acceptance:**
  - Ogni link tenuto dall'Owner registra in `IdleBrightness` i livelli passati da `set_brightness` (Studio via pipe e Light); `Owner::idle(true)` chiama `app::idle::dim` su ogni link aperto, `Owner::idle(false)` `app::idle::restore` con l'ultimo livello (fallback: `start_brightness` del record rev C se noto, altrimenti `Brightness::MAX`).
  - Un `set_brightness` dell'utente durante l'idle aggiorna solo il livello da ripristinare; nessun link chiuso o schermo addormentato viene aperto per l'idle.
- **Dependencies:** T-1
- **Test:** `cargo test -p bezel-power --test screen_owner idle` (`idle_dims_every_kept_screen_and_restores_the_last_level`, `idle_never_opens_a_screen`, `idle_keeps_a_level_set_while_idle_for_the_restore`)
- **Status:** pending

#### T-4: modalità idle nel runtime (logo mobile, niente audio, niente nuove connessioni) e protocollo
- **Specialist:** jdi-doer-bezel
- **Files modified:** `crates/bezel-runtime/src/lib.rs`, `crates/bezel-runtime/src/wire.rs`, `crates/bezel-runtime/src/tests.rs`
- **Acceptance:**
  - `Engine::idle(on)`: ogni scena con link aperto mostra il logo (`LOGO`, documento come `resting_document` con cornice da `idle::logo_box`) e cambia posizione solo a ogni `interval(family)`; tra due spostamenti nessun frame né campionamento sensori; nessun frame di tema; uscendo torna al tema salvato o al logo statico se Light è spento (`rest`).
  - In idle nessuna richiesta spettro: il worker audio rilascia ogni `AudioCapture` (che nel `Drop` chiama `IAudioClient::Stop`); all'uscita la cattura si riapre. Nessun nuovo `Presenter` né riapertura per scene senza link.
  - `wire::Options.idle_saver` (serde default `true`) e `RuntimeOp::Idle { on }`; con l'opzione spenta `idle(true)` non fa nulla e spegnerla durante l'idle esce dall'idle; `Engine::idle_saver()` per Light.
- **Dependencies:** T-1
- **Test:** `cargo test -p bezel-runtime idle` (`idle_shows_the_moving_logo_without_theme_frames`, `idle_releases_audio_capture_and_leaving_reopens_it`, `idle_opens_no_new_connection`, `idle_is_ignored_when_the_option_is_off`, `idle_ends_on_the_resting_logo_when_light_is_off`)
- **Status:** pending

### Wave 3 (parallel-eligible)

#### T-5: Bezel Light collega gli eventi (idle, sospensione con standby rev C, risveglio)
- **Specialist:** jdi-doer-bezel
- **Files modified:** `crates/bezel-cli/src/light_tray.rs`
- **Acceptance:**
  - `run` tiene un `display_state::Watch`; gli eventi passano per `IdleState` e una funzione `on_power_event(engine, owner, transition)` testabile: `EnterIdle/LeaveIdle` → `engine.idle` + `owner.idle`; `Suspend` → `engine.suspend(true)` e scelta rev C con `bezel_cli::finish_shutdown` sui link tenuti, entro 2 s, senza chiudere l'Owner; `Resume` → `engine.suspend(false)` e il percorso `reopen_after_pause` esistente.
  - `idle_saver_on(settings)` legge `idleSaver` (assente = attivo) come `audioCapture`; con l'opzione spenta nessuna transizione.
- **Dependencies:** T-2, T-3, T-4
- **Test:** `cargo test -p bezel-cli idle` (`light_tray::tests::idle_saver_is_on_unless_turned_off`, `light_tray::tests::idle_power_events_drive_engine_and_owner` con engine e owner fake: test sul gestore vero, non grep del sorgente)
- **Status:** pending

#### T-6: opzione «Idle quando i monitor si spengono» nel backend Studio
- **Specialist:** jdi-doer-bezel
- **Files modified:** `apps/bezel-studio/src-tauri/src/settings.rs`, `apps/bezel-studio/src-tauri/src/commands.rs`, `apps/bezel-studio/src-tauri/src/dto.rs`, `apps/bezel-studio/src-tauri/src/backend.rs`, `apps/bezel-studio/src-tauri/build.rs`, `apps/bezel-studio/src-tauri/capabilities/default.json`, `apps/bezel-studio/src-tauri/src/lib.rs`
- **Acceptance:**
  - `Settings.idle_saver: DefaultOn` (`idleSaver`, assente = attivo); `PreferencesDto.idle_saver`; comando `set_idle_saver(on)` che salva e chiama `sync_engine_options`, registrato in `lib.rs`, `build.rs` e capability.
  - `wire::Options.idle_saver` riempito dalle impostazioni nei due punti dove oggi si passa `audio_capture`.
- **Dependencies:** T-4
- **Test:** `cargo test -p bezel-studio idle_saver` (`settings::tests::idle_saver_is_on_unless_turned_off`, test del backend `idle_saver_reaches_the_engine_options`)
- **Status:** pending

### Wave 4 (parallel-eligible)

#### T-7: Studio collega gli eventi (idle sui suoi link, sospensione e risveglio)
- **Specialist:** jdi-doer-bezel
- **Files modified:** `apps/bezel-studio/src-tauri/src/power.rs`, `apps/bezel-studio/src-tauri/src/power/tests.rs`, `apps/bezel-studio/src-tauri/src/engine.rs`, `apps/bezel-studio/src-tauri/src/studio.rs`, `apps/bezel-studio/src-tauri/src/lib.rs`
- **Acceptance:**
  - `EngineBridge::idle(on)` invia `RuntimeOp::Idle` e lo conserva come `suspended` per il motore di recupero; `Studio` abbassa/ripristina i propri link live con `app::idle` e `IdleBrightness` (anche `live_brightness` registra il livello).
  - `power::on_power_event(backend, transition)`: `Suspend` → `shut_down(backend, now + 2 s)` (stessa scelta rev C di D-3, schermi addormentati non svegliati); `Resume` → `leave_final_state`; con opzione spenta nulla; `lib.rs` tiene il `Watch` e chiama solo questa funzione.
- **Dependencies:** T-2, T-6
- **Test:** `cargo test -p bezel-studio idle` (`power::tests::idle_dims_live_screens_and_restores_them`, `power::tests::idle_suspend_applies_choices_and_resume_leaves_final_state`, `power::tests::idle_option_off_ignores_power_events`; un log ordinato unico nel fake)
- **Status:** pending

#### T-8: interruttore in Preferenze, i18n e changelog
- **Specialist:** jdi-doer-bezel
- **Files modified:** `apps/bezel-studio/src/ui/preferences.js`, `apps/bezel-studio/src/bridge.js`, `apps/bezel-studio/src/demo-backend.js`, `apps/bezel-studio/src/i18n/en.js`, `apps/bezel-studio/src/i18n/pt-BR.js`, `apps/bezel-studio/src/i18n/it.js`, `apps/bezel-studio/tests/ui/bridge.test.mjs`, `apps/bezel-studio/tests/ui/spectrum.test.mjs`, `CHANGELOG.md`
- **Acceptance:**
  - Switch `prefs-idle-saver` (role switch, `aria-describedby`) con titolo e suggerimento (`prefs.idleTitle`, `prefs.idleSaver`, `prefs.idleHint`) in en/pt-BR/it; errore → ripristina lo stato e notifica, come la cattura audio.
  - `bridge.setIdleSaver(on)` → `set_idle_saver`; demo backend con `idleSaver: true` di default.
  - Voce Unreleased nel CHANGELOG (inglese, per l'utente).
- **Dependencies:** T-6
- **Test:** da `apps/bezel-studio`: `node --test tests/ui/i18n.test.mjs tests/ui/bridge.test.mjs tests/ui/spectrum.test.mjs` (parità chiavi: «en, pt-BR and it have the same keys, each with the same placeholders»)
- **Status:** pending

## Execution
- Total tasks: 8
- Waves: 4
- Estimated parallel speedup: 2x

## Files modified (all tasks)
- crates/bezel-core/src/domain/{idle.rs,mod.rs}, crates/bezel-core/src/app/{idle.rs,mod.rs}
- crates/bezel-power/src/{display_state.rs,lib.rs,owner.rs}, crates/bezel-power/Cargo.toml, crates/bezel-power/tests/screen_owner.rs
- crates/bezel-runtime/src/{lib.rs,wire.rs,tests.rs}
- crates/bezel-cli/src/light_tray.rs
- apps/bezel-studio/src-tauri/src/{settings,commands,dto,backend,lib,power,engine,studio}.rs, src/power/tests.rs, build.rs, capabilities/default.json
- apps/bezel-studio/src/{ui/preferences.js,bridge.js,demo-backend.js,i18n/en.js,i18n/pt-BR.js,i18n/it.js}, tests/ui/{bridge,spectrum}.test.mjs
- CHANGELOG.md

## Test requirements
- Rust (solo crate toccati): `cargo test -p bezel-core idle`, `cargo test -p bezel-power idle`, `cargo test -p bezel-runtime idle`, `cargo test -p bezel-cli idle`, `cargo test -p bezel-studio idle`; `cargo clippy -p <crate> --all-targets -- -D warnings` per ciascuno.
- UI: `node --test tests/ui/i18n.test.mjs tests/ui/bridge.test.mjs tests/ui/spectrum.test.mjs` (da `apps/bezel-studio`).
- I conteggi sono minimi; i nomi elencati si verificano con `--exact`.
- Copertura minima 80% (PROJECT.md), misurata una volta al `/jdi-verify`.
- Manuale (DoD): `powercfg /requests` senza SYSTEM per Bezel a monitor spenti con tema spettro; foto/video dell'8,8" con logo che si sposta e luminosità ridotta.

## Note (scelte del planner)
- DoD rivisto: la cattura audio si verifica in `bezel-runtime` (`idle_releases_audio_capture_and_leaving_reopens_it`), non in `bezel-sensors`: `audio.rs` non cambia perché `Capture::drop` chiama già `IAudioClient::Stop` e il worker rilascia le catture senza richieste. Il test con dispositivi fake del logo vive in `bezel-runtime` (il core non dipende da `bezel-devices`); `cargo test -p bezel-core idle` copre luminosità e transizioni. L'opzione si verifica con `cargo test -p bezel-studio idle_saver`; la parità i18n con `tests/ui/i18n.test.mjs`.
- Opzione spenta = comportamento di oggi per tutto, anche sospensione/risveglio (coerente con D-5).
- `DisplayDimmed` (monitor attenuato) non è idle.
- Intervallo più lungo (60 s) solo per Turing 3,5" rev A e Kipye, come da Notes di CONTEXT e D-4; gli altri 30 s.
- Ripristino luminosità: ultimo livello impostato tramite Bezel in questa sessione; altrimenti livello di avvio rev C registrato; altrimenti 100%.
- Rilevamento a callback (`DEVICE_NOTIFY_CALLBACK`), senza finestra nascosta: Light (tao) e Studio (Tauri) non devono esporre l'HWND.
