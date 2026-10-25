# FPS senza elevare Studio e Light

L'utente approva il collector separato e conferma min/max corretti sulla 0.2.10.
Non modificati gruppi Windows, avvio automatico o collegamenti utente. Nessun
push, pubblicazione o messaggio inviato a DarkK3y-v2.

## Implementazione

Default: servizio ufficiale Intel PresentMon 2.6.0, API 3.4 FrameQuery.
La console resta un override con presentmonPath esplicito. Worker singolo,
RPC e chiusure fuori dal mutex, generazioni per cambio PID, code limitate,
freschezza sul QPC originale e scadenza 3 s. Riusa min/max su finestre 1 s.
Il servizio condiviso non viene terminato dai client Bezel. La chiusura non
attende una chiamata vendor bloccata: il worker chiude la sessione al ritorno.

Setup opzionale con consenso amministratore una volta, script portabile e
voce riprova nel menu Start. Payload MSI ufficiale verificato per hash e firma,
solo servizio/API/XML, licenza MIT e avvisi dipendenze; nessuna GUI Intel.
Servizio compatibile esterno riutilizzato, conflitti preservati. Componente
con propria voce App Windows, indipendente dalla disinstallazione Bezel.
Rimozione limitata a servizio posseduto e file noti; nessuna cancellazione ampia.

Installato C:/Program Files/Bezel Evo FPS, PresentMonSharedService automatico
LocalSystem. Le opzioni ImagePath non arrivano al callback SCM del servizio
Intel: per nuove installazioni usare default ufficiale PMService senza flag.
Nessuna impostazione di logging diagnostico rimasta nel registro.

## Verifiche

- Sensori: 103 test passati, 6 ignorati; clippy crate/all-targets e fmt passati.
- UI: 46 unit test e 16 casi browser PresentMon passati.
- Script PowerShell nuovi/modificati analizzati e payload firmato verificato.
- Prove normali senza elevazione e senza Performance Log Users: log
  service-proof-normal-592.log (560 frame, 29,99–60,04 FPS) e
  service-proof-normal-17780.log (559 frame, 29,99–60,07 FPS), sotto
  target/logs/fps-integration. Primo evento circa 287/301 ms; scadenza verificata.
- I tentativi iniziali non ricevevano eventi; dopo registrazione query prima
  del tracking e riavvio del servizio posseduto, entrambe le prove riuscite.
  Conservati log falliti; causa iniziale non isolata. Non dichiarare collaudo
  esteso o accettazione Windrose del nuovo adapter già completati.
- Build-evo 0.2.11 riuscita, CLI e metadati Studio verificabili. Pacchetto NSIS
  e ZIP compilati localmente; seconda generazione allinea ultime stringhe/script.

## Pulizia template richiesta dall'utente

Nella sola libreria locale rimossi 7 documenti personali e 7 cartelle modello
incluse nell'installazione. Conservati Pixel V2.1 (Pixel-3.bezeltheme, 58 elementi)
e Midnight 8.8" PoC (60 elementi). JTR rimosso aveva 0 elementi. Gli altri temi
rimangono. Recupero: F:/Dev/bezel/target/logs/template-cleanup-20261025-202557,
manifest.json contiene percorsi originali/destinazione. Il catalogo distribuito
nel repository non è modificato: un futuro installer completo può ripristinare
i modelli inclusi. Non eseguita pulizia di cache o runtime.

## Passo richiesto per consegna

Domanda asincrona già inviata: salvare e chiudere Studio/Light prima di copiare
0.2.11 in F:/Dev/bezel/dist/bezel. Attendere conferma; precedente chiusura era
per 0.2.10. Dopo copia verificare hash e avviare Studio normale, senza RunAs.
Servizio già funzionante, nessuna nuova conferma architettura necessaria.
Verificare su Windrose, poi annotare esito. README e risposta autore aggiornati;
bozza non inviata. Full suite solo prima di push, non richiesta adesso.
