# T-1 — Testo: cifre fisse, parti con stile, testo troppo lungo, transizioni del numero

Data: 20 ottobre 2026. Branch `ux2/t1`, worktree `F:\dev\bezel-ux2-t1`, partenza `c562d90`.
Decisioni: D-2026-10-20-studio-ux-v2-05, -06. Tavole: ValueInspectorOpen, ClockInspectorOpen (parti digitali).

**T-1 completata e verificata**, pronta per il merge `--no-ff` in `windows-improvements`.
Solo motore, file del tema e demo: i controlli nell'inspector sono di T-7a (valore) e T-7b (orologio).

## Fatto

- `7465262` — dominio (`bezel-core`):
  - `TextStyle.tabular_digits` (vero di serie), `overflow: Option<TextOverflow>` (Shrink/Ellipsis/Scroll; `None` = come prima),
    `parts: TextParts` con `PartStyle { family, asset, size_percent, weight, color, baseline: Base|Super }` per
    `TextPart::{Prefix, Number, Unit, Suffix, Hours, Minutes, Seconds, Separators}` e `blink_separators`.
    I font delle parti entrano in `Theme::assets()` (si impacchettano col tema).
  - `clock::clock_parts` divide il testo dell'orologio in ore (`%H`, `%I`), minuti, secondi, separatori (testo fisso
    fra due di questi) e resto; `format_clock` ora è la loro unione (stessa uscita di prima, test invariati).
  - `sensor::format_reading_parts` divide numero e unità (`("4.72", " GHz")`); `format_reading` ne è l'unione.
  - Transizioni: `TransitionMode::Count` (il numero scorre in modo uniforme) e `Roll` (cifre a rullo, attenuato come
    Fluida); `ValueTransition.average` (1–30 letture, 1 = spento). `ShownValues` porta anche i `Roll { from, to, progress }`
    e `Roll::wheels(decimali, cifre)` dà per ogni colonna la cifra e quanto ha girato (contachilometri: una colonna gira
    solo mentre quelle a destra passano da 9). Un elemento con media resta in `shown` anche a riposo (mostra la media),
    senza chiedere fotogrammi; la media sopravvive alle letture mancanti e alle modifiche del tema.
- `ba0a6d0` — file del tema (`bezel-themes`): `tabularDigits` (omesso se vero), `overflow`, `parts` (ogni parte con
  `font`, `fontAsset`, `sizePercent`, `weight`, `color`, `baseline`; `separators.blink`), `valueTransition.mode`
  `count`/`roll` e `average` (omesso se 1). Valori fuori scala riportati nei limiti (10–400 %, peso 100–900, media ≤ 30),
  valori sconosciuti rifiutati. Un tema vecchio si apre e si riscrive senza campi nuovi (test).
  Importatori Turing/Python: campi nuovi ai valori predefiniti.
- `39a0650` — renderer (`bezel-render`): nuovo `text/rich.rs`. Valori, orologi e meteo (e i testi fissi con
  `overflow`) si disegnano a pezzi, ognuno con il proprio carattere/dimensione/peso/colore, tutti sulla stessa linea di
  base; `Super` alza la parte fino a portarne la cima al livello di quella del numero. Cifre fisse: si chiede `tnum`
  al font e in più ogni cifra occupa l'avanzamento della cifra più larga (centrata), così funziona anche con font senza
  `tnum`. Testo lungo: Rimpicciolisci (fino a 3 passi), Taglia… (puntini nel colore dell'ultima parte), Scorre
  (40 px/s, giro con spazio di 2 em, un fotogramma ogni 50 ms via `next_change`). Separatori lampeggianti: visibili nella
  prima metà di ogni secondo, fotogramma al cambio. Rullo: ogni cifra mostra quella attuale e la successiva spostate in
  verticale, ritagliate alla propria riga. Con «movimento ridotto» (`set_motion(false)`) scorrimento e lampeggio sono
  fermi e non chiedono fotogrammi. `ThemeRuntime::transitioning` ora guarda solo i valori che si muovono.
- `5d5689e` — demo e UI pura: nuovo `src/text-layout.js` (disposizione dei pezzi, cifre fisse, testo lungo, tempi di
  lampeggio, 100 % righe), `clockParts` in `clock-format.js`, Conta/Rullo/Media e `rollWheels`/`demoRoll` in
  `value-transition.js` (stessa formula del Rust), `demo-render.js` disegna con lo stesso schema e restituisce `textMs`
  (prossimo fotogramma per scorrimento/lampeggio), che `demo-backend.js` aggiunge alle scadenze dell'anteprima.

## Test eseguiti

- `cargo fmt --all -- --check`: verde.
- `cargo test -p bezel-core`: **314 verdi**; `-p bezel-themes`: **80 verdi**; `-p bezel-render`: **108 verdi**
  (tutti i golden esistenti invariati con il nuovo percorso del testo). `cargo clippy -p <crate> --all-targets -D warnings`
  pulito per i tre crate; `cargo check --workspace --all-targets` verde (CLI e Studio compilano).
- Test nuovi Rust: parti di orologio/valore, ruote del contachilometri, Conta, media; andata e ritorno di ogni campo del
  DTO; renderer: larghezza «11» = «88» con Inter sia con `tnum` sia col solo ripiego, parte colorata/ridotta/rialzata
  (cima allineata entro 4 px), parti dell'orologio e lampeggio con/senza movimento, Rimpicciolisci/Taglia/Scorre,
  rullo (inizio = vecchio numero, fine = nuovo, a metà diverso da entrambi e dentro la riga), misura naturale con parti.
- UI unitari: `clock-format`, `value-transition`, `text-layout`, `demo-backend`, `static-guards` — **81/81 verdi**.
- e2e dell'area (`clock`, `fit-content`, `text-case`, `value-color`, `value-inertia`, `value-transition`, `weather`,
  `inspector-trial`, `inspector-preview`): **56 verdi, 28 rossi**. I rossi sono 7 spec × 4 progetti che cercano ancora le
  schede dell'inspector («Look», sezione `transition` visibile) tolte da T-0: **falliscono identiche sulla base `c562d90`**
  senza T-1 (verificato su `dark-en`). Non le ho toccate (G4): vanno migrate a `openSection` — annotato per T-7a/T-10.
- Verifica a vista del demo (Playwright, PNG nella scratchpad): parti, esponente, separatori spenti a metà secondo,
  puntini, rimpicciolimento e scorrimento corretti.

## Differenze e scelte da segnalare

- **Cifre fisse attive di serie anche sui temi esistenti** (D-05 «attiva di serie», piano «omesso se vero»): il file non
  cambia, ma valori/orologi/meteo con font a cifre proporzionali (es. Inter) ora hanno cifre di larghezza uguale e quindi
  possono apparire leggermente più larghi. I testi fissi non cambiano mai (non mostrano numeri che variano).
- **IBM Plex Sans** nel test di larghezza: nel repository c'è solo in `.woff2` (font dell'interfaccia), che il renderer
  non legge; il test usa Inter in due modi (con `tnum` e con il solo ripiego dell'avanzamento uguale). Con IBM Plex Mono
  e JetBrains Mono le cifre sono già uguali.
- Maiuscole/minuscole si applicano pezzo per pezzo: con «Iniziali maiuscole» una parola spezzata fra due parti avrebbe
  due maiuscole (caso teorico, le parti sono numeri e separatori).
- Il rullo usa come testo il numero di arrivo: se cambia il numero di cifre (99 → 100) la colonna in più appare con lo 0
  che gira; da 100 a 99 la colonna sparisce subito.
- Lo scorrimento ignora l'allineamento (parte da sinistra) e riguarda ogni riga troppo lunga separatamente.
- Testo lungo nel player: non toccato (il player ha la sua impaginazione).
- File fuori dall'elenco «Files» della task, toccati in modo minimo: `bezel-core/src/domain/sensor.rs` (divisione
  numero/unità), `bezel-core/src/app/runtime.rs` (`transitioning`), importatori in `bezel-themes/src/import/`, letterali di
  test del renderer, `demo-backend.js` (una voce nelle scadenze). Nuovi: `bezel-render/src/text/rich.rs`,
  `text_parts_tests.rs`, `src/text-layout.js` e test.

## Riuso e duplicazione

Riusati `TextEngine` (risoluzione dei font, cache dei glifi, `Coverage` esteso con il ritaglio per righe),
`paint_for`, `Layer`, `next_change` del renderer, `Transitions`/`Tween`/`ease`, `format_clock`, `applyTextCase`,
`demoSensorValue`, `easedDemoValue`. **Nessuna duplicazione**: il formattatore dell'orologio e quello dei valori hanno
un'unica implementazione da cui derivano testo intero e parti (Rust e JS); la disposizione del demo è un solo modulo puro.

## Per le task successive

- T-7a: controlli Parti (riga Parti + editor della parte), «Cifre a larghezza fissa», «Se il testo non ci sta», Salta/
  Conta/Rullo, Durata, «Media su»: usare `transitionPatch(e, { mode, durationMs, average })`, `averageOf`,
  `TRANSITION_MODES`, `AVERAGE_RANGE`, `partStyle`, `OVERFLOWS`, `VALUE_PARTS`/`CLOCK_PARTS` da `text-layout.js`.
- T-7b: parti dell'orologio digitale e Separatori Fissi/Lampeggiano (`parts.separators.blink`).
- T-3: «valore al centro» della lancetta può riusare `rich_job`/`RichJob` del renderer.
- T-10: migrare le 7 spec e2e ancora basate sulle schede (`text-case`, `value-color` ×2, `value-inertia`,
  `value-transition` ×2, `weather`).
