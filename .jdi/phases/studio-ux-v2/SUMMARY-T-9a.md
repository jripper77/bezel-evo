# T-9a — Pannelli Widget, Sensori, Livelli, Media — riepilogo

21 ottobre 2026, Claude. Branch `ux2/t9a`, worktree `F:/dev/bezel-ux2-t9a`,
base `c694a83` (T-8 integrata). Decisione D-2026-10-20-studio-ux-v2-14,
tavole LibraryPanelsNew e MainNew (pannello sinistro).

## Cosa è fatto

- **Widget**: quattro gruppi apribili Valori (5) · Tempo e media (4, con lo
  Spettro) · Testo e forme (3) · Contenitori (2), conteggio sempre visibile,
  stato ricordato; una ricerca apre i gruppi che trova. Intestazione «14 tipi»,
  segnaposto «Cerca: anello, ora, meteo…».
- **Sensori**: categorie apribili (processore e grafica aperte all'inizio) con
  icona, conteggio e valori principali nel titolo (prima percentuale e prima
  temperatura leggibili, altrimenti la prima lettura; nessuno per Sistema),
  aggiornati a ogni campione. Righe su due livelli: nome intero + valore, poi
  chiave + sparkline; se non leggibile «—» e il motivo («non disponibile ·
  richiede LibreHardwareMonitor») al posto della chiave. Chip rapidi Tutti /
  Disponibili / Non disponibili · N. Sonda e Stato dietro il pulsante Filtri
  accanto alla ricerca (si apre da solo quando lo stato dei sensori apre la
  libreria già filtrata). Intestazione «23 · ogni 1 s». Nota in fondo aggiornata.
  **Nomi interi a 240, 272 e 360 px** (test `scrollWidth ≤ clientWidth`).
- **Livelli**: conteggio accanto al titolo, tipo di oggetto dietro lo stesso
  pulsante Filtri (con il puntino quando filtra), base della Card «sempre
  visibile», facce con il numero di oggetti, barra di selezione con Raggruppa,
  Crea card e **Allinea** (sei allineamenti e due distribuzioni, la stessa lista
  dell'inspector) su una riga, nota in fondo. Occhio/lucchetto al passaggio e
  arancioni se attivi, linea di inserimento: già presenti.
- **Media**: totale file · dimensione nell'intestazione, schede con icone,
  riquadro compatto con un solo «+» (Immagine… / Video…), sezione «Nel tema»
  apribile con il conteggio, **«in uso: …» su ogni file** calcolato dai
  riferimenti reali del tema, **cestino solo sui non usati**, azioni al
  passaggio o col focus, nota, righe «Dalla collezione» e «Icone».
- Correzione di T-8: la barra mobile scende sotto la selezione quando sopra
  c'è la barra degli strumenti dello stage.

## Commit

`b49ca8f` barra mobile · `5a15fc0` Widget · `a9de5f3` Sensori ·
`d17e4c5` + `02dd3d9` Livelli · `2e4e172` Media · più il commit di questo
riepilogo.

## Primitivi riusati, nessuna duplicazione

`accordionSection` (esteso: senza icona, riassunto visibile da aperto) e
`createSectionMemory` (esteso: identificativi, aperture e chiave propri del
pannello) per tutti i gruppi apribili; `chipButton`, `selectField`,
`openMenuAt`, `iconButton`; `ALIGN_EDGES`/`alignItems` spostati in
`ui/object-menu.js` e usati da inspector e Livelli; un solo `filtersToggle`
per Sensori e Livelli; `assetUses` (puro, rispecchia `Theme::assets` del core)
per badge e cestino. Icone nuove `filter`, `star`, `test-values` in
`ui/icons.js`. Chiavi riusate dove il testo esiste già (`insp.theme.every`,
`shell.memoryUnavailable`, `inspector.alignTools`, `common.background`).
`index.html` non è stato toccato: conteggi, pulsanti e note sono creati da
`ui/library.js` nei contenitori esistenti.

## Test eseguiti

- Unitari UI: 660/661; l'unico fallimento (`video-framing.test.mjs`) è uguale
  sulla base. `library-model.js` e `inspector-sections.js` al 100% delle righe.
- E2E area: `library` (52), `libre`, `profiles`, `canvas-object-bar`,
  `video-preview` insieme 108 verdi; l'unico caso rosso,
  `video-preview:38` (associazione dall'Archivio), fallisce uguale sulla base.
- Confronto G3: `evidence/t9a/` (Widget, Sensori, Livelli con selezione, Media
  pieno) in italiano chiaro e scuro, confrontate con LibraryPanelsNew e MainNew.

## Spec adeguate e perché

Nell'area della task: `library.spec` (gruppi, intestazioni, Filtri, Allinea,
Media). Fuori area, solo dove la mia modifica cambiava il controllo usato, senza
indebolire le verifiche: `libre.spec` (filtro categoria → ricerca, filtri
dietro il pulsante), `profiles.spec` (apre la categoria Sistema invece del chip),
`video-background.spec` e `video-preview.spec` («+» di Media e azioni al
passaggio), ricette `fidelity/recipes/{library,inspector}.mjs`. Nuovo aiuto
`addMedia()` in `tests/e2e/helpers.mjs`.

## Differenze rimaste rispetto alle tavole

- La freccia dei gruppi è a destra come nell'inspector (la tavola Widget la
  mette a sinistra): un solo componente.
- Livelli conserva Espandi/Comprimi tutto e le frecce su/giù (al passaggio),
  per la tastiera; la tavola non li mostra.
- Media conserva il pulsante «Cerca GIF e sticker… KLIPY» in fondo; le righe
  verso Collezione e Icone si aggiungono senza un conteggio delle icone
  (il catalogo si carica solo aprendo la scheda).

## Da annotare per T-10

Fallimenti preesistenti, uguali sulla base: `background-chooser` (10 casi,
pulsante «Cambia sfondo…» dell'inspector del tema), `video-background` 3 casi
(stesso riquadro), `video-preview:38`, `clipboard`, `editing-017` (2),
`icons` (2), `inspector-components:47`, `inspector-preview:259`,
`layer-drag` (2), `scenarios-shell` (5). Log in `target/logs/t9a/` e
`F:/dev/bezel/target/logs/t8-baseline/`.
