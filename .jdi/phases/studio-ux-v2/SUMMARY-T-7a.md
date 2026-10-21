# T-7a — Inspector dei widget collegati

Completata il 21 ottobre 2026 nella worktree `F:\Dev\bezel-ux2-t7a`, branch `ux2/t7a`,
partendo da `f603233` (T-6 integrata). Nessun push, build del prodotto, cambio di
versione, setup o installazione; `dist/` invariata.

## Cosa è fatto

- Valore/Testo: contenuto fisso multilinea o sensore, inserimento di ora/data/nome
  macchina quando disponibile/valore reale; conversione in testo con il formato
  del sensore. Parti prefisso/numero/unità/suffisso con font, peso, colore,
  dimensione relativa e apice, ripristino dell'ereditarietà e Annulla. Cifre fisse,
  overflow, Conta/Rullo, durata e media delle letture; soglie esistenti conservate.
- Ombra del testo approvata in D-17: modello facoltativo, DTO, maschera e sfocatura
  native/demo, colore/opacità e raggio nell'inspector. Da spenta non genera maschere.
- Anello/Barra: direzione ed estremità, spessore/arco, riempimento uniforme o
  sfumatura tramite editor condiviso, traccia, segmenti, angoli e preset esistenti.
  Ripristino corretto delle unioni di paint, con un solo Annulla. Demo allineato
  alla geometria nativa delle quattro direzioni, segmenti e sfumature.
- Lancetta: tutti i controlli di T-3, zone (max 8), arco e colori, tacche/numeri,
  forma/lunghezza/coda, perno/cappuccio, immagine dal catalogo reale, massimo e
  ritorno; valore centrale con formato e parti riusati dal Valore.
- Grafico: una/due serie indipendenti da Linea/Area/Barre/Arrotondate (D-18),
  sorgente inferiore e colore conservati cambiando stile. La serie inattiva non
  viene campionata. Distanza fra serie, finestra storica, scala e controlli
  precedenti conservati. Configurazione della rotazione nel collegamento ⚡ della
  Disposizione; aprirlo non modifica il tema.
- Spessore delle barre approvato in D-19: `visualization.barWidth` facoltativo.
  Senza resta il calcolo precedente da intervalli e spazi; `lineWidth` storico non
  cambia significato. Larghezza esplicita centrata e limitata all'intervallo,
  ripristino automatico; stessa geometria nel demo.
- Selezione multipla: tre anteprime isolate reali, prova temporanea senza modifiche
  al documento, nomi selezionati rimovibili, soli campi comuni e valore «misto»;
  geometria, sorgente, colore e opacità aggiornati in un solo comando Annulla.
  Allineamento, distribuzione, raggruppamento e menu oggetto restano disponibili.
- Layout comune: geometria compatta, appartenenza a gruppi/Card da dati veri,
  ordine livelli, opacità numerica, testo fisso distinto dallo stato dal vivo.
  Tutte le stringhe nuove sono nell'area inspector, con parità en/pt-BR/it.

## Commit

- `ee54298` — ombra nativa/demo e compatibilità DTO.
- `9c1a4ef` — serie e arrotondamento indipendenti, sorgente inferiore inattiva.
- `defd7b6` — geometria demo dei misuratori e formato dei valori nativo/demo.
- `0b263cc` — larghezza barre facoltativa e prove di compatibilità.
- `04e13ed` — primitivi condivisi e modifiche atomiche della selezione.
- `ef8b961` — inspector collegati e spec e2e dell'area.
- `119fb8d`, `d17c62c` — leggibilità delle etichette del Grafico.
- `411d356` — test dell'intero fotogramma dei temi a barre.
- `521555a` — ricette e catture it chiaro/scuro.

## Test con esito

Tutte le verifiche finali sotto sono verdi; nessuna suite completa eseguita.
I log di sviluppo sono raggruppati in `target/logs/t7a/` della worktree.

- `cargo test -p bezel-core -p bezel-themes -p bezel-render`: verdi;
  core 325 test, themes 91 più 2 doctest, render 129 prima degli ultimi due test.
  Dopo l'aggiunta delle verifiche sull'intero tema, `cargo test -p bezel-render`:
  **131 passati**, due test ignorati ereditati (uno unitario e il benchmark timing).
- `cargo clippy -p bezel-core -p bezel-themes -p bezel-render --all-targets -- -D warnings`:
  verde; clippy del solo render ripetuto dopo i due nuovi test, verde.
- `cargo fmt --all -- --check`: verde.
- Unità UI mirate: **135 passate** nei moduli inspector-edit, graph-inspector,
  gauge-layout, text-shadow, value-format, editor-store, demo-backend,
  inspector-model, inspector-sections, i18n e static-guards. Ultima modifica ai
  testi: i18n/static-guards **51 passate**.
- `node scripts/e2e-slot.mjs tests/e2e/inspector-linked.spec.mjs`: **28 passate**
  (7 casi × en/pt-BR × chiaro/scuro), comprese accessibilità, tastiera, Annulla,
  ereditarietà, prova con movimento ridotto, ombra e pixel del demo.
  Le sole etichette abbreviate sono state poi verificate dal controllo i18n e
  dalle catture italiane finali.
- Regressioni mirate durante il lavoro: inspector-sections, inspector-components,
  inspector-stepping e inspector-trial verdi insieme ai casi collegati
  (esecuzioni da 56 e 40 casi; non sono conteggi distinti da sommare ai 28).
  Nessuna asserzione indebolita o spec di altre task modificata.
- Compatibilità barre: checksum RGBA **1239134809160493365** registrato prima
  dell'estensione e rimasto identico; vecchio tema senza visualization confrontato
  pixel per pixel con una matrice indipendente, anche con `lineWidth` 2 o 12.
  Larghezze esplicite provate nei pixel nativi, nel tema completo e nel canvas JS.
  Campi assenti omessi e andata/ritorno verificati nei DTO.
- G3: **14 catture** con `scripts/fidelity/shoot.mjs`, sette stati in it nei due
  temi; dettagli e contesto conservati in [evidence/t7a](evidence/t7a/README.md).

## Confronto e differenze dalle tavole

Consultate le PNG e il sorgente HTML offline delle sei tavole assegnate (Valore e
Testo nella stessa tavola). L'apertura file:// delle tavole nel browser non era
supportata nell'ambiente; il confronto usa le PNG e le catture dell'app.

I dati esemplificativi delle tavole non sono copiati: sensori, letture, font,
asset, geometrie e nomi provengono dal modello e dal catalogo. La serie inferiore
non viene inventata quando manca la sorgente. Le facce dei font provengono dal
catalogo: fino a sei sono campioni Aa, oltre sei rimane il selettore completo.

I pannelli conservano controlli e note precedenti non disegnati nelle tavole
(preset, precisione della finestra, modalità importate, azioni e allineamento),
perciò sono più alti; sono registrati nelle evidenze per la revisione T-10.
Clip rimane visibile solo nei testi importati senza overflow esplicito, evitando
che aprire l'inspector cambi la resa precedente. Il Grafico usa l'abbreviazione
italiana «Arrotond.» presente nella tavola per evitare il taglio dell'etichetta.

## Riuso e T-10

Nessuna duplicazione dei primitivi: `numberField`, `iconChoice`, `previewChoice`,
`partsRow`, `objectHeader`, `fontPicker`, `colorPicker`, `imageField`, `sizeField`,
`bindButton`, menu oggetto e programmazione `preview-animation` restano condivisi.
Le estensioni riusabili sono descritte in `INSPECTOR-COMPONENTS.md`.
I nuovi helper di testo e riempimento servono anche T-7b; la logica pura è fuori
`src/ui/` e testata. Nessun secondo accordion o editor di parti.

A T-10: suite completa e copertura una volta; verifica visiva complessiva dei
pannelli e del renderer Windows con immagini reali, adattamento delle vecchie
spec che si aspettano slider/checkbox/schede sostituiti. La demo conserva la
limitazione preesistente sulle immagini segnaposto, inclusa la bitmap personale
della Lancetta: la geometria vettoriale e i nuovi campi sono allineati, la prova
con un asset reale va eseguita anche nativamente. Nessuna immagine o funzione
nuova inventata per aggirare il bridge.
