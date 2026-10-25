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
