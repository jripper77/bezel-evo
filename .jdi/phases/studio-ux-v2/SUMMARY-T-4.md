# T-4 — Icone meteo animate

Data: 20 ottobre 2026. Branch `ux2/t4`, worktree `F:\dev\bezel-ux2-t4`, partenza `a02a07f` (T-1…T-3 unite).
Decisione: D-2026-10-20-studio-ux-v2-09. Tavole: WeatherAnimations, WeatherInspectorOpen (Comportamento);
specifica `design/weather-motion.md`.

**T-4 completata e verificata**, pronta per il merge `--no-ff` in `windows-improvements`.
Motore, file del tema e demo; i controlli (Ferma/Leggera/Vivace, velocità) sono di T-7b.

## Fatto

- `59bcec5` — dominio, file del tema, renderer:
  - `Weather.icon_motion` (`still`/`gentle`/`lively`, predefinito `still` per i temi esistenti) e `icon_speed` (0,5–2).
    DTO `iconMotion`/`iconSpeed` facoltativi, omessi a `still`/1; velocità riportata nei limiti; valori sconosciuti rifiutati.
  - Funzione pura `pose(famiglia, t, ampiezza, velocità) → IconPose` che segue `weather-motion.md` per le otto famiglie:
    raggi che ruotano (24 s) e disco che pulsa (4 s, curva morbida), nuvole che ondeggiano (7 s), strisce di nebbia
    sfasate (5+i s, posizione e opacità), gocce (1 s, sfasate 0/0,33/0,66, lineari), fiocchi (3 s, tre punti), lampi del
    temporale ai tempi relativi della specifica, «non disponibile» che respira (0,45–0,85). Leggera dimezza spostamenti e
    variazione d'opacità attorno al valore medio; la velocità divide i periodi. `soft` = cubic-bezier(0.45,0,0.55,1).
    Test di periodicità: `pose(t) = pose(t + periodo)` per ogni famiglia e velocità.
  - `weather.rs` costruisce le icone a parti (disco, raggi, nuvola, strisce, gocce/fiocchi, fulmine, croce).
    **A riposo l'icona è disegnata esattamente come prima** (stesse primitive unite: i golden del meteo passano invariati).
    In movimento ogni parte prende la sua trasformazione e opacità; il 3D aggiunge l'ombra ellittica a terra che segue la
    nuvola, l'alone del sole (0,10→0,30) e il lampo sul riquadro (0,12 col fulmine).
  - Renderer: dissolvenza incrociata di 300 ms al cambio di famiglia (stato per scena ed elemento), fotogramma ogni
    33 ms (Vivace) o 50 ms (Leggera) solo mentre l'icona si muove ed è disegnata; con «movimento ridotto» l'icona è a
    riposo e non chiede fotogrammi.
- `7715dc3` — demo: `weather-icon.js` espone le parti (`weatherIconParts`; `weatherIconPaths` ne deriva e dà la stessa
  uscita di prima, verificato), nuovo `weather-motion.js` (specchio di `pose`), `demo-render.js` disegna le parti in
  movimento con ombra, alone e lampo e chiede i fotogrammi. I nuovi oggetti meteo nascono con `iconMotion: 'gentle'`
  (`editor/widgets.js`).

## Test eseguiti

- `cargo fmt --all -- --check` verde; `cargo check --workspace --all-targets` verde; clippy `-D warnings` pulito su core,
  themes, render.
- `cargo test`: bezel-core **325**, bezel-themes **87**, bezel-render **122**, bezel-runtime **58**, verdi.
- Nuovi Rust: periodicità e valori chiave della posa per famiglia/ampiezza/velocità; DTO; renderer: icona ferma senza
  fotogrammi, ogni famiglia si muove in ogni stile restando visibile, passo 33/50 ms, movimento ridotto = riposo,
  dissolvenza (a metà diversa, dopo 300 ms uguale all'icona nuova), lampo 3D sul riquadro.
- UI unitari (`weather-motion` 98,6 % righe, `weather-icon` 100 %, `inspector-trial`, `demo-backend`, `static-guards`,
  test con `createWidget`): **117/117** verdi.
- e2e `inspector-trial`, `inspector-preview`: **36/36**. `weather.spec` è fra quelle già rosse dalla base (schede, T-10).
- Verifica a vista del demo nei quattro stili a tre istanti: movimento e ombra corretti.

## Differenze e scelte da segnalare

- **Ferma = icona di sempre**, non «posa a t = 0» della specifica: così i temi esistenti restano identici (la posa a
  t = 0 sposterebbe la nuvola di 3 unità). Ombra 3D, alone e lampo compaiono solo in movimento.
- Le icone statiche esistenti (24 unità, «non rifarle») hanno meno parti dei riferimenti a 64 unità: nuvoloso ha una
  nuvola sola (la nuvola posteriore del riferimento non c'è), la nebbia due strisce invece di quattro, «non disponibile» è
  la croce (respira come la nuvola tratteggiata del riferimento). I movimenti sono applicati alle parti che ci sono, con
  gli spostamenti scalati da 64 a 24 unità.
- La nuvola, spostandosi a sinistra fino a 3/64 del lato, può uscire di poco dal riquadro dell'icona quando l'icona
  tocca il bordo dell'oggetto (lì viene tagliata).
- Il demo non fa la dissolvenza al cambio di famiglia: la condizione del demo è fissa (cambia solo nella prova ▶).
- File fuori dall'elenco «Files»: `weather-icon.js`, `editor/widgets.js` (predefinito `gentle`), nuovi
  `weather-motion.js`, `weather_motion_tests.rs`.
- Nota operativa: durante un confronto ho usato `git stash` in una worktree pulita e `git stash pop` ha ripreso uno stash
  vecchio e condiviso («WIP on fid/t7»), andando in conflitto. Ho riportato la worktree a HEAD senza perdere nulla:
  lo stash è ancora in `git stash list`, intatto.

## Riuso e duplicazione

Riusate le geometrie e le palette statiche esistenti (ora come parti), `paint_for`, `solid`, `Layer::blit`, la dissolvenza
`faded` (una sola funzione per parti e cambio di famiglia), `next_change` del renderer. **Nessuna duplicazione**:
`weatherIconPaths` deriva da `weatherIconParts`; Rust e JS hanno ciascuno una sola funzione di posa.

## Per le task successive

- T-7b: Comportamento del Meteo: Ferma/Leggera/Vivace (`ICON_MOTIONS`), velocità (`ICON_SPEED`).
- T-9d: i temi predefiniti possono usare `iconMotion`.
