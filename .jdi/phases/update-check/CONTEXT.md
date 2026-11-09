# update-check — contesto

Obiettivo: segnalare una release più nuova pubblicata su GitHub, senza scaricare né installare nulla.
Decisioni bloccate: D-2026-11-09-update-check-1, -2, -3 (concordate in chat il 2026-11-09: «vai, facciamo il primo passo»).

## Definition of Done
- Light controlla al massimo una volta al giorno e solo con `updateCheck` attivo; con l'interruttore spento non fa richieste.
- Voce nel tray con la versione nuova; il clic apre la pagina fissa delle release.
- Preferenze › Aggiornamenti: interruttore (attivo di default), testo su cosa viene inviato, versione trovata con il pulsante per la pagina di download.
- Testi in en, it, pt-BR con parità di chiavi.
- Test: dominio, file condiviso, passo di controllo di Light, backend di Studio, bridge, demo, e2e delle Preferenze.
- Manuale: con una release più nuova pubblicata compaiono la voce nel tray e la riga nelle Preferenze.
