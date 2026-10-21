# T-9d — Temi predefiniti 1920×480 — riepilogo

21 ottobre 2026. Avviata da Codex/ChatGPT nella worktree `F:/dev/bezel-ux2-t9d`
(branch `ux2/t9d`, base `ef0fbc8`), completata da Claude su richiesta
dell'utente. Tavole ThemePixel, ThemeNebula, ThemePaper, ThemeNowPlaying.

## Cosa è fatto

- **Quattro temi** orizzontali 1920×480 in `themes/`: `pixel-wide`,
  `nebula-wide`, `paper-wide`, `now-playing-wide`. Solo widget nativi
  (orologio, valori, anelli, barre a blocchi, grafici, meteo, player, spettro,
  forme e un SVG di architettura per tema), nessun dato salvato nel tema.
  Generati da un solo script deterministico, `scripts/create-ux-v2-themes.py`.
- **Caratteri inclusi** (commit di Codex `20642b7`): Fraunces (anche il taglio
  ottico «144pt»), Instrument Sans, Space Grotesk, Bricolage Grotesque, statici
  con licenza OFL in `themes/fonts/`, registrati nel renderer e nel demo;
  `THIRD_PARTY.md` aggiornato.
- **Demo**: nomi delle famiglie tra virgolette nel font del canvas
  (`canvasFont`, un solo punto per tutti i testi: prima «Fraunces 144pt»
  ricadeva sul carattere predefinito a 10 px); letture demo che i temi usano e
  che il motore già offre (nome CPU/GPU, ventola CPU, VRAM, memoria totale e
  disponibile come complemento dell'usata, meteo configurato).
- **Test dei temi inclusi**: ogni tema dentro il canvas, oggetti con id unici,
  rendering nativo con letture misurate, meteo, musica con copertina e spettro
  misurato, e con tutti gli ingressi mancanti; chiavi sensore ammesse solo se
  il demo o il meteo configurato le forniscono.
- Ritocco di Claude: in Paper «· edizione della sera» accanto a «Bezel».

## Decisione dell'utente durante la task

Le tavole mostrano dati che oggi non hanno una sorgente (previsioni orarie,
cache RAM, variazioni degli indici, avanzamento della giornata): **fuori da
studio-ux-v2**. I temi usano solo dati esistenti; le idee sono in
`.jdi/todos/2026-10-21-widget-data-extensions.md`. Per questo: Pixel mostra il
volume reale nella fascia «Giornata»; Nebula e Paper non hanno la curva delle
previsioni; Paper non ha le frecce ▲▼.

## Commit

Codex: `20642b7`. Completamento: `fd41b2e` (font del canvas e letture demo),
`72bac4b` (quattro temi, generatore, test), `97599d1` (confronto con le
tavole), più il commit di questo riepilogo.

## Test eseguiti

- Rust: `cargo test -p bezel --test bundled_themes` 2/2 (anche il nuovo test
  con ingressi presenti e assenti), `cargo test -p bezel-render` 138 verdi
  (1 ignorato come prima), clippy `-p bezel -p bezel-render --all-targets
  -D warnings` e `cargo fmt --check` verdi.
- Unitari UI: 671/672; l'unico fallimento (`video-framing.test.mjs`) è
  uguale sulla base. Nuovo test della citazione dei font; adeguati due test del
  demo alle letture aggiunte.
- Confronto: `evidence/t9d/index.html` con tavola, frame nativo con e senza
  dati e demo, in italiano chiaro e scuro.

## Differenze rimaste rispetto alle tavole

- I dati senza sorgente indicati sopra (scelta dell'utente).
- Il testo del meteo è il widget Meteo nativo (città, temperatura, condizione
  su righe), non la frase della tavola.
- Il demo è un'anteprima approssimata: non applica la spaziatura delle
  maiuscole e usa dati d'esempio; il riferimento è il frame nativo.
- La copertina del Now Playing dipende dalla sessione multimediale reale.

## Da annotare per T-10

- I temi vanno copiati in `target/release/themes` alla prossima build e
  inclusi nel pacchetto (lo fanno gli script di build come per gli altri).
- Decidere se servono versioni 3,5″ dei quattro temi (domanda aperta del
  passaggio Design).
