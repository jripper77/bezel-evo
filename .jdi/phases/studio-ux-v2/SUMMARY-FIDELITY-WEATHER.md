# Fedeltà — Meteo

22 ottobre 2026. Branch `fid/inspector-weather`, base `b324b2ff`.
Implementazione: `3d37a0a`.

## Interventi

- Ricerca della località con `searchField` comune e comando iconico affiancato; Invio avvia la stessa ricerca del pulsante. Risultati reali del bridge con `listCard`, nome e regione/paese, senza copiare le due città di esempio della tavola. La località selezionata viene salvata con una sola modifica.
- Unità e lingua affiancate; dimensione icona e distanza dal testo affiancate con campi numerici compatti, icone e unità. Il valore automatico e i limiti restano invariati.
- Icona della condizione con toggle iniziale a sinistra; `toggleField` è esteso con `leading`, conservando comportamento e stile comuni. Stili a quattro miniature del renderer, didascalia semplice senza titolo Aspetto ripetuto.
- Testo prima dell’icona nella sezione Colore; colore proprio solo per Contorno/Piena, ereditarietà e ripristino mantenuti. Ferma/Leggera/Vivace con icone del catalogo e senza intestazione duplicata.
- Nessun campo del tema, runtime o renderer modificato; nessuna build/installazione/push. Nessuna chiave i18n aggiunta, tutti i testi restano tradotti.

## Verifiche

- **57/57 unitari mirati verdi**: weather-motion, weather-widget, i18n, static-guards. Log `target/logs/fidelity-weather/unit-final.log`.
- **44/44 e2e verdi**: weather, inspector-media, features-016-inspector nei quattro progetti chiaro/scuro en/pt-BR. Log `target/logs/fidelity-weather/e2e-final.log`.
- Nuovo caso: ricerca e selezione da tastiera, un Undo ripristina la località, dimensione automatica ripristinabile, unità/lingua e dimensione/distanza alla stessa altezza, assenza di overflow a 320 px, icone e accessibilità. Nessuna asserzione esistente indebolita.
- Nella prima esecuzione l’icona del comando ricerca aveva un identificatore inesistente; i controlli console l’hanno rilevato. Corretto con `arrowRight` già nel catalogo; l’ultima esecuzione è tutta verde.
- Catture italiano chiaro/scuro in `evidence/fidelity-weather/`, confrontate con WeatherInspectorOpen. La ricetta apre i risultati con la ricerca vera del demo, senza risultati inventati nel prodotto.

## Differenze motivate e residui

- Il risultato usa `listCard` comune con regione/paese sotto il nome e chevron, anziché una seconda riga di scelta specifica per Meteo. Il nome accessibile include tutti i dati del risultato. La ricetta mostra un risultato perché è quanto restituisce il bridge demo, non le due località del disegno.
- «Al cambio di meteo» non aggiunge un interruttore: D-2026-10-20-studio-ux-v2-09 e weather-motion.md prescrivono la dissolvenza incrociata automatica di 300 ms. Il modello non prevede di spegnerla; il suggerimento esistente lo dichiara. Resta la differenza esplicita dalla tavola, senza comando fittizio o modifica della decisione.
- Restano Lancetta/Orologio analogico, contenitori/Tema/multipla, shell/pannelli, Profili/Galleria e revisione finale delle ricette/fixture. La galleria generale è ancora quella di T-12 e va rigenerata alla fine; non è richiesta ora una verifica manuale dell’utente.
