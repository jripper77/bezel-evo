# Phase 22: Display idle saver — Context  (slug: display-idle-saver)

## Goal
Quando Windows spegne i monitor Bezel ferma la cattura audio, abbassa la luminosità, riduce il traffico e mostra un logo mobile anti burn-in; alla riaccensione riprende. La sospensione/risveglio del PC applica lo stesso stato.

## Locked decisions
- D-2026-11-06-display-idle-saver-1: luminosità idle fissa al 10%, ripristino alla riaccensione.
- D-2026-11-06-display-idle-saver-2: logo Pixel mobile ogni 30 s fisso, su tutti gli schermi; rilevamento via GUID_CONSOLE_DISPLAY_STATE in `crates/bezel-power`; idle = caso d'uso del core.
- D-2026-11-06-display-idle-saver-3: la scelta di standby rev C vale solo per la sospensione del PC; a monitor spenti tutti mostrano il logo; schermi già spenti/addormentati non vengono svegliati.
- D-2026-11-06-display-idle-saver-4: tutti i modelli (Turing 3,5" e Kipye con aggiornamento più raro), Bezel Light e sospensione/risveglio inclusi; opzione attiva di default, disattivabile in Preferenze; i18n en/pt-BR/it.
- D-2026-11-06-display-idle-saver-5: la cattura loopback si ferma in idle e in sospensione e riparte al ritorno; stessa opzione di D-4.

## Canonical refs
- `crates/bezel-sensors/src/audio.rs` (loopback), `apps/bezel-studio/src-tauri/src/settings.rs` (`audio_capture`)
- `crates/bezel-core/src/ports/mod.rs:132,141` (`set_brightness`, `screen_off`)
- `apps/bezel-studio/src-tauri/src/standby.rs`, `bezel_core::domain::standby`
- `crates/bezel-power`, D-2026-10-03-power-off-standby-1..9, `.jdi/todos/2026-10-03-power-off-standby.md`
- Canvas design (logo Pixel): https://claude.ai/artifact/7muP7mTQAwBH8diy2AWZ6p

## Out of scope
- PC-idle screensaver per inattività tastiera/mouse (TODO.md:167)
- `bezel run` headless
- Luminosità/intervallo configurabili

## Definition of Done

### Auto-verifiable
- [ ] Le transizioni monitor spento → acceso (e sospensione → risveglio) portano lo stato idle dentro e fuori
      **Verify:** `cargo test -p bezel-power idle`
      **Source:** CONTEXT
- [ ] In idle la cattura loopback è ferma e alla riaccensione riparte
      **Verify:** `cargo test -p bezel-sensors audio_capture_stops_on_idle` (nome da raffinare nel piano)
      **Source:** CONTEXT
- [ ] In idle il core invia `set_brightness` al 10% e il logo ai dispositivi fake, nessun frame di tema; alla riaccensione ripristina la luminosità
      **Verify:** `cargo test -p bezel-core idle`
      **Source:** CONTEXT
- [ ] Le chiavi i18n della nuova opzione hanno parità en/pt-BR/it
      **Verify:** controllo i18n esistente di `apps/bezel-studio` (comando individuato dal planner)
      **Source:** CONTEXT
- [ ] L'opzione è attiva di default e, disattivata, l'idle non scatta
      **Verify:** `cargo test -p bezel-studio idle_option` (nome da raffinare nel piano)
      **Source:** CONTEXT

### Manual
- [ ] Con un tema con spettro e monitor spenti, `powercfg /requests` non mostra più SYSTEM per Bezel
      **Verify:** human confirmation required
      **Evidence:** output di `powercfg /requests`
      **Source:** CONTEXT
- [ ] Sull'8,8" il logo si sposta ogni 30 s e la luminosità cala
      **Verify:** human confirmation required
      **Evidence:** foto/video dello schermo in idle
      **Source:** CONTEXT

## Notes
Il rilevamento va nell'adattatore `crates/bezel-power` (unsafe isolato e documentato); studio e Bezel Light collegano solo l'evento. Il Turing 3,5" ha banda bassa: logo statico per intervallo più lungo. Il piano deve coprire anche il comando di sospensione (scelta di standby rev C) e il ripristino al risveglio.
