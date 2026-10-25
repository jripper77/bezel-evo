# Servizio FPS separato ? stato 25 ottobre 2026

Dettaglio completo: [.jdi/todos/2026-10-25-fps-service.md](../../todos/2026-10-25-fps-service.md).

- Utente approva collector separato e conferma min/max corretti sulla 0.2.10.
- Worktree F:/Dev/bezel-fps-presentmon, fid/fps-presentmon. Installazione reale
  F:/Dev/bezel/dist/bezel. CARGO_TARGET_DIR F:/Dev/bezel/target.
- Servizio Intel ufficiale 2.6 installato e funzionante, client API3.4 senza
  elevazione e senza gruppo Performance Log Users. Due prove 60/30/60 riuscite,
  560/559 frame e scadenza 3s. Log falliti iniziali conservati; non sovrastimare.
- Default servizio, console solo con path esplicito; RTSS fallback. FrameQuery
  registrata prima di StartTracking, QPC originale. Nessuna RPC/drop sotto mutex.
- 103 test sensori, clippy, 46 UI e 16 browser verdi. Build 0.2.11 riuscita;
  installer/ZIP locali finali prodotti; 170 file manifest verificati, hash
  pacchetti copiati in main dist. Eseguibili ancora NON sostituiti.
- Setup opzionale en/it/ptBR e portable, manutenzione copiata nel componente.
  Logging temporaneo rimosso dal registro. Nessun gruppo o startup cambiato.
- 14 template locali rimossi come richiesto, tenuti Pixel V2.1 e Midnight PoC.
  Backup con manifest sotto main target/logs/template-cleanup-20261025-202557.
- Attesa conferma NUOVA chiusura Studio/Light, richiesta asincrona gi? inviata.
  NON copiare finch? manca; app0.2.10 in uso. Dopo copia verificare hash, avviare
  Studio0.2.11 NORMALE senza VerbRunAs e provare Windrose.
- Documenti README/guide/crediti/bozza autore allineati al servizio. Nessun invio,
  push, merge sul ramo base o pubblicazione autorizzati.

## Consegna completata

Utente conferma salvataggio e chiusura di entrambi. Copiati CLI/Studio0.2.11
in main dist/bezel e verificati hash rispetto a ready-0.2.11.json. CLI --version
0.2.11; metadati Studio0.2.11. Backup0.2.10 in target/logs/fps-integration/
before-0.2.11. Aggiunti script setup e payload fps senza ripristinare template
rimossi. Studio PID42968 avviato NORMALE, token non elevato verificato.
Log installed-hashes-0.2.11.json e launched-0.2.11.json. Servizio attivo.
Prossimo: accettazione Windrose sul display hardware, non ancora eseguita.

## Regressione DRG Survivor, diagnosi ancora aperta

Il gioco iniziale PID372 era terminato prima della ripresa utente. La prima
misura dopo conferma puntava quindi a un PID non valido: non usarla come prova.
Rilevato il nuovo PID39932, API diretta:0frame con150/150 campioni in primo
piano (15s), prova valida. Anche console ufficiale Intel elevata non ha creato
righe CSV:105119 eventi ETW persi con tracking completo;8623 con tracking
CPU essenziale;8610 ripetendo CPU essenziale con servizio temporaneamente
fermato e riavviato in finally. Non dimostra un semplice difetto permessi
del client. Non attribuire ancora la causa definitiva a Windows o al driver.
Ulteriori prove workload precedente fallite, log service-proof-normal-41500,
39376,6796; servizio accetta tracking ma non consegna frame. I due successi
precedenti non costituiscono validazione sufficiente del gioco reale.

Logging diagnostico Intel rimosso e servizio ripristinato. Nessuna modifica
agli eseguibili e nessuna nuova versione. Studio chiuso dall'utente; Light
15156 ancora in uso. Non terminare gioco o Light. Utente lamenta consumo
crediti: ridurre esplorazioni ripetute, nessun agente parallelo. Prossima prova
utile: confronto controllato dopo riavvio Windows per separare stato ETW
della sessione da incompatibilita persistente. Riavvio non eseguito: preservare
documenti ed esecuzioni utente. Problema FPS NON risolto.
