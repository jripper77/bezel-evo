# Parità delle immagini nelle anteprime del demo

Rilevato durante la revisione visiva finale Studio UX v2 del 22 ottobre.

Il selettore demo ora fornisce un vero PNG 96×96 (`c0797d9`): la scheda file e la libreria lo decodificano e mostrano miniatura e dimensioni reali. Il renderer del canvas resta però quello precedente: `demo-render.js`, ramo `case image`, disegna il contorno; `drawPicturePlaceholder` usa una scacchiera nella Forma. `demo-backend.renderElement` e `renderedThumbnail` passano per quel renderer. Non rappresentano quindi i pixel reali dell’immagine nella testata oggetto/nel canvas. Il nativo usa già la decodifica del motore.

Questa limitazione è esplicita in REVIEW e nella galleria, non dichiarata corretta. Una nuova attività deve definire il passaggio/caching dei pixel decodificati dal browser al renderer demo, senza una seconda semantica di fit, ripetizione, angoli o trasparenza. Riutilizzare Image.decode/OffscreenCanvas e le geometrie esistenti; caricare solo al cambio di asset, con durata legata al documento e cache limitata. Verificare PNG/SVG e GIF reali, pixel di fit/ritaglio/trasparenza contro gli stessi casi Rust. Non aggiungere richieste remote o lavoro per fotogramma quando l’immagine è ferma.

È un’estensione della gestione dei media del demo, oltre il riallineamento CSS/DOM: da pianificare con l’utente, che richiede di discutere i lavori con nuova decodifica. Nessun renderer o decoder nuovo è stato aggiunto nella correzione grafica.
