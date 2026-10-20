# T-2 — Orologio analogico

Data: 20 ottobre 2026. Branch `ux2/t2`, worktree `F:\dev\bezel-ux2-t2`, partenza `991881d` (T-1 unita).
Decisione: D-2026-10-20-studio-ux-v2-07. Tavola: ClockInspectorOpen (colonna Analogico).

**T-2 completata e verificata**, pronta per il merge `--no-ff` in `windows-improvements`.
Motore, file del tema e demo; i controlli dell'inspector (Forma/Colore/Comportamento analogici) sono di T-7b.

## Fatto

- `27ad3f0` — dominio, file del tema, renderer:
  - `TextContent::Clock.analog: Option<AnalogClock>` (lo schema `pattern` resta, per tornare al digitale).
    `AnalogClock`: tacche (nessuna/ore/ore e minuti), numeri (nessuno, 3-6-9-12, 1–12, romani), lancette ore/minuti/
    secondi visibili, forma (sottili/arrotondate/a freccia), bordo in px, colori di quadrante, tacche e numeri, e di ogni
    lancetta, secondi a scatti o fluidi, data (nessuna/giorno/giorno e mese, nella lingua dell'orologio).
    Predefiniti = anteprima della tavola (tacche delle ore, tre lancette arrotondate, bordo 2, #0B0D0F/#AEB5BD/#E8EAED/accento).
  - Funzioni pure: `angles` (ore con minuti e secondi, minuti con i secondi, secondi a scatti o fluidi),
    `next_change` (prossimo secondo; 33 ms se fluidi; nessuno senza lancetta dei secondi), `date_text`, `numerals`.
  - `LocalTime.millisecond` (nuovo; Studio e CLI lo leggono da chrono, gli altri costruttori mettono 0) per i secondi
    fluidi allineati al PC. Il riuso del fotogramma nel runtime confronta il secondo intero (`whole_second`), così il
    riuso continua a funzionare; resta limitato da `next_change`.
  - DTO `analog`: oggetto con campi tutti facoltativi, omessi al predefinito (un quadrante predefinito è `{}`); valori
    sconosciuti rifiutati, bordo limitato a 0–64 px. Un orologio digitale non scrive nulla di nuovo.
  - `bezel-render/src/analog_clock.rs`: quadrante inscritto nel riquadro (scala del quadrante di 64 unità della tavola),
    bordo nel colore delle tacche al 30 %, tacche, numeri e data col carattere dell'elemento, lancette (la sottile al
    60 % e a punta piatta, la freccia con punta triangolare), mozzo nel colore della lancetta più in alto.
    Fotogrammi: un secondo con la lancetta dei secondi, 33 ms se fluida; con «movimento ridotto» i secondi vanno a scatti.
- `34f1aa5` — demo: `analogOf`, `handAngles`, `dialDue`, `dialNumerals`, `dialDate` in `clock-format.js` (puri, testati)
  e `drawDial` in `demo-render.js` con le stesse misure; `textMs` chiede i fotogrammi come il renderer. Numeri e data
  rimpiccioliti e data più vicina al centro (prima si sovrapponeva al 6), uguale nel renderer.

## Test eseguiti

- `cargo fmt --all -- --check` verde; `cargo check --workspace --all-targets` verde (CLI, Studio, media compilano).
- `cargo test`: bezel-core **317**, bezel-themes **83**, bezel-render **112**, bezel-runtime **58**, tutti verdi;
  clippy `-D warnings` pulito su core, themes, render.
- Nuovi: geometria delle lancette a **3:00, 6:30, 9:45:30** (angoli nel dominio e punte/coda in pixel), secondi fluidi e
  a scatti, prossimo fotogramma; pixel del quadrante (colori di lancette, faccia, mozzo, angoli fuori dal cerchio),
  lancetta dei secondi nascosta, tacche/minuti, numeri 3-6-9-12/1–12/romani, data giorno/giorno e mese, bordo più
  tenue, tre forme di lancetta; DTO: assenza, `{}`, ogni campo andata e ritorno, predefiniti omessi, valori rifiutati.
- UI unitari (`clock-format`, `demo-backend`, `static-guards`): **63/63**. e2e `clock`, `fit-content`,
  `inspector-trial`: **20/20**.
- Verifica a vista del demo (quattro quadranti diversi): corretta e leggibile.

## Differenze e scelte da segnalare

- La tavola non dice il colore del bordo: uso il colore di «Tacche e numeri» al 30 % (nell'anteprima #3A4149 su
  #0B0D0F, che è circa questo). Nessun campo colore in più.
- Posizione della data: sotto il centro, dentro i numeri (la tavola non la mostra).
- Il mozzo prende il colore della lancetta dei secondi (come nell'anteprima), o dei minuti/ore se quella è nascosta.
- Adatta al contenuto: un orologio analogico non ha una misura naturale (come prima, il riquadro resta).
- `LocalTime.millisecond` tocca molti file (costruttori di test in CLI, Studio, media, runtime): una riga ciascuno.

## Riuso e duplicazione

Riusati `format_clock` (data), `TextEngine::draw` (numeri e data), `solid`, `polar`, `Layer`, `next_change` del renderer
(`text_due` di T-1), `set_motion`. **Nessuna duplicazione**: angoli, numeri, date e scadenze hanno una sola
implementazione per lato (Rust in `clock.rs`, JS in `clock-format.js`), il disegno usa quelle.

## Per le task successive

- T-7b: Stile Digitale/Analogico (= `analog` assente/presente; tornando al digitale si può tenere l'oggetto per non
  perdere le scelte, decisione dell'inspector), Tacche, Numeri, Lancette, Forma, Bordo, colori Quadrante/Tacche e numeri/
  Lancette, «Lancetta dei secondi» a scatti/fluida (`sweep`), «Data nel quadrante». Usare `analogOf` per i predefiniti.
- T-8: maniglie dell'orologio analogico (CanvasHandles) con `Dial`/`hand_ends` come riferimento geometrico.
