# T-8 — punto verificato, attività ancora in corso

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

## Commit

- `a138a5c`: motore puro e test delle maniglie.
- `a251ded`: collegamento canvas/inspector, tastiera, i18n ed e2e.
- `9456d79`: lingua UI per i numeri vicino al cursore.
- `bbf4424`: checkpoint iniziale verificato.
- `48031ec`: selezione multipla e rotazione condivisa.
- `1551269`: modifica diretta del Testo e Ctrl-ridimensionamento.
- `a06f8b1`: facce Card e ingresso nei membri.
- `2e0a040`: spostamento/scala dell’immagine nelle Forme.

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
2. Orologio analogico: le lunghezze delle lancette sono ancora costanti nei
   renderer. Includere i campi mancanti in questa attività (D-20), facoltativi
   e omessi ai valori predefiniti, con test legacy/pixel nativi e demo.
3. Immagine: maniglia di ancoraggio e verifica del modello oggi discreto.
   La modifica dell’immagine nelle Forme è ora completata (vedi sopra).
4. Player: fornire al motore puro il **vero rettangolo della copertina**
   (`coverFrame`) e collegare fade/divisore. Non usare l’intero riquadro del
   Player come rettangolo della cover. I campi coverFade e coverGap già esistono.
5. Testo/valore: clic sulla parte per selezionarla nell’inspector. Ctrl-SE e
   modifica inline del Testo sono ora completati. Verificare tutte le parti
   con le geometrie reali, senza valori di esempio introdotti nel prodotto.
6. Guide rosa con distanza; completare focus/letture dei controlli per tutti
   i tipi, prove di oggetti ruotati e confronto visivo completo scuro/chiaro.
7. Rieseguire solo i test delle aree ulteriormente toccate, scrivere
   SUMMARY-T-8.md finale, aggiornare Status e unire con --no-ff solo alla fine.

Le zone della lancetta condividono lo stesso confine tra gli intervalli
adiacenti; le operazioni pure non mutano il documento originale. Le prove
transitorie e i rettangoli reali del rendering restano distinti dalle patch.
