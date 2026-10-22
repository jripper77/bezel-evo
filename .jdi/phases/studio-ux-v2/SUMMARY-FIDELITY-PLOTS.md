# Riallineamento — Grafico e Spettro

22 ottobre 2026. Branch `fid/inspector-plots`, base `059b937`.

## Fatto

- Grafico: riepilogo Precisione richiudibile, durate rapide sempre visibili, frase di durata/campionamento e vincoli conservati dentro il dettaglio. Apertura ricordata per oggetto nello stato UI, senza campi nel tema. Unificata l’asse temporale con le etichette del righello, eliminata la ripetizione.
- Serie e scala con didascalie semplici, icone Una/Due a specchio; spessore e distanza affiancati, reset della larghezza facoltativa conservato. Riempimento Area come numero percentuale compatto, con Undo singolo, invece dello slider esteso.
- Spettro: numero barre/frequenze sulla stessa riga, spessore/spazio affiancati, sorgente audio e movimento con icone del catalogo. Le cinque miniature usano etichette corte Specchio/LED e conservano i nomi completi per tooltip e accessibilità, per evitare parole troncate o spezzate. Questa abbreviazione è l’unica differenza lessicale rispetto alla tavola.
- Stili delle miniature condivise trasferiti in base.css. Estesi `fieldGrid` (colonne) e `previewChoice` (didascalia breve, nome completo), senza secondo componente.
- Nuovo `detailsField`, disclosure HTML nativo per dettagli secondari, riusabile. Nuove chiavi a parità en/pt-BR/it.
- Ricetta aggiuntiva `ux2-graph-area` con CPU/GPU reali nel demo, collegata alla galleria accanto al caso barre arrotondate. Nessun dato dimostrativo nel codice di prodotto. La galleria finale dovrà essere rigenerata al termine delle altre correzioni.

## Verifiche

- **72 unitari mirati verdi:** i18n, static-guards, graph-inspector, spectrum, inspector-helpers.
- Prima esecuzione degli e2e esistenti: **99/100 verdi**, un errore SVG durante l’introduzione dell’icona musica; aggiunta al catalogo. Poi **56/56 verdi** nel gruppo Spettro/media/nuovi casi.
- La verifica visiva ha scoperto le frequenze ancora impilate: `Object.assign` non applicava la variabile CSS della griglia. Corretto con `style.setProperty` e aggiunta asserzione geometrica (stessa coordinata verticale per i tre campi). Dopo la correzione **28/28 verdi** per audio-spectrum e inspector-plots, incluso «24000» completamente leggibile a 320 px.
- Grafico definitivo: **32/32 verdi** per inspector-preview e inspector-plots. Migrati soltanto l’apertura esplicita di Precisione e il ruolo slider→spinbutton del riempimento; tutte le verifiche di frase, unità, picchi, limiti, renderer e Undo mantenute. Nuovi test verificano apertura conservata, Undo del campionamento/riempimento, frequenze e icone.
- Le catture finali italiano chiaro/scuro sono in `evidence/fidelity-plots/`; esaminati Grafico Area e Spettro, poi corretta anche la dicitura che si spezzava nelle cinque miniature.
- Complessivamente 108 casi distinti hanno esito verde nelle esecuzioni mirate; non è una nuova suite completa del prodotto. Ultimo gruppo dopo le didascalie registrato nel log `target/logs/fidelity-plots/e2e-caption-final.log`.
- Nessuna modifica al formato del tema, runtime o renderer; nessuna build, installazione o push.

## Residui

Restano media/trasparenza, Lancetta/Orologio analogico, Card/Tema/multipla, shell/pannelli e Profili/Galleria. Conservare gli avvisi funzionali sul Player mancante e sulla sorgente audio: non scambiarli per dati campione delle tavole. Il riallineamento completo non è consegnato e non è richiesta ancora la verifica manuale dell’utente.
