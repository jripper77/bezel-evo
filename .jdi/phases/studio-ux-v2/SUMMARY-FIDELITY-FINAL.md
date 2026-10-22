# Revisione visiva finale — punto di consegna manuale

22 ottobre 2026. Worktree `F:/dev/bezel-studio-visual-review`, branch `fid/studio-visual-review`, base `8454a87a`.

**Pronto per la verifica visiva dell’utente (punto 5). Non è approvazione manuale della fase.**

Galleria canonica: `evidence/final/index.html`, rigenerata sul codice corretto. **108 stati** (54 scuri, 54 chiari), **208 PNG** validi; tutti i collegamenti alle immagini verificati, tutte le **30 tavole** presenti. Inventario e SHA-256 in `evidence/final/verification.json`. I frame dei quattro temi sono quelli nativi già prodotti in T-9d, non nuove prove hardware.

## Ultime correzioni

- `c0797d9`: il selettore di immagini demo usa un PNG reale 96×96 già incluso nel tema Turing (3547 byte), con miniatura e metadati reali. Gli altri import demo privi di contenuto restano senza metadati inventati. Nessun nuovo decoder o asset esterno.
- `321a295`: etichette numeriche neutre e compatte, header senza il livello ripetuto (tooltip e metadati mantenuti), dettagli da contenuto/geometria tramite helper puro; croci di rimozione sui chip della multipla con descrizione accessibile e azione esistente; angoli e due comandi dello Spettro riordinati e compatti usando i componenti comuni.
- `761bc03`: Spaziatura lettere usa il controllo numerico compatto già scelto dal proprio editor.
- `576ca55`: ricette e copertura della galleria. Orologio analogico quadrato 300×300, PNG raster reale, Profili attivo/pronto/manuale con runtime demo reale e contenuto completo.
- Galleria: tutte le 30 tavole censite, inclusi Logo, Connect e riferimento DeviceBoot con limiti dichiarati; schemi chiaro/scuro, profili nei tre stati.

## Test mirati

- Fixture PNG: 101/101 unitari (demo-backend, media-inspector, i18n, static-guards); 64/64 e2e (inspector-media-controls e library, quattro progetti). Decodifica reale PNG e dimensioni 96×96 verificate nel browser.
- Presentazione: 74/74 unitari (inspector-model, inspector-helpers, spectrum, i18n, static-guards).
- Header/controlli: 40 casi e2e distinti passati tra il giro iniziale e la riesecuzione mirata. Un errore introdotto durante la rimozione del livello dal testo aveva rimosso dati ancora usati dal tooltip: corretto prima del commit. I 7 casi della spec inspector-linked in light-pt sono poi tutti verdi; gli altri tre progetti e inspector-media-controls verdi. La prima acquisizione interrotta non è quella consegnata.
- Font/campi comuni: 63/63 unitari (text-layout, inspector-edit, i18n, static-guards) e 28/28 inspector-linked dopo l’ultima correzione della Spaziatura lettere.
- Multipla: 8/8 inspector-containers, inclusa rimozione del singolo chip e ritorno alla selezione completa.
- Spettro: 20/20 audio-spectrum, campi condizionali e Undo invariati.
- Suite completa e copertura generali non ripetute: questa consegna prepara la verifica visiva dell’utente; non è una nuova pubblicazione o verifica generale di fase.

## Differenze e limiti da mantenere espliciti

- Rotazione su riga propria per la leggibilità approvata in T-10; assente su Card/gruppi/membri. Multipla: solo se tutti gli oggetti possono ruotare. Duplica/Elimina nei menu e Ordine in Disposizione come T-12.
- Angoli con disegno proporzionale D-22, più ampio del disegno sintetico delle tavole. Editor aggiuntivi esistenti accessibili mediante disclosure; avvisi di sorgenti mancanti e funzionalità già supportate conservati.
- **Limite del demo immagini:** la scheda file mostra il PNG reale e le dimensioni; il renderer demo del canvas resta a contorno/scacchiera e non decodifica le texture. Non è dichiarata parità pixel dell’anteprima oggetto per queste immagini. Estensione da pianificare in `.jdi/todos/2026-10-22-demo-image-preview-parity.md`, senza aggiungere una nuova decodifica in questa correzione grafica.
- Nessun fulmine fittizio per legare l’opacità: il modello non lo espone. Restano cinque effetti reali dei profili, nessuna nuova transizione inventata nella correzione grafica. Salva secondario per D-2026-10-09-studio-redesign-fidelity-6.
- Meteo: dissolvenza automatica 300 ms per D-09; non viene aggiunto l’interruttore non previsto dal modello. Le animazioni non sono dimostrate da PNG statici: valgono i test del motore/demo di T-4.
- Esportazione `.bezeltheme` confermata; `.turtheme` solo in importazione. Video negli oggetti rinviati a T-11. Dati mancanti dei quattro temi sostituiti come già autorizzato, senza nuove sorgenti.
- Connect conserva il dialogo sull’editor; il materiale lo indica come riferimento di stile, non come nuovo onboarding. DeviceBoot è riferimento del display, non una prova hardware. Logo conservato.
- Porte, nomi, font selezionati, valori, proporzioni degli oggetti e miniature provengono dai temi/demo reali. Non copiati i dati campione delle tavole.

Non sono stati eseguiti build, installazione, push, pubblicazione o nuove prove hardware. Versione e installazione locale restano quelle precedenti.

## Registro conclusivo delle tavole

| Tavole | Riscontro finale |
|---|---|
| InspectorComponents, MainNew | Primitivi e shell riallineati; Salva secondario per D-6; valori/nomi reali. |
| RingInspectorOpen, BarInspectorOpen, GraphInspectorOpen | Header/Tipo/Disposizione/Ordine, forma e colore comuni; grafico Area con seconda serie reale. Rotazione su riga propria approvata. |
| ValueInspectorOpen, ClockInspectorOpen, NeedleInspectorOpen | Parti reali del testo, editor compatto, campi raggruppati; analogico quadrato comparabile; extra proprietà esistenti conservate. |
| WeatherInspectorOpen, WeatherAnimations | Ricerca, unità/lingua, colore e moto; D-09 preservata. PNG dei controlli; movimento verificato dai test T-4, non dal fermo immagine. |
| PlayerInspectorOpen, SpectrumInspectorOpen | Copertina/trasparenza/progresso e spettro compatti; avvisi su sorgenti realmente mancanti conservati. |
| ShapeInspectorOpen, ImageInspectorOpen | Scheda file e fit/fade/angoli riallineati; raster 96×96 reale; limite del renderer demo immagini esplicito. |
| CardInspectorOpen, ThemeInspectorOpen, MultiInspectorOpen | Facce con miniature/conteggi reali, transizioni e chip; Tema e multipla compatti; rimozione chip verificata. D-22 e assenza Rotazione Card preservate. |
| LibraryPanelsNew, StatusBarNew, ScreenPanelsOpen, CanvasHandles | Larghezze comparabili, rail/Widget/Livelli/Media e chip corretti; stati e menu veri; maniglie nei diversi contesti. Nessuna nuova prova fisica. |
| ProfilesNew, ThemesNew | Composizione ricostruita coi primitivi comuni; profili attivo/pronto/manuale, cinque effetti supportati, export .bezeltheme. |
| ThemePixel, ThemeNebula, ThemePaper, ThemeNowPlaying | Ricontrollati i quattro frame nativi T-9d rispetto alle tavole; sostituzioni dei dati già approvate conservate. |
| Logo, Connect, DeviceBoot | Marchio conservato, Connect riferimento di stile/dialogo esistente; DeviceBoot riferimento hardware senza nuova cattura. |

## Per la verifica dell’utente

Aprire la nuova galleria finale, usare i collegamenti alle immagini intere per leggere i pannelli e confrontare soprattutto composizione, proporzioni, controlli, colori e icone. Le vecchie gallerie dei singoli blocchi sono evidenze intermedie, non il riferimento canonico finale. Valutare esplicitamente le eccezioni sopra, in particolare le immagini nel canvas demo. Le prove sul display reale richiedono una successiva build/installazione richiesta dall’utente.

Gli intermedi grafici della sola attività sono conservati in target/scratch/fidelity-final e fidelity-final-refresh: la pulizia ricorsiva è stata bloccata dalla revisione automatica degli strumenti. Non necessari per aprire la galleria finale.
