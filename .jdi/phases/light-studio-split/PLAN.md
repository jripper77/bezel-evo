# Piano — light-studio-split

Decisioni: D-2026-11-05-light-studio-split-1…4.

## Stato di partenza (verificato nel codice, 5/11)

- `remember_document` (backend.rs) scrive `screenThemes[schermo]` a ogni apertura e salvataggio:
  il tema aperto in Studio diventa il tema di Light.
- `finish_live_off` toglie lo schermo da `liveScreens` e chiama `live(false)`: spegnere la
  diretta toglie lo schermo anche a Light.
- `restore_live` all'avvio riaccende la diretta su tutti i `liveScreens`.
- Studio scrive le bozze nella stessa scena del runtime che Light presenta (chiave = schermo),
  quindi i profili si fermano mentre Studio guarda lo schermo.
- La panoramica dei profili mette come «tema base» il documento aperto in Studio.

## Task

| # | Area | Cosa | File |
|---|---|---|---|
| T1 | runtime | `RuntimeOp::Revert`: la scena torna al documento salvato (di Light), riprendono i profili | bezel-runtime lib.rs, wire.rs, tests.rs |
| T2 | studio bridge | `Draft.saved` opzionale (una bozza in diretta non diventa il salvato di Light); `release(key)` = Revert + oblio della bozza | engine.rs |
| T3 | settings | `editorThemes` + migrazione; Studio riapre da lì | settings.rs, backend.rs |
| T4 | backend | chiave di scena `__editor__/<schermo>` fuori diretta; diretta on/off senza toccare `liveScreens`; niente `restore_live`; panoramica profili dai settings | backend.rs, lib.rs |
| T5 | backend | comando `use_on_light(theme, screen, profile)`; dopo il salvataggio Light riceve la nuova versione (scene base, profili) | backend.rs, commands.rs, dto |
| T6 | light | fallback `lastTheme` solo per settings vecchi (senza `editorThemes`) | light_tray.rs, profiles.rs |
| T7 | UI | «Usa su Light» nella galleria, conferma al salvataggio di un tema in uso, testi en/pt-BR/it, demo backend | ui/*, i18n, bridge.js, demo-backend.js |
| T8 | test | unit Rust dei crate toccati, unit UI ed e2e delle aree toccate | — |

Commit atomici `feat(light-studio-split): …`.
