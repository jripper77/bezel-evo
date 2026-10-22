# Fedeltà — Lancetta e Orologio analogico

22 ottobre 2026. Branch `fid/inspector-instruments`, base `17f24d10`.
Implementazione: `9cf8f43`.

## Interventi

- Lancetta: arco e estremità affiancati, numeri della scala con campioni ricavati dai veri estremi del sensore e didascalia corta «tutti», distanza nella stessa riga; quattro miniature distinte per la forma. Lunghezza/coda/spessore su una riga, cappuccio a icone con diametro affiancato. Nomi accessibili completi conservati.
- Zone: colore, estremo iniziale, estremo finale e rimozione sulla stessa riga, con campi reali modificabili e selettore colore comune. Rimosso il vecchio stile locale a due righe: usa `fieldGrid` e i primitivi esistenti.
- Valore al centro in Forma, accanto al toggle e alla disclosure comune «Modifica testo centrale». L’editor completo delle parti e degli stili rimane disponibile; la disclosure conserva l’apertura durante modifiche e Undo. Anche l’immagine facoltativa è in Forma e riusa la scheda `assetField`/`imageField`.
- Orologio: icone distinte Ore/Minuti/Secondi, lunghezze affiancate, forma delle lancette a icone con bordo affiancato. Colori delle tre lancette su una riga con selettori comuni; movimento dei secondi con icone, senza intestazione ripetuta. Restano selezione della data e lingua effettive.
- Estesi i primitivi condivisi: `numberField` accetta `caption` per una didascalia corta mantenendo il nome completo; `fieldGrid` accetta una lista di larghezze per le colonne e allinea i controlli in basso anche con didascalie su più righe. Nessuna seconda implementazione delle righe o dei campi.
- Tre chiavi tradotte con parità en/pt-BR/it. Aggiornata docs/user/studio-editing.md per l’editor del valore centrale.
- Nessun cambiamento a temi, motore o renderer; nessuna build/installazione/push.

## Verifiche

- **65/65 unitari mirati verdi**: needle-look, clock-format, inspector-model, i18n, static-guards. Aggiunta verifica della classificazione di centro/immagine della Lancetta in Forma. Log `target/logs/fidelity-instruments/unit-final.log`.
- **112/112 e2e verdi**: inspector-instruments, inspector-linked, canvas-property-handles, canvas-clock-handles, clock, inspector-media nei quattro progetti chiaro/scuro en/pt-BR. Log `target/logs/fidelity-instruments/final-area.log`.
- Nuovi test a 320 px: numeri su una riga, editor centrale aperto dopo modifica/Undo, zone modificabili e rimozione reversibile, icone distinte, lunghezze affiancate, colore modificabile, assenza di overflow e accessibilità.
- Migrato il solo passaggio di inspector-linked che apriva Testo per il centro: ora apre Forma e la disclosure; tutte le asserzioni su parti/cifre fisse/Undo restano. Il resto dell’area e i controlli canvas sono verdi senza indebolimenti.
- Quattro catture italiano chiaro/scuro in `evidence/fidelity-instruments/`, esaminate contro NeedleInspectorOpen e la metà analogica di ClockInspectorOpen. La prima acquisizione ha individuato la scelta «Tutte le principali» su una seconda riga; corretta con didascalia breve e campioni degli estremi senza spazi superflui, poi verificata geometricamente a 320 px.

## Differenze motivate e residui

- Il disegno della Lancetta mostra soltanto il toggle del valore centrale e descrive le opzioni del Valore. La disclosure aggiunge un accesso concreto a tutte quelle opzioni, usando `detailsField` già comune; niente pagina o editor alternativo.
- Le lunghezze delle tre lancette dell’orologio e la lingua della data restano esposte: sono proprietà già supportate e collegate anche alle maniglie del canvas, benché non dettagliate nella tavola. Sono ora raggruppate in modo compatto.
- Le icone e i campi rispettano il tema UI reale. Nessun valore di esempio del mockup diventa una lettura o una configurazione fittizia.
- Restano Card/gruppi/facce, Tema e multipla, shell/pannelli, Profili/Galleria e revisione finale delle ricette/fixture. `evidence/final/` resta alla versione T-12: rigenerare l’intera galleria dopo gli altri blocchi. Non è richiesta ancora la verifica manuale dell’utente.
