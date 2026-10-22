## Aggiornamento — Meteo, 22 ottobre

Ricerca, righe compatte, ordine Colore e icone del movimento corrette in `3d37a0a`. Test, catture e differenze vincolate ai primitivi/alla decisione D-09 in `SUMMARY-FIDELITY-WEATHER.md`. Restano gli altri blocchi sotto e la chiusura generale.

## Aggiornamento — media, 22 ottobre

Immagine/GIF, Forma e Player corretti nel blocco `616a7be`; controlli condivisi, test e differenze motivate in `SUMMARY-FIDELITY-MEDIA.md`. Catture it chiaro/scuro in `evidence/fidelity-media/`. Restano aperti gli altri blocchi e la rigenerazione finale; il registro iniziale sotto resta la traccia del confronto di partenza.

# Confronto dell'interfaccia e piano delle correzioni

**Aggiornamento delle correzioni:** completati i quattro blocchi Colore, Testo,
Geometria e Grafico/Spettro (commit e seguito in `HANDOFF.md`). C06 risolta nella
ricomposizione disponibile, resta il fulmine Opacità privo di modello; C08
fondazione risolta, C09 compatto con disegno mantenuto per D-22. C03/C04/C05 e
le composizioni per tipo restano parziali dove non esplicitamente documentate.
R02 ha una nuova ricetta Grafico Area con due sensori veri nel demo. Non chiusi
C10, multipla di C11, shell/pannelli e dialoghi. Le righe sotto sono il registro
di baseline: usare i SUMMARY-FIDELITY-* per distinguere quanto già corretto.
La galleria finale della fase non è ancora rigenerata e non è richiesta
approvazione manuale fino al completamento dei blocchi restanti.

**Avanzamento dopo la ripresa:** T-12 completata (`SUMMARY-T-12.md`). Risolta la
parte Ordine di C11; corretti contrasto dell'anteprima di C01, compattezza Tipo
C02, selezione neutra e icone delle transizioni di C05, Durata/Media e ritorno a
capo di C07. C06 ha ora campioni, didascalia e nomi corti coerenti, ma resta da
ricomporre l'intero editor. R00 corretto per Valore/Testo e Orologio, ancora
aperto per gli altri casi. Le righe dell'audit sotto descrivono la baseline e
restano il registro per proseguire; questa nota non chiude gli altri riscontri.

22 ottobre 2026, ChatGPT. Worktree `F:/dev/bezel-ux2-t12`, branch `ux2/t12`, base `fb6b42f`, con bozza T-12 non committata. Questo è un audit e non una consegna della correzione.

## Esito

**Il riallineamento visivo non è concluso.** Le differenze sono soprattutto nella composizione dei controlli: elementi impilati invece che affiancati, titoli interni aggiuntivi, editor delle parti molto più grandi, scelte testuali al posto di icone e miniature con proporzioni diverse. Non basta ritoccare qualche colore. I test funzionali verdi non attestano la fedeltà alle tavole.

Criterio concordato nell'ultima precisazione dell'utente: essere precisi su struttura, proporzioni, controlli, colori e icone; tollerare piccole differenze di rasterizzazione. Nessuna percentuale arbitraria di somiglianza né identità dei dati d'esempio.

## Riferimenti e verifica della loro coerenza

- HTML completo: `C:/Users/danie/Desktop/Bezel Studio Redesign.html`, 4.106.452 byte; SHA256 `40d74062cee6833ed1b1dc243932afad8ce32ec42b122349f220a3382d91a692`.
- Estratti come testo i manifest compressi e i template annidati, senza eseguire i loro script: riconosciute tutte le **30 tavole** per titolo. Il CSS di design è identico in **30/30** alle tavole HTML del repository, una volta escluse dichiarazioni `@font-face`, commenti e spaziatura. Questo verifica gli stili, non dimostra identità di ogni nodo DOM: il documento completo contiene anche interpolazioni e componenti dinamici, mentre le tavole offline sono materializzate.
- Inventario riproducibile della corrispondenza: `evidence/audit-2026-10-22/reference-inventory.json`. Gli estratti di lavoro sono in `target/scratch/t12/html-reference/`.
- Il browser ha rifiutato il protocollo `file:` per il documento completo. Il blocco non è stato aggirato: confronto delle sue sorgenti come testo e verifica visiva contro i PNG offline già disponibili.
- Screenshot inviati dall'utente: header, Tipo, Allinea, Direzione, Colore, Comportamento. Sono parte del riferimento, insieme alle decisioni.

## Evidenza prodotta

Galleria di audit: `target/scratch/t12/audit-gallery/index.html`. È stata rigenerata separatamente dalla galleria finale della fase: **96 catture di stato** in italiano, 48 per schema, ripartite in inspector 40, shell 40, library 12, screen 4 (i conteggi per area comprendono i due schemi). Alcuni stati hanno anche il dettaglio del pannello oltre al fotogramma intero.

Aggiunte **4 catture** (logo e nessuno schermo, entrambi gli schemi) in `target/scratch/t12/audit-extra/`. Totale: **100 catture di stato**. Log: `target/logs/t12/audit-gallery.log` e `audit-extra.log`. Le catture descrivono il codice in bozza, non una versione installata.

I quattro temi sono stati confrontati con i frame nativi già prodotti da T-9d, senza ricompilare: prospetto `target/scratch/t12/templates-audit.png`. La galleria collega anche tali evidenze precedenti; non sono nuove misure hardware.

La tavola scura è il riferimento cromatico disponibile. In chiaro si controllano geometria, leggibilità e coerenza dei token approvati: non si confronta il colore chiaro con il PNG scuro fingendo che debba coincidere. Le maschere magenta sul tempo di rendering nelle catture complete sono prodotte dallo script, non sono elementi della UI.

## Problemi del metodo di confronto da risolvere prima dei ritocchi

| ID | Riscontro | Conseguenza / intervento |
|---|---|---|
| R00 | La ricetta impone 360 px a quasi tutti gli inspector. Le tavole Valore/Testo, Orologio e Schermo usano pannelli da 380 px (880 meno 80 di margini e 40 di intervallo, diviso due). LibraryPanelsNew usa 400 px per pannello; la ricetta ne usa 360 e cattura anche la rail. | Confrontare ciascun pannello alla sua larghezza di riferimento, ritagliando il contenuto pertinente. Poi verificare separatamente le larghezze più strette supportate. Non correggere il prodotto per compensare una cattura sbagliata. |
| R01 | MainNew prevede inspector da 360 px; la shell parte da 320 (`shell.css`). | Verificare le decisioni sulle dimensioni predefinite e riallineare il default se non esiste una deroga; conservare il ridimensionamento dell'utente. |
| R02 | Anello acquisito pieno semplice, tavola con sfumatura; Grafico arrotondato con seconda serie non configurata, tavola Area con due sensori; Immagine acquisita come icona Tabler; Card senza oggetti nelle facce. | Aggiungere stati comparabili con dati demo, senza nomi o valori finti nella UI di prodotto. Configurare stato/opzioni nel test, non modificare il documento reale dell'utente. Gli stati attuali restano utili come copertura aggiuntiva. |
| R03 | Profili: cattura con una regola non vera e un solo schermo; riferimento con profilo attivo e più regole. Dialogo acquisito a 900 px e troncato in basso. | Cattura completa del contenuto e stati attivo/pronto/pausa separati; verificare `Overview.status.active` e `status.rules`, senza dedurre l'attivazione dalla selezione. |
| R04 | Una galleria generata contiene immagini, ma non misura né certifica le differenze. | Per ogni gruppo corretto, esame affiancato a scala uguale e registro risolto/residuo; nessun via libera automatico perché lo script termina con codice zero. |

Misure utili già leggibili nelle sorgenti: anteprima oggetto 72 px; riga accordion 48 px; input 34 px con raggio 8 px; scelte neutre con padding esterno 2 px, pulsanti alti 30 px, selezione scura `#262B31`. Nei token attuali il selezionato neutro usa `--seg-thumb: --surface-3`, ossia `#1D2227`: rimuovere l'arancione locale da solo non lo rende uguale al riferimento. Le modifiche cromatiche vanno comunque risolte attraverso i token e le decisioni, non con esadecimali sparsi nei fogli di area.

## Differenze condivise confermate

| ID | Componente | Differenza osservata e correzione proposta |
|---|---|---|
| C01 | Header | La composizione T-12 è più vicina, ma la miniatura trasparente rende invisibili testi bianchi in chiaro (Valore e Orologio). Recuperare un fondo coerente e contrastante, conservando anteprima reale 72 px. I sottotitoli generici «tipo · livello» di molti widget non comunicano il contenuto come nelle tavole; usare i dati già disponibili e togliere la duplicazione del livello ora presente in Ordine. |
| C02 | Tipo | La bozza riduce il selettore: controllare larghezza complessiva, altezza, distanza dal titolo e icone su tutti i cinque tipi. Non considerare concluso il controllo su un solo Anello. |
| C03 | Numeri | Etichette, icone e unità sono spesso disposte diversamente: Durata ha l'icona nell'etichetta e il titolo a destra, con input a tutta larghezza; la tavola ha titolo sopra e icona dentro un input compatto. Problema condiviso da spessori, distanze, bordo e altri campi. Correggere `numberField`/composizione condivisa, non ogni widget con CSS proprio. |
| C04 | Etichette e sezioni | Titoli interni duplicano etichette del campo: Forma/Forma, Ombra/Ombra, Riproduzione/Riproduzione, Serie/Serie, Visibilità/Opacità. Appaiono anche icone di sezione generiche estranee alle tavole. Eliminare ridondanze nella composizione preservando nomi accessibili e tutti i comandi. |
| C05 | Scelte e icone | Direzione ora è neutra nella bozza, ma il token selezionato resta differente; molte scelte usano soltanto testo dove la tavola mostra icone. Altre usano card grandi anziché righe compatte. Riusare `iconChoice`, `previewChoice`, `iconButton` secondo il tipo effettivo mostrato dal catalogo. |
| C06 | Colore | La bozza aggiunge campioni, ma il campione Sfumatura diventa quasi pieno quando il valore corrente è un colore unico. Gradiente ancora composto da campi separati anziché barra con estremi/punto medio; fulmine su riga autonoma; Traccia divisa in toggle + picker + titoli; Opacità sotto «Visibilità». Ricomporre l'editor comune: riempimento e fulmine affiancati, picker o gradiente sotto, traccia compatta, opacità col proprio collegamento dove già supportato. Esadecimali visualizzati con formato uniforme. |
| C07 | Comportamento | Le icone Salta/Fluida/Inerzia sono state aggiunte in bozza. Durata e Media restano impilate; Valore mostra cinque scelte troppo compresse e testi sovrapposti. Overflow mostra anche l'opzione precedente, con etichette tagliate. Conservare modalità e compatibilità esistenti senza obbligare cinque etichette lunghe nella stessa riga. |
| C08 | Parti e caratteri | `partsRow` mostra nomi astratti e un grande modulo con font/colore/peso/baseline separati. Le tavole mostrano parti del testo reale e un editor compatto della parte scelta; anche font/peso/dimensione hanno ingombri diversi. Riallineare il componente condiviso per Valore, Orologio e Lancetta. Nessun secondo editor. |
| C09 | Angoli | La bozza conserva anteprima geometrica + campo raggio e selettore Uguali/Singoli. Nelle tavole la riga base è compatta con icona dell'angolo, numero e comando per i singoli. Ripresentare la stessa logica di `corner-radii.js`, senza duplicare ritaglio o modificare modello. |
| C10 | Immagini e trasparenza | `imageField` rende soprattutto un select; riferimento con miniatura, nome, metadati e sostituzione. Trasparenza lineare usa slider lunghi invece della piccola anteprima con campi raggruppati. Correggere i primitivi esistenti, poi verificarli su Immagine, Forma e copertina Player. |
| C11 | Disposizione / T-12 | Ordine è nella posizione richiesta e il livello manca nella multipla come previsto. Restano da verificare stato disabilitato, tutti gli ambiti, menu e Undo con e2e pertinenti. Multipla conserva controlli e intestazioni diversi dalle tavole: campi non prefissati, distribuzione su due pulsanti separati, raggruppamento molto ingombrante, chip senza rimozione individuale. |

Questi sono difetti da correggere o verifiche da chiudere, non differenze approvate. C01 (fondo trasparente) e parte dell'impaginazione C03 vanno riesaminati specificamente nella bozza T-12: non attribuire automaticamente ogni problema alla base.

## Registro per tavola / area

| Tavola | Riscontro oltre ai componenti condivisi | Prossimo passo |
|---|---|---|
| InspectorComponents | Contratti comuni rispettati solo in parte nella composizione finale. | C01–C10 prima dei moduli per tipo. Non serve inventare un'altra libreria di controlli. |
| RingInspectorOpen | Stile dopo Arco anziché prima; arco con grande anteprima propria; Spessore, Verso, Estremi, blocchi su righe separate; controllo di prova aggiuntivo in fondo. | Ricomporre Forma secondo la tavola, mantenendo prova e funzionalità nel flusso condiviso; acquisire sfumatura e blocchi attivi. |
| BarInspectorOpen | «Visualizzazione» e «A blocchi» duplicano titoli; Angoli precede una composizione molto più alta; traccia e durata diverse. | C03–C07/C09, stessa sequenza della tavola, stato blocchi attivo oltre a quello spento. |
| ValueInspectorOpen | Sensore prima della scelta contenuto; Decimali e Unità non affiancati; editor parti molto alto; cinque transizioni affollate. | C07/C08 e composizione Dati; conservare l'eventuale comportamento di overflow precedente senza forzarne la migrazione. |
| NeedleInspectorOpen | Molti campi in colonna; centro del valore apre l'intero editor nel flusso; zone come intervalli numerici separati anziché elenco compatto; mancano diverse icone delle scelte. | Ricomporre gruppi e editor condivisi; mantenere la possibilità di configurare ogni proprietà oggi disponibile. |
| GraphInspectorOpen | Finestra temporale estesa sempre visibile con molte note, contro riepilogo Precisione richiudibile; Serie/Scala con titoli aggiuntivi; spessore e distanza in colonna. | Riallineare disposizione e disclosure conservando durata, passo, vincoli e reset dello spessore facoltativo. |
| ClockInspectorOpen | Digitale: parti astratte e modulo ampio; analogico: lunghezze e colori su molte righe anziché raggruppati; icone del moto mancanti. | C08 e composizioni specifiche, confrontando ciascuna colonna a 380 px; verificare anche miniatura chiara. |
| WeatherInspectorOpen | Ricerca città con pulsante separato; unità e lingua impilate; ordine testo/icona in Colore diverso; velocità/movimento senza icone e raggruppamento della tavola. | Ricerca con risultati e stato configurato come due casi; ricomporre senza cambiare geocodifica o dati. |
| WeatherAnimations | Cattura statica di un widget non prova le otto famiglie, quattro stili e movimento. | Riutilizzare condizioni demo e specifica `weather-motion.md` per una verifica separata; non dichiarare verificata l'animazione da un PNG. |
| PlayerInspectorOpen | Dati con titoli interni extra; copertina, spaziatura, progresso e trasparenza in ordine diverso e molto più estesi. | C09/C10 e ricomposizione Forma; nessuna nuova decodifica o sorgente audio. |
| SpectrumInspectorOpen | Barre/frequenze in verticale anziché riga; Modalità con testi troncati; movimento senza icone. | Raggruppare dati e geometria, usare icone delle tavole e correggere label corte/nomi accessibili. |
| ShapeInspectorOpen | Sagoma come testo senza icone; finestra video con riga dedicata; bordo e angoli con grandi sottosezioni; immagine come select e campi verticali. | C04/C09/C10, preservando ripetizione, scala e spostamenti condizionali. |
| ImageInspectorOpen | Fixture attuale è un'icona SVG, quindi mostra anche colore/contorno/ombra; fit testuale e file select non corrispondono alle card del riferimento. GIF ha playback senza icone. | Fixture raster + Copri per vedere ancoraggio; GIF separata, SVG come copertura aggiuntiva. Video resta escluso per T-11 rinviata. |
| CardInspectorOpen | Lista facce senza miniature nella riga e con editor molto ampio; forma e transizioni composte con sottosezioni; aggiunta trigger come pulsanti larghi. | Facce popolate, miniature reali, editor e regole compatti. **Non aggiungere Rotazione**: errore già deciso nella tavola. |
| MultiInspectorOpen | Differenze C11; tavola con azioni in fondo supera in parte la nuova richiesta T-12. | Ordine nella Disposizione; Duplica/Elimina nei menu come richiesto adesso, non ripristinare Elimina nel pannello per copiare il vecchio disegno. Rotazione solo se tutti ruotabili. |
| ThemeInspectorOpen | Stato non salvato su riga separata, orientamento/rotazione e righe Sfondo meno compatti. | Riallineare header e righe con dati reali; conservare chooser e conferma dello sfondo esistenti. |
| MainNew | Inspector default 320 invece di 360; toolbar orientamento con parole invece di sole icone; Salva non arancione nella cattura; rail e ingombri tile diversi. | Verificare regole/decisioni precedenti, poi riallineare shell e Widget. Non confondere tema demo e valori diversi con difetti. |
| LibraryPanelsNew | Sensori più vicini, ma viewport non mostra gli stati mancanti/collassati; Livelli ha frecce persistenti in più e fixture senza Card/gruppi; Media non mostra casi realmente in uso, miniature demo mancanti. | Cattura a 400 px senza rail; poi densità delle righe, stati hover/selezione/visibilità/blocco, Card e gruppi, uso degli asset. |
| StatusBarNew | Tre chip presenti; popover Sensori ripete conteggi; testata prestazioni priva del pallino nella cattura; stati cromatici molto più accesi del riferimento. | Verificare struttura e colori per neutro/operativo/attenzione/errore, con dati di stato appropriati. Non falsificare tempi o provider. |
| CanvasHandles | Catture prodotte per famiglie e ingressi, ma spesso oggetto piccolo nel canvas verticale anziché dettaglio grande della tavola. | Casi a zoom comparabile, confronto di forma/dimensione/posizione delle maniglie e feedback del campo; drag funzionale resta nelle spec esistenti. Non riorganizzare le maniglie solo per rendere uguali dati diversi. |
| ProfilesNew | Aggiunta regole come chip testuali anziché griglia 2×3 a icone; priorità a tutta larghezza; cambio tema come select; testate/miniature con proporzioni diverse. | Riusare accordion, chip, menu e campi, ricomponendoli come nella tavola. Preservare gli stati reali e il salvataggio automatico. |
| ThemesNew | Anteprime alte legate al formato verticale, contro strisce basse; filtri su due righe; azioni laterali impilate; metadati e titolo distribuiti diversamente. | Cornici basse uniformi con contenuto reale contenuto, filtri e toolbar compatti, header e azioni comuni. **Esportazione `.bezeltheme` confermata**, import `.turtheme` resta. |
| Connect | Dialogo sull'editor contro tavola a pagina intera. Il README del design lo classifica però come riferimento di stile invariato; la tavola contiene anche istruzioni Linux obsolete. | Registrare differenza; non allargare T-12 a un nuovo onboarding né copiare istruzioni Linux. Rivalutare lo scope separatamente alla ripresa. |
| Logo | Acquisito simbolo attuale chiaro/scuro; nessuna correzione del marchio dimostrata dall'audit. | Conservare asset; non ridisegnare logo nel riallineamento dei controlli. |
| DeviceBoot | Riferimento della schermata fisica, non un pannello dell'editor. Nessuna nuova cattura hardware. | Fuori da questa verifica grafica dell'interfaccia; non dichiarare collaudo sul display. |
| ThemePixel | Impaginazione principale riconoscibile nei frame nativi; Giornata→Volume e cache→dato disponibile già autorizzati. | Non riaprire le sostituzioni dati; eventuali ritocchi grafici dopo i controlli dell'editor. |
| ThemeNebula | Blocchi principali conservati; meteo e serie temporali differiscono per dati/sostituzioni già documentate. | Distinguere i dati reali dall'architettura grafica; nessun nuovo forecast. |
| ThemePaper | Tre colonne conservate; meteo in righe e grafico CPU al posto del forecast, niente variazioni degli indici: autorizzati. | Conservare sostituzioni; non introdurre motore dati nella correzione di fedeltà. |
| ThemeNowPlaying | Impaginazione generale presente; copertina e forma dello spettro dipendono dagli ingressi; dati diversi. | Confrontare geometria con ingressi deterministici, senza pretendere la copertina della tavola. |

## Ordine di lavoro proposto alla ripresa

1. **Confronto affidabile:** sistemare larghezze, ritagli e stati delle ricette (R00–R04); conservare questa baseline. Preparare casi corrispondenti senza contaminare prodotto o temi dell'utente.
2. **Fondazione comune + chiusura T-12:** C01–C07/C09; completare Ordine/menu e test degli ambiti, correggere la migrazione della spec che apre due volte il menu. Verificare subito Anello e Barra, poi tutti i chiamanti dei primitivi. Documentazione richiesta da T-12 e commit solo quando pronta.
3. **Testo e misure compatte:** C08, Valore/Testo, Orologio e Lancetta; verificare resa dei valori ereditati e campi facoltativi, senza cambiamenti al formato dei temi.
4. **Media e contenitori:** C10, Player, Meteo, Spettro, Forma, Immagine/GIF, Card, Tema e multipla; stessa composizione ovunque, differenze specifiche solo quando il comportamento lo richiede.
5. **Shell e pannelli:** MainNew, LibraryPanelsNew, StatusBarNew, Schermo/Archivio e maniglie. Rispettare le deroghe già scelte su Rotazione e standby.
6. **Profili e galleria:** due confronti dedicati per le differenze strutturali, non semplici ritocchi CSS; nessuna riscrittura del runtime né perdita di regole/stati reali.
7. **Chiusura visiva unica:** nuova galleria completa, registro di ogni differenza chiuso o motivato, e riepilogo breve delle eccezioni per l'utente. I quattro temi restano con i dati disponibili autorizzati; Connect/DeviceBoot non entrano implicitamente in T-12.

Ogni gruppo va tenuto piccolo, nella propria attività/worktree dopo la chiusura di T-12, con test dei soli moduli e spec toccati, guardie statiche, i18n e controllo chiaro/scuro/en/pt. Non creare una grande patch che renda impossibile attribuire una regressione. La proposta va valutata alla ripresa: stanotte non sono state applicate nuove correzioni al prodotto durante l'audit.

## Verifiche e limiti al momento del salvataggio

- Unitari del checkpoint, sul codice attuale: **89/89 passati**, `target/logs/t12/unit-checkpoint.log` (layer-order, editor-store, layers, inspector-helpers, i18n, static-guards).
- E2e precedenti: **60 passati, 4 falliti**, stessa spec duplicazione nei quattro progetti. Il test apre il menu direttamente e poi di nuovo tramite il nuovo helper: da correggere, non da ignorare. Nessuna asserzione indebolita nell'audit.
- Generazione galleria e catture extra concluse con esito positivo. Non sono stati eseguiti nuovi test Rust, suite completa, build, installazione o push.
- Restano da verificare gli stati interattivi (focus/hover/disabilitato, trascinamento e animazioni), tutti i casi non coperti dalle ricette e le misure sul display reale. Questo audit identifica la direzione e i problemi concreti; non li dichiara risolti.
