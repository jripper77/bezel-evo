# T-8 — punti di controllo (attività completata: vedi SUMMARY-T-8.md)

21 ottobre 2026. Branch **ux2/t8**, worktree **F:/Dev/bezel-ux2-t8**, base
`c66a247`. Non unire questo ramo finché non è completata l’intera acceptance T-8.
Non creare un’altra worktree T-8 alla ripresa: proseguire questa.

## Fondazione già verificata

- `src/editor/handles.js`, puro: otto maniglie comuni e ammissibilità della
  rotazione, proporzioni della selezione, proiezione/inversa delle maniglie
  di anello, lancetta, angoli, barra (valore transitorio), grafico, testo,
  spettro e fade. Riusa geometria di rotazione, gauge della lancetta,
  `corner-radii`, spettro e patch dei percorsi esistenti.
- Trascinamento e tastiera delle maniglie di proprietà collegati al canvas;
  le otto maniglie di dimensione sono ora pulsanti nativi utilizzabili con
  le frecce. Shift = passo più grande; Shift sugli archi = 15°; Alt sugli
  angoli modifica un solo angolo. Il trascinamento è un gesto/Undo unico.
- Valore vicino al cursore e campo dell’inspector evidenziato; la sezione del
  campo si apre se chiusa. Anche dimensioni e rotazione mostrano il valore.
  I numeri seguono la lingua UI, con formatter riusati; i valori interi dei
  campi di misura non generano lunghe code decimali.
- Doppio clic sui limiti del grafico riattiva la scala automatica.
- Le maniglie degli angoli piccoli sono distanziate da quelle di dimensione;
  la proiezione sottrae lo spostamento visivo, senza salto al primo movimento.
- Nessun campo tema nuovo e nessun Rust modificato in questo passo.

## Passo successivo completato

- Selezione multipla: otto maniglie sul riquadro comune, ridimensionamento
  proporzionale degli oggetti radice, senza trasformare due volte i discendenti.
  Gli angoli mantengono le proporzioni della selezione. Un trascinamento è un
  solo Undo. I blocchi ereditati sopprimono tutte le maniglie della selezione.
- Rotazione multipla: usa lo stesso `rotateSelection` dell’inspector (stesso
  angolo individuale), con pulsante solo se TUTTI gli oggetti sono ammessi.
  Card/gruppi/membri e finestre video restano esclusi, come D-21.
- Ctrl sulla maniglia SE scala solo il testo, lasciando invariato il riquadro;
  funziona anche da tastiera. La misura separata del testo resta disponibile.
- Testo fisso: doppio clic apre un textarea nativo sul canvas con il contenuto
  corrente; Invio va a capo, Ctrl+Invio applica, Esc annulla, la perdita di focus
  applica. Un solo Undo/Redo per l’intera modifica. Il doppio clic nel textarea
  resta al controllo nativo; gli oggetti bloccati non entrano in modifica.
- Card: linguette con i nomi reali delle facce, cambio faccia e aggiunta con i
  comandi esistenti; focus conservato e Undo. Clic normale sui membri seleziona
  la Card; doppio clic entra nella Card. Esc o il pulsante di uscita ritorna
  alla Card. Dentro, un altro doppio clic sul Testo apre il suo editor.
  I blocchi disabilitano le linguette e impediscono l’ingresso.
- Forma con immagine a dimensione propria (`center/tile/tileX/tileY`): doppio
  clic entra in modifica, trascinamento sposta l’immagine negli assi locali
  anche sulla Forma ruotata; rotella o +/− scalano; frecce spostano (Shift 10).
  Esc/Invio o il pulsante chiudono la modalità. Limiti condivisi con l’inspector
  (5–1000%). Riusa gli offset/scala esistenti e `createBurst` per Undo della
  rotella; il burst termina anche prima dei comandi da puntatore/tastiera.
- Nessun campo tema nuovo, nessun Rust o renderer modificato in questo passo.
  Primitivi riusati: `segmented`, `iconButton`, `resize`, `selectionRoots`,
  `transformCard` tramite `setFrame`, `patchElements`, comandi delle facce,
  geometria della rotazione e tipi di ripetizione già usati dall’inspector.
  Nessuna duplicazione di componenti o implementazione del ritaglio.

## Ulteriore passo completato — orologio e guide

- Orologio analogico: `analog.hourLength`, `minuteLength`, `secondLength`
  facoltativi, moltiplicatori della lunghezza originale (100% = invariata),
  intervallo 10–200%. Il valore originale è omesso in Rust e rimosso anche
  dallo stato JS quando si torna al 100%; nessuna inizializzazione scrive i
  campi. La coda della lancetta dei secondi e gli spessori non cambiano.
- Dominio, DTO, renderer nativo e demo estesi insieme. Prima della modifica
  sono stati catturati i riferimenti pixel originali nei tre stili: sottili,
  arrotondate e freccia. I test verificano quei riferimenti invariati e i pixel
  delle nuove lunghezze; test di andata/ritorno per ogni campo e per i default.
- Tre maniglie a rombo sulle punte, solo per le lancette visibili, con campi
  nell’inspector, trascinamento, tastiera e Undo. Le punte restano all’ora reale:
  vengono riposizionate ai nuovi frame senza ricreare i pulsanti, così il focus
  resta fermo. Nessun timer aggiunto. Il moto ridotto segue la politica esistente.
- Guide rosa: distanza reale dal vicino allineato più prossimo, in pixel del
  documento, anche quando il canvas è ingrandito. Nessuna distanza artificiale
  per oggetti sovrapposti o guide riferite solo al canvas. Spariscono a fine gesto.
- Il campo dell’inspector illuminato viene portato in vista se necessario,
  senza prendere il focus dalla maniglia. `alignmentDistances` è puro, in
  `editor/snap.js`, e non muta i riquadri; nessun calcolo delle distanze se non
  ci sono guide attive. Primitivi riusati, nessuna duplicazione.

## Ripresa di Claude (21 ottobre, dopo `0f90ef6`)

ChatGPT si è fermata al limite di utilizzo; Claude prosegue nella stessa
worktree e sullo stesso ramo. Quattro commit, uno per voce:

- `a17aa9e` **Copertina Player.** `coverFrame` puro in `player-parts.js`, stessa
  regola di `TextFurniture::layout` (quadrato in alto a sinistra, lato =
  min(altezza, 40% larghezza, larghezza − distanza); da sola riempie il riquadro).
  L'editor non ha una sessione dal vivo: si assume una sessione con durata,
  quindi titolo/avanzamento come configurati. Le maniglie della sfumatura
  copertina stanno sul quadrato vero; nuovo **divisore** (punto) per
  `coverGap`, assente con copertina da sola o spenta.
- `f528318` **Ancoraggio Immagine.** Modello rimasto discreto (9 posizioni,
  `center` = assente). La maniglia scatta alla posizione più vicina; con le
  frecce si sposta di una posizione (`keyStep`). Solo con Riempi (`cover`).
  `highlightField` ora trova anche i gruppi di scelta (`role="group"`).
- `965b2a8` **Parti del testo.** Un clic semplice sul Valore o sull'Orologio
  digitale già selezionato sceglie la parte sotto il cursore nella riga
  «Parti» dell'inspector. Pezzi dalla lettura reale dell'ultimo campione
  (`formatValue` con la quantità del catalogo) o dall'ora attuale; geometria
  con lo stesso `fitText` del renderer demo e la misura canvas esportata
  (`canvasMeasure`). Nessun valore d'esempio nel prodotto. `layoutText` porta
  ora l'`id` della parte su ogni frammento. Un'apertura di sezione che ridisegna
  il modulo non perde più il campo da evidenziare.
- `2eec51b` **Barra mobile.** Sopra la selezione, oltre la maniglia di
  rotazione: sensore collegato (apre il selettore dell'inspector), ▶ Prova (la
  prova staccata dell'inspector, non salva nulla; disattivato con movimento
  ridotto), Duplica (comando esistente, un Undo) e ⋯ (stesse voci del menu
  contestuale, `objectMenu.openAt`). Nascosta durante un gesto; con più oggetti
  solo Duplica e ⋯.

Verifiche: unitari mirati verdi (handles, player-parts, text-layout, i18n,
static-guards e affini: 82 + 73), `handles.js` di nuovo **100%
righe/rami/funzioni**. E2E delle sei spec canvas insieme: **88 verdi**
(`target/logs/t8/e2e-claude-bar.log`), nuova `canvas-object-bar.spec.mjs`.
Spec vicine `editing-017`, `object-rotation`, `canvas-wheel-zoom`: 24 verdi,
28 fallimenti tutti preesistenti — i 20 già annotati più i due casi
`editing-017` (campi X e «Individuali» in sezioni chiuse), che falliscono
uguali sulla base `c66a247` (`F:/Dev/bezel/target/logs/t8-baseline/editing-017.log`).
Evidenze it chiaro/scuro in `evidence/t8-bar/` (barra, divisore, parte).

Da annotare per T-10: in italiano il prefisso «distanza» del campo Distanza
del Player (T-7b) restringe l'input fino a nascondere il numero (il valore c'è).

Resta aperto: pulsante **«Valori di prova»** nella barra del canvas
(«Mostra valori di prova invece dei sensori»). Il runtime ha la prova solo per
l'anteprima isolata dell'inspector (`inspect_trial`); per tutta la scena serve
una funzione nuova del runtime: decisione chiesta all'utente. La maniglia
«valore di prova» della Barra dipende da questa scelta.

## Verifiche del nuovo passo

- **139 unitari UI mirati verdi**: handles, clock-format, demo-render-analog,
  demo-backend, cards, editor-geometry, rotation, inspector-model, i18n e static-guards.
  `handles.js`: **100% righe/rami/funzioni**, con soglie Node esplicite.
  Log `target/logs/t8/unit-clock-step-final.log`.
- **68 e2e verdi insieme** nelle cinque spec canvas di T-8: nuovi controlli
  dell’orologio, focus durante i cambi d’ora, pixel legacy/demo/nativi,
  visibilità del campo evidenziato e distanze delle guide, più i gesti già
  esistenti della task. en/pt-BR × chiaro/scuro e axe.
  Log `target/logs/t8/e2e-clock-step-final.log`.
- Rust toccato: `bezel-core` **325 test + 2 doc test**; `bezel-themes` **95**;
  `bezel-render` **138** (137 verdi nella prima esecuzione e il solo nuovo test
  pixel corretto/rieseguito verde). La prima posizione del campione era sul
  bordo antialias del cappuccio arrotondato: spostata oltre il cappuccio,
  tolleranze conservate. Nessuna suite ampia rilanciata per quel test.
  Timing release, corpus esterno e un test nativo già ignorato restano ignorati.
- Clippy `-p bezel-core -p bezel-themes -p bezel-render --all-targets -- -D warnings`
  verde; fmt e diff check verdi. Log `analog-rust-test.log`,
  `analog-pixels-retry.log`, `analog-themes-test.log`, `analog-rust-doc-test.log`,
  `analog-timing-test.log`, `analog-clippy.log` nella cartella della task.
- Riferimenti precedenti catturati da renderer invariati: `analog-baseline.log`
  e `analog-demo-baseline.log`. Nessun accesso a temi o applicazioni personali.
- Evidenza it chiaro/scuro aggiornata in `evidence/t8-clock/`: due stati, otto
  PNG (intero e dettaglio), con l’inspector portato al campo della lunghezza.
  È ancora un confronto parziale: non chiude l’accettazione finale T-8.

## Commit

- `a138a5c`: motore puro e test delle maniglie.
- `a251ded`: collegamento canvas/inspector, tastiera, i18n ed e2e.
- `9456d79`: lingua UI per i numeri vicino al cursore.
- `bbf4424`: checkpoint iniziale verificato.
- `48031ec`: selezione multipla e rotazione condivisa.
- `1551269`: modifica diretta del Testo e Ctrl-ridimensionamento.
- `a06f8b1`: facce Card e ingresso nei membri.
- `2e0a040`: spostamento/scala dell’immagine nelle Forme.
- `fa71016`: checkpoint del passo precedente.
- `aa2e0cc`: lunghezze analogiche, renderer, maniglie e test.
- `966aec8`: guide con distanza e campo evidenziato portato in vista.

## Verifica del passo successivo

- **85 unitari mirati verdi**, inclusi handles, cards, editor-geometry, rotation,
  i18n e static-guards. I 12 test del solo `handles.js` mantengono **100% righe,
  rami e funzioni**, con soglie tutte a 100 e percorso incluso esplicitamente.
  Log `target/logs/t8/unit-step-final.log`.
- **52 e2e verdi insieme**, quattro spec nuove di T-8 in en/pt-BR × chiaro/scuro:
  `canvas-property-handles`, `canvas-selection-handles`, `canvas-card-handles`,
  `canvas-shape-image`. Compresi accessibilità, tastiera/focus, Undo/Redo,
  blocchi ereditati, trasformazione singola dei discendenti, rotazione esclusa
  e offset dell’immagine negli assi locali a 0°/90°.
  Log `target/logs/t8/e2e-step-final.log`.
- `git diff --check` e sintassi JS verdi. Nessuna suite completa, verifica
  Cargo non pertinente (nessun Rust toccato), build prodotto o push.
- Evidenza italiana chiaro/scuro dei quattro stati nuovi (16 PNG: schermata
  e dettaglio per stato/tema), ricette ripetibili e motivi delle differenze in
  `evidence/t8-selection-text/README.md`. È un confronto parziale, non chiude G3.

## Verifica precedente e questioni ancora da registrare a T-10

- 10 test `handles.test.mjs` verdi; **100% righe, rami e funzioni** del solo
  `src/editor/handles.js`, con soglie Node impostate tutte a 100.
- 66 unitari mirati verdi: i18n, static-guards, rotation, editor-geometry,
  inspector-model. Le successive 51 verifiche i18n/statiche sono un sottoinsieme.
- Nuova `canvas-property-handles.spec.mjs`: 12 casi distinti verificati
  (en/pt-BR × chiaro/scuro), compresi tastiera, focus, Undo e axe.
  Nell’ultima esecuzione un caso light-en si è bloccato **prima** di provare
  le maniglie: il selettore Schermo della topbar intercettava il clic Aggiungi
  durante il layout iniziale. Quel solo caso è stato rieseguito isolato ed è
  verde; è verde anche il successivo controllo mirato dopo la correzione della
  lingua dei numeri. Non è stata indebolita alcuna asserzione.
- Spec preesistenti `object-rotation` e `canvas-wheel-zoom`: 20 casi verdi
  (zoom, pan e parità pixel/anteprima), 20 fallimenti da registrare a T-10:
  i casi di rotazione cercano il campo in Disposizione senza aprirla; il caso
  di inquadratura cerca il pulsante nel nuovo Sfondo chiuso. Due casi
  rappresentativi, `object-rotation:21` e `canvas-wheel-zoom:140`, falliscono
  anche su **windows-improvements/c66a247**, senza questo lavoro T-8.
  Nessuna modifica a queste spec, come prescritto da G4.
- `git diff --check` verde. Nessuna suite completa, build prodotto o push.

Log della worktree: `target/logs/t8/`. Confronto della base integrata:
`F:/Dev/bezel/target/logs/t8-baseline/e2e.log`. Artefatti di test e screenshot
sono già raggruppati sotto la cartella della task; nessuna pulizia di artefatti
precedenti, applicazioni installate o dati utente.

## Confronto visivo parziale

Letti i PNG CanvasHandles e MainNew; aperti entrambi gli HTML offline in
Chromium. Ricetta `shell/ux2-handles-ring`, italiano chiaro/scuro, in
`evidence/t8-checkpoint/`: mostra una manipolazione reale dello spessore,
lettura accanto al cursore e campo evidenziato. **Non è ancora il confronto
finale di tutti i widget richiesto da T-8.**

## Da completare prima del merge

1. Barra mobile: sensore reale, ▶ Prova, duplica, menu condiviso; pulsante
   Valori di prova nella barra del canvas. Riutilizzare il bridge di anteprima
   T-0, `createInspectorTrial` e `preview-animation`; la prova non deve salvare
   nulla. Il motore puro restituisce `trial` per la barra, ma l’adattatore del
   canvas per ora la filtra: non c’è un controllo finto o inattivo.
   Attenzione: `inspector_trial::Trial::new` isola l’oggetto e la sua curva
   parte dalla lettura attuale, non dal minimo. Non usare l’inversa della
   curva come se garantisse il valore 0–100% richiesto dalla maniglia Barra.
2. Immagine: maniglia di ancoraggio e verifica del modello oggi discreto.
   La modifica dell’immagine nelle Forme è ora completata (vedi sopra).
3. Player: fornire al motore puro il **vero rettangolo della copertina**
   (`coverFrame`) e collegare fade/divisore. Non usare l’intero riquadro del
   Player come rettangolo della cover. I campi coverFade e coverGap già esistono.
4. Testo/valore: clic sulla parte per selezionarla nell’inspector. Ctrl-SE e
   modifica inline del Testo sono ora completati. Verificare tutte le parti
   con le geometrie reali, senza valori di esempio introdotti nel prodotto.
5. Completare focus/letture dei controlli per tutti i tipi e il confronto
   visivo completo scuro/chiaro. Le guide con distanza sono ora completate.
6. Rieseguire solo i test delle aree ulteriormente toccate, scrivere
   SUMMARY-T-8.md finale, aggiornare Status e unire con --no-ff solo alla fine.

Le zone della lancetta condividono lo stesso confine tra gli intervalli
adiacenti; le operazioni pure non mutano il documento originale. Le prove
transitorie e i rettangoli reali del rendering restano distinti dalle patch.
