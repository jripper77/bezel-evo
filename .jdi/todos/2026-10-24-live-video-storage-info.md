# Video di sfondo e Live hardware — 24 ottobre 2026

## Causa individuata

L'utente conferma che la 0.2.3 apre JTR2 Black nel canvas e spegne Live correttamente, ma riattivare Live non presenta il tema sul display quando contiene il video di sfondo.

Il percorso ThemeRuntime::start_video -> on_device -> find_video chiede ScreenStorage::info per scegliere tra flash interna e scheda. Il Link asincrono del presenter gestiva size/resume/stop ma rispondeva Unsupported a info. L'errore bloccava l'invio hardware prima del primo frame. I temi a tinta unita o con video già selezionato come deviceVideo non passavano dallo stesso percorso.

## Correzione

Commit 0881553: nuovo comando Info sul worker del display; richiesta non bloccante dal motore, risposta memorizzata e stato invalidato alla riconnessione. Una sola richiesta in coda, con lo stesso limite delle altre query. Nessun upload o cambiamento dei file sul dispositivo.

## Prove

- Nuovo test a_bundled_video_missing_on_the_display_still_presents_the_theme: falliva sulla versione precedente (presentation did not settle), passa dopo la correzione.
- Casi distinti: clip assente senza scheda, clip assente con scheda, clip già memorizzata. Verificati i frame effettivamente presentati, la destinazione interna/scheda, una sola query Info, assenza di modifiche allo storage nella ricerca e avvio della clip già presente.
- cargo test -p bezel-runtime --locked: 72 passati, uno ignorato perché richiede un tema locale.
- Clippy runtime --all-targets -D warnings, fmt mirato e diff --check verdi.
- Nessun modulo UI o altro crate modificato; nessuna suite completa ripetuta.

## Build e consegna

Build Windows 0.2.4 avviata con build-evo.ps1. Utente ha confermato salvataggio e chiusura di Studio/Light; controllo processi senza eseguibili aperti prima del completamento. Nessun processo terminato forzatamente, nessun push/installazione/pubblicazione. Conferma visiva del display reale ancora da acquisire dopo il riavvio.
## Build completata

Build-evo.ps1 completata in 1m46s senza retry dopo la chiusura dell'utente. CLI e metadati Studio 0.2.4 verificati; commit versione c0b5318. Hash CLI EF2A893585F32B7D63750C97BFBF456786937904C3105B53CF22FF75F99FA90C; Studio 051A6940747C0B4781C86E8C4E783611ACB308999402F85C81374F7BFC1E617B. Verbale in target/logs/live-video-storage-info/build-verification-0.2.4.json. Studio riavviato da target/release (PID 24740). Prossimo: utente importa JTR2 Black, attiva Live e conferma il display fisico. Se la clip manca sul dispositivo, la copertina e gli oggetti devono arrivare comunque; il trasferimento resta esplicito con «Invia allo schermo». Nessun file caricato automaticamente nello storage dei dispositivi, nessun setup/installazione/push/pubblicazione.