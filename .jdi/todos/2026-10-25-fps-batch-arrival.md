# FPS intermittenti: consegna a blocchi PresentMon

## Segnalazione e causa riprodotta

Utente: con 0.2.7 il valore non arriva sempre. La correzione Alt-Tab da sola
non risolve. Analizzati i result.json dei precedenti probe separati Windrose
20261025-003403 e 20261025-003945: ogni riga conserva QPC sorgente e istante
di arrivo al reader, quindi il ritardo è misurabile senza una nuova cattura.

| Traccia | Frame | Ritardo mediano / p95 / massimo (ms) | Poll in primo piano | Buchi vecchio / nuovo calcolo |
|---|---:|---|---:|---|
| 003403 | 4267 | 785,54 / 1264,53 / 1522,54 | 88 | 25 / 0 |
| 003945 | 2893 | 812,87 / 1329,47 / 1576,50 | 75 | 21 / 0 |

I punti replay usano foreground osservato a ogni poll, escludendo i primi tre
secondi. Non sono finestre qualificate integralmente come il precedente
confronto prestazioni e non dimostrano overhead, accuratezza del valore FPS o
validazione hardware dell'adattatore integrato. Provano che il filtro precedente
può scartare frame validi già presenti nei tracciati reali.

## Correzione

Commit 5914d0bb: media del secondo che termina all'ultimo timestamp sorgente,
con freschezza separata MAX_AGE di tre secondi. PID e swap chain restano isolati;
frame futuri o di altri processi non aggiornano l'età. Storia fino a 30 secondi
sorgente, soggetta anche al limite della coda. L'arrivo di dati bufferizzati
non rinnova il timestamp. Allineato il watchdog di ritorno foreground alla
stessa soglia di freschezza; errori reali mantengono il backoff.

Regressione fallisce prima della correzione; dopo passano 97 test sensors,
5 hardware ignorati, clippy all-targets -D warnings e fmt mirati. Test copre
batch con ritardo 1,5 s, stessa lettura senza nuovi dati, scadenza, sorgenti
estranee, timestamp futuri e ritorno di frame nuovi. Guide en/pt-BR, changelog
0.2.8, bozza risposta autore e note locali aggiornati.

## Evidenze e stato

Nella worktree target/logs/fps-integration: batch-regression-red.log,
batch-tests.log, batch-clippy.log, batch-arrival-replay.json. Script di replay
conservato in target/scratch/fps-integration/analyze-arrivals.py; input originali
nel repository principale target/logs/fps-evaluation. Conservare per la prova
hardware ancora aperta. Build locale 0.2.8 riuscita con build-evo.ps1 in 1m55s;
versione CLI, metadati Studio e sensori simulati verificati. Commit versione
6df1559, hash in build-hashes-0.2.8.json. Nessun push o invio all'autore.
Chiesta chiusura del Light 0.2.7 PID 12028 ancora attivo dalla tray prima
di sostituire i binari in dist/bezel. Dist resta alla versione precedente
finché la consegna non è registrata. Prova hardware ancora da confermare.
