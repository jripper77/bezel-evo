# Servizio FPS separato — lavoro attivo, 25 ottobre 2026

## Autorizzazione e stato utente

Utente approva la proposta: Studio/Light normali, collector separato con
permessi configurati nell'installazione; non aggiungere l'account a gruppi.
Conferma anche min/max FPS corretti sulla 0.2.10. Installata locale 0.2.10,
Studio PID43328/Light41060 elevati da dist/bezel, documenti da preservare.
Non chiedere ancora l'approvazione di questa architettura: già autorizzata.
Nessun push, pubblicazione o invio all'autore autorizzato.

## Repository

Worktree F:/Dev/bezel-fps-presentmon, ramo fid/fps-presentmon. Main sorgenti
in F:/Dev/bezel resta invariato. CARGO_TARGET_DIR=F:/Dev/bezel/target; junction
worktree target/release verso main target/release. Installazione canonica
F:/Dev/bezel/dist/bezel. Nuovo lavoro servizio NON ancora committato nei sorgenti.

## Implementazione corrente

- Nuovi fps/service_api.rs e service.rs in bezel-sensors. Default usa servizio;
  presentmon_path esplicito conserva backend console legacy. Fallback RTSS.
- DLL scoperta come sibling del servizio PresentMonSharedService interrogando
  SCM in sola lettura, mai PATH/CWD. LoadLibraryEx con ricerca limitata,
  API3.4 verificata, QueryElement ABI32byte, Version40byte, blob offset/size e
  sovrapposizioni validati. FrameQuery/ConsumeFrames (non DynamicQuery cached).
  Metric ID dal SDK2.6.0: [swap1, CPUQPC7, BetweenPresents78, CPUBusy9,
  CPUWait10, GPUBusy14, GPUWait15], statNONE. QPC uint64 da verificare con
  evidenza hardware: il primo probe non ha ancora prodotto dati decodificati.
- Worker unico per sorgente; tutte RPC e drop client fuori dal mutex. Coda8192,
  storia240000 e 30s; generazioni scartano risultati vecchi. Stop controlli non
  bloccano su RPC vendor. In shutdown una RPC in corso può finire dopo il drop
  del worker; non promettere join atomico o cleanup dopo kill. Client RAII libera
  query, stop tracking, session, poi DLL; servizio condiviso non viene ucciso.
- Riusa metrics/minmax1s/lows e freschezza3s validati. Idle10/60s, ritornofocus,
  backoff API2..60s. Nuove stringhe fps.reason.setup in en/pt-BR/it.
- Feature windows-sys LibraryLoader e Services aggiunte; FreeLibrary è in
  Foundation. Clippy is_multiple_of già corretto. 103 test sensori verdi,
  5 ignorati prima di aggiungere il nuovo probe ignorato; clippy verde.

## Componente Windows già installato per prova (autorizzato)

Creati setup-game-fps.ps1/.cmd e prepare-game-fps.ps1. Payload di soli 3 file
Intel dal MSI2.6.0 verificato; niente GUI Intel. Licenza MIT ufficiale copiata
in packaging/windows/PresentMon-LICENSE.txt. Preparazione sorgente amministrativa
MSI /a nel cache riutilizzabile target/scratch/game-fps-package/source-2.6.0;
payload worktree target/scratch/fps-service-payload. Firme Intel valide e hash
hardcoded nello script; script parse e Validate riusciti senza cambi sistema.

Poi eseguito Install con consenso UAC: servizio PresentMonSharedService Running,
Auto, LocalSystem, PID14360, path:
"C:/Program Files/Bezel Evo FPS/PresentMonService.exe" --etw-session-name BezelFpsService.
DLL e XML sibling, script manutenzione e licenza. Registrato componente separato
in HKLM Uninstall/BezelEvoFPS. Account NON membro Performance Log Users e shell
test NON elevata. Non configurati shortcut o startup utente.

Lo script riusa servizio Intel esistente verificato senza sovrascriverlo;
uninstall tocca solo servizio con path nostro e file pinned. Prima della
consegna correggere ordine uninstall: rimuovere record ARP dopo i file, per
consentire retry se DLL caricata. Aggiornare copia script installata con una
nuova chiamata Install idempotente; non fermare servizio se payload identico.

## Prove in corso e prossimi passi

Build debug CLI completata. Nuovo test ignorato esplicito in service_api.rs:
installed_service_streams_fresh_frames_without_elevation, PID da
BEZEL_FPS_TEST_PID. Richiede solo il workload D3D posseduto, non uccide giochi.
Primo tentativo log service-proof-normal-18900.log: API open/query nessun errore,
ma insufficient real frame events. Il caller aveva lanciato workload senza
argomento, quindi30s invece12 e lo ha terminato posseduto dopo test18+wait5;
stdout vuoto per terminazione anticipata. Non usare come prova positiva.

Secondo tentativo tool exec session76649 IN CORSO: ora workload argomento12,
CREATE_NO_WINDOW come i probe precedenti (il workload crea propria finestra
topmost nonattivante); test18s. Aggiunti diagnostici cfg(test) raw_frames,
first_values e dump ABI u64/f64. Leggere output con write_stdin session76649.
Log service-proof-normal-<PID>.log, workload-retry.stdout.log. Test deve mostrare
>=100 frame, rate60/30/60 e scadenza oltre3s dopo fine workload. Se raw>0 ma
decoded0 verificare tipo QPC/metric IDs tramite introspection SDK invece di
inventare conversioni. Se raw0 verificare ETW del servizio/workload e API query.

Ancora da completare:
- prova positiva frame-query da processo normale + ripristino FPS hardware.
- test decodifica/errore ABI e lifecycle extra se servono, clippy/fmt crate.
- Packaging: aggiungere payload+script a manifest package-evo.ps1, checkboxFPS
  opzionale e retry nel menu Start, LangString en/it/ptBR (attuali solo en/it;
  aggiungere PortugueseBR alle lingue NSIS), compilare installer locale.
- Fonte preparazione cache già validata; nessun download in avvio app. Solo
  preparazione pacchetto scarica MSI se cache assente. Licenza/THIRD_PARTY.
- Guide/README/bozza autore da riscrivere sul default servizio (contributo
  originale sempre accreditato, console avanzata resta), niente DLSS inventato.
- Nuova versione almeno0.2.11 per eseguibili; build-evo, poi chiusura salvata
  Studio/Light prima copia. Nuovo lancio normale (senza VerbRunAs) deve provare
  obiettivo; non cambiare gruppi account. Nessuna full suite finché non push.

Tutti i diagnostici in target/logs/fps-integration, nessuna pulizia ampia.
