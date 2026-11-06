# Phase 22: Display idle saver — Summary  (slug: display-idle-saver)

## Executed tasks
- T-1: `domain::idle` (`PowerEvent`, `Transition`, `IdleState`, `IDLE_BRIGHTNESS` 10%, `interval` 30/60 s, `logo_box` su 9 posizioni a passo 4, `IdleBrightness`) e `app::idle::{dim, restore}` (solo `set_brightness`, niente se il modello non ha retroilluminazione). Commit 9ab1bd40. 15 test `idle_*` verdi (11 unitari + 4 d'integrazione con `FakeConnector`). Deviazioni: `Transition::Resume { idle }` porta se i monitor sono ancora spenti; `IdleState::set_enabled` restituisce la transizione quando l'opzione cambia; i test di `app::idle` stanno in `crates/bezel-core/tests/idle.rs` (nuovo, fuori da `files_modified`) perché i fake di `bezel-devices` si usano solo nei test d'integrazione del core.
