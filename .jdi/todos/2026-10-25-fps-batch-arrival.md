# FPS intermittenti: consegna a blocchi PresentMon

## Ripresa: avvio da cartella errata e richiesta DLSS

Nuova segnalazione: conta FPS non parte. Processi osservati Light 13372 e
Studio 11364 erano entrambi 0.2.8 da target/release, non elevati. Verificata
assenza PresentMon in tools/sensors di quella cartella e nelle directory Intel
di discovery, mentre dist/bezel/tools contiene la copia verificata. Binari
CLI target/dist con hash uguali: non occorre un'altra build per questo avvio.
Letta voce HKCU Run 'Bezel Evo' ancora verso target/release/bezel-studio.exe
--hidden; esiste anche BezelEvoLight verso l'installazione in LocalAppData.
Nessuna voce di avvio o scorciatoia modificata, nessun permesso permanente.

Dopo nuova conferma utente di salvataggio/chiusura, riaperto Studio da
dist/bezel: PID 38836 e runtime 10848, percorsi canonici e token elevati
verificati direttamente. Titolo Bezel Evo 0.2.8, rispondente. Monitor osserva
PresentMon 17988 figlio di 10848; all'ultimo punto PID 29164, quindi non
dichiarare stabilità della cattura o lettura FPS verificata. Evidenze
relaunch-0.2.8-processes.json e relaunch-0.2.8-collectors.json nei log del task.

Utente mostra Steam con '214 DLSS' e chiede la stessa indicazione. Verificata
FAQ ufficiale Steam: etichetta DLSS/FSR/FG associata al framerate comprensivo
dei generati, con FPS del gioco separabili. PresentMon Console documenta
--track_frame_type come beta e dipendente da strumentazione gioco/driver;
SDK estratto 2.6.0 espone Application/Repeated/Intel XeFG/AMD AFMF, senza
enum DLSS. Non inferire che il solo SDK permetta o impedisca in assoluto la
funzione; manca una prova mirata della sorgente con i dati necessari attivi.
Nessun suffisso DLSS implementato o promessa di rilevamento universale.

Fonti: https://help.steampowered.com/en/faqs/view/3462-CD4C-36BD-5767 e
https://github.com/GameTechDev/PresentMon/blob/main/README-ConsoleApplication.md.

## Consegna locale

Utente conferma chiusura di Light e Studio. Verificata assenza dei processi,
copiati i due binari 0.2.8 in dist/bezel con hash fonte/copia uguali; evidenza
installed-hashes-0.2.8.json. Studio elevato avviato dalla cartella canonica,
PID 11936. Restano prova Windrose e ritorno da Alt-Tab. Nessun push o invio
all'autore. Le sezioni seguenti conservano la diagnosi e gli stati precedenti.

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
