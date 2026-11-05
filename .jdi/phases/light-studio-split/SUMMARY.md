# Riepilogo — light-studio-split (5/11/2026)

Commit: 9b4e11f8, 04614b9c, ba504eda, 18c21662 (più questo riepilogo).

## Fatto

- **Runtime**: `RuntimeOp::Revert` riporta una scena al tema salvato (di Light) e lascia ripartire
  il profilo; le scene `__editor__…` (solo di Studio) si eliminano quando Studio si disconnette.
- **Light**: il ripiego su `lastTheme` (ultimo tema aperto in Studio) vale solo per impostazioni
  vecchie senza `editorThemes`; idem nella panoramica dei profili e in `bezel` CLI.
- **Studio, impostazioni**: `editorThemes` (memoria dell'editor per schermo), migrato alla prima
  lettura; gli schermi che Light mostrava conservano il tema che mostravano.
- **Studio, backend**: aprire/salvare non toccano `screenThemes`; fuori diretta l'editor lavora
  sulla scena `__editor__/<schermo>`; la diretta non scrive `liveScreens` e non riparte
  all'avvio; spegnendola lo schermo torna a Light (Revert) o si spegne se Light non lo gestisce;
  la panoramica profili prende il tema base dai settings, non dal documento aperto.
- **Studio, comando `use_on_light`**: tema base (`screenThemes` + `liveScreens`) o profilo;
  uno schermo non ancora di Light riceve anche il tema base. Light lo mostra subito.
- **Salvataggio**: un tema in uso come base viene inviato subito alle scene di Light; se è in un
  profilo, `profiles.json` viene riscritto così Light rilegge i temi.
- **UI**: «Usa su Light» nella galleria (dettaglio e menu ⋯), scelta tema base / profilo con il
  profilo attivo proposto; distintivo «Base» sulle schede; conferma «Aggiornare anche Light?» al
  primo salvataggio di un tema in uso (non per i temi inclusi, che si salvano come copia).
  Testi en/pt-BR/it. Demo: temi di Light separati dal tema aperto.

## Test

- `cargo test` bezel-runtime 82, bezel-studio 238, bezel 104+: verdi; clippy e fmt verdi.
- UI unit 713/713; e2e themes, profiles, library, scenarios-shell, scenarios-library,
  live-screen-controls, dialogs-fidelity, shell: verdi; nuovo e2e «use on Light…».

## Non verificato

- Prova sull'hardware reale (Light + Studio + schermo): da fare dopo una build locale.
- Primo avvio di un utente nuovo: la diretta non assegna più nulla a Light, quindi finché non
  usa «Usa su Light» lo schermo torna spento chiudendo Studio.
