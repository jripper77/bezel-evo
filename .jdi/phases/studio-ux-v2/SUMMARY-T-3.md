# T-3 — Lancetta estesa

Data: 20 ottobre 2026. Branch `ux2/t3`, worktree `F:\dev\bezel-ux2-t3`, partenza `1804ad2` (T-1 e T-2 unite).
Decisione: D-2026-10-20-studio-ux-v2-08. Tavole: NeedleInspectorOpen, CanvasHandles (scheda Lancetta).

**T-3 completata e verificata**, pronta per il merge `--no-ff` in `windows-improvements`.
Motore, file del tema e demo; i controlli sono di T-7a, il trascinamento dei confini sul canvas di T-8.

## Fatto

- `6110e51` — dominio, file del tema, renderer:
  - Nuovo `bezel-core/src/domain/needle.rs`: `NeedleLook` facoltativo sulla lancetta (`ElementKind::Needle.look`).
    **Senza `look` la lancetta si disegna esattamente come prima** (linea dal perno al bordo): i temi esistenti non cambiano.
    Con `look`: arco (spessore, estremità dritte/arrotondate), arco di fondo Unico/Zone/Sfumatura, zone
    `{ from, to, color }` nelle unità del sensore (max 8), colore della lancetta dalla zona, tacche principali (ogni 10 %)
    e piccole (ogni 2 %), numeri della scala (nessuno/inizio e fine/metà/a ogni tacca) con distanza e formattati con la
    lingua (virgola decimale in it e pt-BR), forma Sottile/Affusolata/Freccia/A punto, lunghezza e coda in % del raggio,
    cappuccio Nessuno/Pieno/Anello con diametro e colore, valore al centro con le opzioni del Valore (stile completo con
    parti e cifre fisse, formato, prima/dopo), segno del massimo con ritorno dopo N s.
  - Funzioni pure testate: `share`, `zone_at`, `needle_color`, `gradient_at`, `tick_shares`, `number_shares`,
    `scale_number`, `Peak` (stato del segno del massimo: sale col valore, resta per N s, poi torna al valore).
  - DTO `look`: campi facoltativi omessi al predefinito (un look predefinito è `{}`), valori limitati (lunghezza 10–150 %,
    coda 0–100 %, secondi 1–3600, zone tagliate a 8), valori sconosciuti rifiutati.
  - Renderer: nuovo `needle_gauge.rs`. L'arco ha il raggio massimo che il riquadro permette attorno al perno per
    l'ampiezza scelta (`Gauge::fit`); tacche, numeri, segno del massimo (lancetta tratteggiata tenue), lancetta, cappuccio.
    Il valore al centro passa per lo stesso `rich_job`/`draw_rich` del Valore (T-1): parti, cifre fisse, Rullo/Conta.
    Lo stato del segno del massimo è nel renderer per scena ed elemento (come i rotori di `sensor_motion`) e chiede un
    fotogramma quando deve ricadere. Con un'immagine come lancetta arco, scala e cappuccio restano e l'immagine sostituisce
    la lancetta disegnata. Numeri della scala con la stessa funzione di scrittura dell'orologio analogico.
- `3f7234a` — demo: `src/needle-look.js` (predefiniti, zone, sfumatura, tacche, numeri, `fitGauge`, picco del demo; 100 %
  righe) e disegno in `demo-render.js` con le stesse misure; il valore al centro usa il disegno del testo del Valore.

## Test eseguiti

- `cargo fmt --all -- --check` verde; `cargo check --workspace --all-targets` verde.
- `cargo test`: bezel-core **323**, bezel-themes **86**, bezel-render **117**, bezel-runtime **58**; clippy `-D warnings`
  pulito su core, themes, render.
- Nuovi Rust: dominio (quote, zone e confini, colore dalla zona, sfumatura, tacche, numeri, formattazione per lingua,
  stato del massimo); DTO (assenza, `{}`, ogni campo, limiti, rifiuti); geometria dell'arco (semicerchio, perno in basso,
  arco che scende sotto il perno, solo angoli, verso antiorario); pixel: colori delle zone sull'arco, lancetta dal colore
  della zona e dal proprio colore, arco unico tenue, sfumatura, cappuccio pieno/anello/nessuno, tacche principali e
  piccole, numeri, quattro forme diverse, valore al centro, segno del massimo che resta e poi ricade con `next_change`,
  lancetta vecchia invariata.
- UI unitari (`needle-look`, `demo-backend`, `static-guards`): **66/66**.
- e2e `inspector-layout`, `inspector-redesign`, `inspector-trial`, `library`: **76 verdi, 4 rossi**: `library.spec`
  «the Group tile…» cerca `#inspector h2`, che T-0 ha sostituito con l'intestazione nuova: **rosso anche sulla base
  `1804ad2`** (verificato su `dark-en`). Annotato per T-10.
- Verifica a vista del demo (cinque lancette diverse, compresa una senza `look`): corretta.

## Differenze e scelte da segnalare

- La tavola non ha un colore per l'arco «Unico»: uso «Tacche e numeri» al 30 % (stessa scelta del bordo dell'orologio).
- Il segno del massimo è tratteggiato nel colore delle tacche al 35 % (nell'anteprima #E8EAED tratteggiato .35).
- Posizione del valore al centro: sotto il perno, centrato, alto 1,3 volte la sua dimensione.
- Numeri della scala nel carattere del valore al centro se c'è, altrimenti Inter; dimensione 12 % del raggio (8–64 px).
- Il segno del massimo nel demo è il valore più alto dei campioni degli ultimi N s (il renderer tiene lo stato vero);
  il comportamento a vista è lo stesso.
- File nuovi fuori dall'elenco «Files» (gauges.rs non è stato toccato): `domain/needle.rs`, `needle_gauge.rs`,
  `needle_look_tests.rs`, `src/needle-look.js` e test. `analog_clock::write` reso condiviso.

## Riuso e duplicazione

Riusati `rich_job`/`draw_rich` (valore al centro), `analog_clock::write` (numeri), `push_arc`, `polar`, `solid`,
`Cap`, `TextStyle`/`DisplayFormat` e i loro DTO, il valore mostrato delle transizioni (`ShownValues`), `next_change`.
**Nessuna duplicazione** fra Rust e JS oltre allo specchio voluto del demo.

## Per le task successive

- T-7a: sezioni Forma/Colore/Comportamento della lancetta come la tavola, usando `lookOf` e le liste `TRACKS`,
  `SCALE_TICKS`, `SCALE_NUMBERS`, `NEEDLE_SHAPES`, `PIVOT_CAPS`; attivare il `look` quando si tocca una funzione nuova
  (un oggetto nuovo può nascere con `look: {}`).
- T-8: confini delle zone trascinabili sull'arco: `fitGauge`/`Gauge::fit` danno raggio e angoli.
- T-10: migrare `library.spec` («Group tile», `#inspector h2`).
