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
