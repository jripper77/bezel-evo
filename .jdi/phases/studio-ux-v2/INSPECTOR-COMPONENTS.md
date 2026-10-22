# Catalogo dei componenti — T-0

Implementazione unica: `apps/bezel-studio/src/ui/fields.js`; stile comune: `src/styles/base.css`.
Il chiamante fornisce testi tradotti, valori del documento e callback. I componenti non accedono allo store.
Colori e font continuano a usare `colorPicker` e `fontPicker`, senza varianti locali.

| Componente | Contratto e riuso |
|---|---|
| `accordionSection(descriptor, ctx)` | Restituisce `{element, setExpanded}`. Titolo nativo, riepilogo da chiusa, regione nominata; frecce, Home/End. La memoria appartiene al guscio. |
| `numberField(label, value, onChange, opts)` | Numero con etichetta, unità, icona; `prefix` usa il campo prefissato esistente, `binding: {label, connected, onClick}` aggiunge il fulmine fuori dall'etichetta. Conserva il supporto `adoptControl`. |
| `iconChoice(label, value, options, onChange, opts)` | Opzioni `[value, label, iconName]`, pulsanti `aria-pressed`; `wide`, `iconOnly`, `bare`. Classe `.field-choice`, distinta dalle icone della libreria. |
| `previewChoice(label, tiles)` | Card `{text, art, pressed, onClick}`: implementazione già locale in inspector, ora condivisa. |
| `cornersField(...)` | Nome pubblico del componente esistente `cornerRadii`, stessa API `{tool, control}`; nessuna nuova geometria o seconda implementazione. |
| `fadeField(texts, value, onChange)` | Nessuna/lineare/radiale, direzione/punto medio oppure centro/estensione, opacità iniziale/finale. Emette patch del modello esistente. Testi: `mode, none, linear, radial, angle, midpoint, centerX, centerY, extent, start, end`. |
| `imageField(label, value, choices, onChange, opts)` | Scelta degli asset reali; `preview` e `details` ospitano metadati/controlli del modello specifico. Riutilizzato da Immagine e immagine della Forma. L'importazione resta nel flusso Media esistente. |
| `sizeField(texts, frame, onChange, opts)` | Larghezza/altezza con unità e pulsante di collegamento. `linked` e `onLink` sono stato UI, senza campi nuovi nel tema. `resizedFrame` puro mantiene le proporzioni con una sola modifica allo store. |
| `bindButton(label, connected, onClick)` | Fulmine nativo con stato; usato anche dal toggle del colore legato al valore. Il sensore/editor effettivo resta responsabilità del chiamante. Nessun campo fittiziamente collegabile. |
| `toggleField(label, pressed, onChange, opts)` | Etichetta e pulsante-icona `aria-pressed`; `iconName` seleziona il simbolo, `bolt` riusa `bindButton`. Lo switch condiviso delle altre aree resta disponibile. |
| `objectHeader({preview,name,subtitle,live,tryButton,menuButton})` | Composizione unica della testata: anteprima isolata 72 px, nome, sottotitolo da dati reali, stato e azioni. |
| `typeRow(label,value,options,onChange)` | Riusa `iconChoice` con icone e larghezza uniforme. Il guscio conserva la semantica di `setKind`, sensore e limiti. |
| `partsRow(label,parts,selected,onSelect,renderEditor)` | Scelta delle parti e contenitore dell'editor della parte attiva; i nuovi modelli con parti arrivano da T-1 e vengono collegati in T-7. |

## Moduli per tipo

Tutti i moduli `src/ui/inspector/<tipo>.js` espongono `sections(ctx)`, con descrittori:

```js
{ id, title, icon, summary(element), render(body, ctx) }
```

`sections.js` raggruppa i controlli esistenti secondo `SECTION_ORDER` e non elimina quelli sconosciuti.
I gruppi interni conservano `data-section`; il contenitore apribile usa `data-inspector-section`,
evitando collisioni tra sezione canonica Colore e gruppo di campi Colore.

Il guscio fornisce contesto tradotto e servizi condivisi (`store`, `update`, sensori, picker, bridge,
host, `completePanels` per completare i campi comuni). T-7 può sostituire le funzioni interne `panels`
mantenendo il contratto pubblico. I moduli non implementano memoria, tastiera o un secondo accordion.

`createSectionMemory` conserva le aperture per tipo in `bezel.inspectorSections`, tollera storage non
disponibile o corrotto e gestisce Alt+clic esclusivo. La sola apertura non modifica il documento.
Le nuove icone sono nel catalogo `icons.js` con i nomi dei file SVG offline.

## Verifica e punti di integrazione

Primitivi e flussi verificati da `inspector-primitives`, `inspector-components` e dalle spec
`inspector-*`; accessibilità senza eccezioni in chiaro/scuro e en/pt-BR.
La memoria pura ha copertura 100% di righe, rami e funzioni. Guardrail CSS/SVG e parità i18n verdi.

Nessuna duplicazione. Le specifiche dei nuovi campi dei widget restano assegnate a T-7a/b/c;
la rimozione dei vecchi helper e delle chiavi orfane resta assegnata a T-10.


## Estensioni condivise di T-7a

- `numberField` e `prefixedField`: opzione `nullable`, campo vuoto restituisce `null`;
  l'adozione dell'input conserva anche parser, passo e placeholder aggiornati.
- `textField`: opzione `multiline`, con textarea nativa e lo stesso contratto change.
- `fontPicker`: campioni delle facce con `faceSample`, tramite `iconChoice`, e filtro
  `fixedItalic` per le parti che ereditano il corsivo. Oltre sei facce resta il select.
- `inspector/text.js`: parti, formato del valore, overflow e ombra riusabili anche
  per il centro della Lancetta e da T-7b per l'Orologio.
- `inspector/paint.js`: editor comune dei riempimenti di Barra e Anello, con sostituzione
  atomica dell'unione colore/sfumatura e un solo Annulla.
- `inspector-edit.js`: proprietà comuni e patch pure; `patchElements` applica percorsi
  diversi in un solo comando, conservando trasformazioni delle Card e blocchi.

Nessuna duplicazione dei primitivi: campioni, scelte, parti, intestazioni, menu,
picker e programmazione delle anteprime usano le implementazioni condivise di T-0.


## Estensioni condivise di T-7b

- `textStyleFields`: `partIds`, `partLabel`, `partsLabel` e `scope` permettono di
  riusare lo stesso editor per le parti dell'Orologio; nessun secondo editor.
- `fadeFields`: valore e callback configurabili, riusato per la sola copertina.
- `gaugeFillFields`: percorso del paint configurabile, riusato da Forma e Spettro
  oltre a Barra/Anello; sostituzione atomica del paint con un solo Annulla.
- `previewChoice`: slot `preview` facoltativo per le anteprime reali dei quattro
  stili meteo, disegnate dalla stessa funzione canvas del demo.
- `iconChoice`: accetta anche tracciati del catalogo oltre ai nomi; selettore
  unico per le sette modalità di ripetizione. `numberField` e `prefixedField`
  accettano un placeholder, utile al valore automatico dell'icona.
- Nel renderer `Layer::clip_to` è il ritaglio condiviso fra Immagine e copertina;
  `Layer::fade` è la rampa comune a Forma/Immagine/copertina. Stessa separazione
  nel demo con `drawClippedMedia` e `applyCanvasFade`.

Nessuna duplicazione dei primitivi. Le nuove opzioni del modello sono in D-20.


## Estensioni condivise di T-12

- `iconChoice`: modalità `actions` per comandi momentanei (senza `aria-pressed`),
  `detail` per un dato secondario nella didascalia e `caption` per il testo visibile
  quando il nome accessibile conserva un contesto più esteso; quarta voce opzionale della
  scelta `{disabled, iconOnly, text, leading, description}`. Stesso riquadro e stile condiviso,
  nessuna seconda toolbar per Ordine o Allinea. Le scelte testuali possono andare
  a capo; Tipo resta compatto su una riga.
- `objectHeader`: nome/azioni e sottotitolo/stato sulle due righe condivise.
- `numberField`: `compact` conserva etichetta sopra e icona dentro il controllo,
  riusato da Durata e Media delle transizioni.
- `colorPicker`: `hideLabel` nasconde solo l'etichetta visibile ridondante,
  conservando il nome accessibile. `swatchChip` esportato con estremi opzionali
  per le scelte di riempimento.
- `description` sulle scelte conserva aiuti di comportamento come tooltip e
  descrizioni accessibili, senza aggiungere un paragrafo alle tavole.
- `propertySection`: `showIcon: false` per gruppi che hanno una semplice
  didascalia nella tavola, senza un'icona generica inventata.
- `inspector/order.js` riusa `iconChoice` e `layerOrderAvailability`, comune allo
  store e al menu oggetto esistente; il livello viene da `layerFacts`.

## Riallineamento Colore

- `fieldAction(field, action)` affianca un’azione al controllo sotto la didascalia, usando `.bound-field`.
- `gradientField(start, end, {stops, midpoint, onMidpoint, label})` compone i picker esistenti, una barra e il range nativo; il numero esatto e il cursore modificano lo stesso valore.
- `colorPicker`: `swatchOnly` nasconde soltanto il readout visivo, conservando nome e descrizione accessibili del pulsante.
- `propertySection`: `caption` per le didascalie semplici delle tavole, senza un’icona interna inventata.

## Riallineamento Testo

- `partsRow`: `preview` restituisce le parti del contenuto reale; mantiene ordine e separatori ripetuti. `refreshPreview()` aggiorna solo i testi cambiati; nessuna scrittura nel tema.
- `fieldGrid`: `tail: true` per peso/numero con campo finale da 96 px.
- `fontPicker`: `hideLabels: true` per l’editor compatto, senza perdere i nomi accessibili.
- `propertySection`: `heading: false` quando la didascalia è già nel primo controllo.

## Riallineamento Geometria

- `iconChoice`: `columns` per distribuire molte scelte senza etichette troncate.
- `fieldGrid`: `lead: true` per una prima colonna compatta, complementare a `tail`.
- `cornerRadii`: stessi calcoli e disegno D-22, modi a icone; nel modo uguale selettore e raggio sono nella stessa riga.
- `segmentsFields`: stessa riga comune per Anello e Barra, con numero/spazio disabilitati quando spento.

## Riallineamento Grafico/Spettro

- `detailsField(label, nodes, {open, onToggle})`: details/summary nativi per impostazioni secondarie; lo stato appartiene al chiamante.
- `fieldGrid`: `columns` e `lead` per righe con tre campi senza troncare i numeri; variabili CSS impostate con setProperty.
- `previewChoice`: `caption` facoltativa, nome e tooltip continuano a usare `text` completo. Cinque miniature usano la densità compatta del catalogo.

## Fedeltà media — 22 ottobre

- `imageField`: `asset`, `changeLabel`, `browseLabel`, `onBrowse` per la scheda compatta con selezione nativa e miniatura reale; il chiamante fornisce metadati di `imageAssetFacts`.
- `fadeField`: anteprima 64×64 trascinabile/tastiera e campi numerici compatti, senza modello aggiuntivo. Testi aggiuntivi `x, y, from, to, midPrefix, extentPrefix, directionControl, centerControl`. Un callback al rilascio, nessuno su annullamento.
- `fadeHandlePatch` puro e testato in shape-fade.js; `imageAssetFacts` puro e testato in media-inspector.js. Nessuna geometria nel modulo DOM specifico di un oggetto.

Meteo: `toggleField(..., { leading: true })` porta il comando prima della didascalia con lo stesso pulsante/stato condiviso; nessuna seconda implementazione del toggle. Ricerca riusa `searchField`, `fieldAction`, `iconButton` e `listCard`.
