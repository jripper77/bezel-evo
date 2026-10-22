# Fedeltà — Immagine, Forma e Player

22 ottobre 2026. Branch `fid/inspector-media`, base `d9bd198`.
Implementazione: `616a7be`.

## Interventi

- `imageField` ora ospita la scheda compatta del file, con miniatura disponibile, nome, metadati effettivamente ricevuti e scelta nativa «Cambia…». La miniatura apre il pannello Media esistente. Nessuna dimensione inventata: si leggono le dimensioni intrinseche dell’immagine quando è caricata. Il vecchio uso del primitivo resta compatibile.
- `fadeField` condiviso da Immagine, Forma e copertina Player: campi affiancati, anteprima 64×64, direzione/punto medio, opacità iniziale/finale; centro ed estensione per Radiale. Il trascinamento aggiorna solo l’anteprima e salva al rilascio con un Undo. Annullamento senza modifica; frecce della tastiera supportate con nome accessibile. Geometria della sfumatura coerente con `fadeLine` e renderer, senza campi nuovi nel tema.
- Immagine: miniature Adatta/Copri/Allunga, selezione legacy senza adattamento conservata, ancoraggi esistenti, controlli GIF con icone e senza titoli ripetuti.
- Forma: Sagoma con icone e comando finestra-video affiancato; angoli prima del bordo; campi scala/spostamento sulla stessa riga e scheda immagine comune. Il colore del bordo resta disponibile con spessore positivo.
- Player: scelte a icone quando non suona, copertina/angoli/distanza raggruppati, trasparenza prima del progresso; posizione dei tempi, stile della barra, spessore/estremi/pallino compatti. Le opzioni restano quelle effettivamente gestite dal renderer.
- Riutilizzati `fieldAction`, `fieldGrid`, `iconChoice`, `iconButton`, `previewChoice`, `cornersField`, `propertySection`, `imageField`, `fadeField`; stili comuni in base.css. Due regole identiche preesistenti di Schermo/Media accorpate al medesimo stile base. Nuove chiavi solo nell’area inspector, con parità en/pt-BR/it.

## Verifiche

- **76/76 unitari mirati verdi**: shape-fade, media-inspector, player-parts, inspector-model, inspector-sections, i18n e static-guards. Log `target/logs/fidelity-media/unit.log`.
- **72/72 e2e verdi** nell’ultima esecuzione: inspector-media-controls, player-parts, shape-image, device-video, inspector-media, corner-radii, canvas-shape-image e features-016-inspector, nei quattro progetti chiaro/scuro en/pt-BR. Log `target/logs/fidelity-media/verified-area.log`.
- Nuove verifiche: un solo Undo per trascinamento, nessuna modifica dopo pointercancel, tastiera radiale senza perdita dell’estensione, contenimento a 320 px, metadati della vera anteprima SVG e apertura Media.
- Migrati i ruoli slider→spinbutton di punto medio/opacità; nessuna asserzione indebolita. Le tre azioni Adatta al contenuto di player-parts passano dal menu comune: il pulsante nel pannello era già rimosso da T-12 nella base `d9bd198`. La scelta estremi è ora esclusiva Arrotondati/Piatti, verificando entrambi gli stati.
- Le prime prove del nuovo test file hanno scoperto una preparazione errata: doppio clic su una riga Media non inserisce un’immagine; le azioni della riga richiedono hover. Inoltre il file PNG dimostrativo non contiene dati immagine. Il test usa ora un asset SVG effettivo, senza falsificare i metadati. Gli errori erano nel nuovo test, non regressioni di spec preesistenti.
- Dieci catture italiano chiaro/scuro con shoot.mjs, esaminate per Forma, Player, Immagine/SVG, Immagine/raster e GIF: `evidence/fidelity-media/`. Nuovo stato raster nella ricetta e nella galleria, mantenendo separata la copertura SVG.
- Nessun Rust o renderer modificato; nessuna suite completa, build, installazione o push.

## Differenze motivate e residui

- Disegno proporzionale degli Angoli conservato come impone D-22: occupa più spazio della sola icona/raggio della tavola. Distanza copertina nella riga successiva per non comprimere il controllo comune. Gli avvisi reali sulle sessioni multimediali restano.
- L’anteprima della trasparenza usa il colore di accento del tema UI, senza copiare l’immagine campione magenta del mockup; direzione e rampa di opacità sono quelle del modello effettivo.
- La cattura raster usa il PNG dimostrativo esistente, privo di dataUrl: mostra quindi nome/formato e segnaposto, senza dimensioni o miniatura fittizie. La cattura SVG verifica invece l’anteprima e i metadati disponibili. Migliorare le fixture raster durante il blocco finale delle ricette rimane aperto.
- Restano da riallineare Meteo, Lancetta, Orologio analogico, Card/Tema/multipla, shell/pannelli e Profili/Galleria; la galleria finale non è ancora rigenerata. Il lavoro complessivo rimane aperto, senza chiedere ora verifiche manuali all’utente.
