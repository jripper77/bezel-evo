# Pubblicazione 0.2.12 - 26 ottobre 2026

Utente autorizza commit, push GitHub, README e release notes dalla precedente
release pubblicata; risposta a DarkK3y e gestione PR da discutere dopo.
Nessun messaggio al contributore da inviare automaticamente.

- Ultima pubblicata v0.2.0. Locali 0.2.1-0.2.11, nuovo pacchetto0.2.12.
- Integrato fid/fps-presentmon nel repository principale con merge --no-ff.
  Originale head PR2 4b4f473c presente come antenato, autori preservati.
- PR2 risultava aperta prima del push: verificare stato dopo aggiornamento base.
- README/changelog e guide FPS aggiornati. Recupero DRG dopo reboot descritto
  come riscontro utente, senza dedurre causa certa o validazione prolungata.
- Suite completa avviata una volta: sessione53206 fmt/clippy/llvm-cov,
  sessione3059 npm test. Output target/logs/release-0.2.12.
- Utente conferma Studio e Light salvati e chiusi, provenivano da target/release.
  Non avviare installer o app senza richiesta: pubblicazione non e installazione.
- Cache PresentMon disponibile nel worktree target/scratch/game-fps-package;
  passare MSI locale a package-evo per evitare download inutile. Libre principale
  target/package-libre. Dist/bezel e template personali da preservare.
- Proseguire: esiti verifiche, package-evo assoluto (incrementa VERSION), note
  cumulative, commit release, push windows-improvements e HEAD:main solo FF,
  tag annotato v0.2.12 e release con3asset. Verificare Latest e PR2.
- Non includere img/ preesistente non tracciato. Nessun agente parallelo.

## Verifiche completate e pacchetto pronto

- Fmt e clippy workspace/all-targets passati. Copertura Rust89,80%.
- Suite UI iniziale695casi:694passati e1aspettativa obsoleta (video preparabile
  offline). Corretto solo il test, rieseguito il suo modulo:18passati. Nessuna
  nuova modifica prodotto. Copertura UI99,22%; non ripetuta tutta la suite.
- Tutti1232casi browser passati in11,6min con4worker, nessuna spec da ripetere.
- package-evo assoluto0.2.12 riuscito. CLI --version e metadati Studio0.2.12.
  Manifest ZIP170file, architettura x64, script setup e SHA256 verificati.
- Cache Intel preparata dai file ufficiali gia verificati nel worktree, con
  nuova verifica pin/firme. Nessun download duplicato e nessun setup eseguito.
- Note cumulative dist/Bezel-Evo-0.2.12-release-notes.md dalla0.2.0, tre download,
  avviso SmartScreen, crediti e limitazioni ETW/DLSS.
- Ultimo fetch conferma origin/main e origin/windows-improvements antenati di
  HEAD; pubblicazione solo fast-forward. Per esito pubblicazione vedere STATE.md
  locale e la release GitHub, verificati dopo l'upload.
