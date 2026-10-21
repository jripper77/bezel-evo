# T-8 — punto verificato, attività ancora in corso

21 ottobre 2026. Branch **ux2/t8**, worktree **F:/Dev/bezel-ux2-t8**, base
`c66a247`. Non unire questo ramo finché non è completata l’intera acceptance T-8.
Non creare un’altra worktree T-8 alla ripresa: proseguire questa.

## Passo completato

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

## Commit

- `a138a5c`: motore puro e test delle maniglie.
- `a251ded`: collegamento canvas/inspector, tastiera, i18n ed e2e.
- `9456d79`: lingua UI per i numeri del valore accanto al cursore.

## Verifica di questo passo

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
2. Ridimensionamento comune della selezione: collegare `frameHandles` e
   `scaledFrames` al canvas, con Undo unico, limiti e blocchi. Rotazione solo
   se tutti gli oggetti sono ammessi (D-21); niente rotazione di Card/gruppi/membri.
3. Orologio analogico: le lunghezze delle lancette sono ancora costanti nei
   renderer. Includere i campi mancanti in questa attività (D-20), facoltativi
   e omessi ai valori predefiniti, con test legacy/pixel nativi e demo.
4. Immagine: maniglia di ancoraggio e verifica del modello oggi discreto;
   Forma: doppio clic per spostare/scalare l’immagine di riempimento.
5. Player: fornire al motore puro il **vero rettangolo della copertina**
   (`coverFrame`) e collegare fade/divisore. Non usare l’intero riquadro del
   Player come rettangolo della cover. I campi coverFade e coverGap già esistono.
6. Testo/valore: Ctrl sulla maniglia per scalare il testo, clic sulla parte
   per selezionarla nell’inspector, doppio clic per editare il testo fisso.
   La maniglia separata di dimensione testo esiste già; non copre quella SE
   di ridimensionamento del riquadro.
7. Card: linguette delle facce, aggiunta faccia e ingresso con doppio clic.
8. Guide rosa con distanza; completare focus/letture dei controlli per tutti
   i tipi, prove di oggetti ruotati e confronto visivo completo scuro/chiaro.
9. Rieseguire solo i test delle aree ulteriormente toccate, scrivere
   SUMMARY-T-8.md finale, aggiornare Status e unire con --no-ff solo alla fine.

Le zone della lancetta condividono lo stesso confine tra gli intervalli
adiacenti; le operazioni pure non mutano il documento originale. Le prove
transitorie e i rettangoli reali del rendering restano distinti dalle patch.
