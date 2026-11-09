# update-check — riepilogo

Fatto tutto il piano. Light: thread `bezel-update-check` (primo passaggio dopo 60 s, poi ogni 60 s; richiesta a GitHub solo se sono passate 24 h dall'ultima riuscita). Il risultato va in `update.json`, la voce «Aggiornamento disponibile: X.Y.Z» compare nel tray e apre la pagina fissa delle release. Studio: sezione Aggiornamenti nelle Preferenze, con l'interruttore, il testo sulla riservatezza e la versione trovata con il pulsante «Apri la pagina di download».

Verifiche: test di dominio (3), runtime (1), Light (3), Studio backend (1) e lib completa 250, JS bridge e demo, i18n 35, e2e update-check + debug + gif-search (32). Clippy `-D warnings` e fmt puliti su core, runtime, bezel e Studio. Prova reale dell'API: risponde `v0.2.24` (nessuna voce con la 0.2.25 installata, corretto).

Non coperto dai test automatici: la riga «versione trovata» nelle Preferenze (il demo non simula una release nuova) e la voce del tray. Si verificano a mano alla prossima release pubblicata.
