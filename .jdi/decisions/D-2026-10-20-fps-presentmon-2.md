# D-2026-10-20-fps-presentmon-2: sorgente PresentMon su Windows

Decisione originaria proposta da DarkK3y-v2 nella PR #2, conservata nei commit
originali. Integrazione locale aggiornata il 25 ottobre dopo il confronto
console/API su Windrose.

## Decisione

Console PresentMon facoltativo per i nove sensori FPS, RTSS come ripiego per
il solo frame rate. Nessun servizio Intel installato da questa integrazione.
Conservati i vincoli concordati: nessuna ricerca in PATH e sessione ETW fissa
BezelPresentMon, con stop_existing_session limitato a quel nome.
Un semaforo globale evita che due istanze aggiornate si rubino la cattura.

## Correzioni del ramo di integrazione

PID della finestra in primo piano come candidato e una sola swap chain,
scelta per conteggio recente. L'identificazione automatica rimane euristica.
Tempi QPC originali, richiesta qpc_time_ms, media un secondo e storico massimo
30 secondi. Code limitate e statistiche sospese in caso di perdita di dati.
Lows con almeno 100/1000 frame. Limite CPU/GPU/cap solo stimato quando chiaro.

Console nascosta, job kill-on-close, lettori e code isolati per cattura.
Errori dopo lo spawn passano subito al ripiego; retry limitati e backoff
azzerato solo dopo frame validi. Il job viene associato prima del completamento
dell'avvio; non si sostiene una garanzia atomica fra spawn e associazione.

## Conseguenze

Servono diritti ETW: amministratore o Performance Log Users. Bezel non li
concede automaticamente. Fonte, indisponibilità e dettagli devono essere
leggibili, con messaggi Studio nelle tre lingue.

Autore e validazione hardware originari restano attribuiti alla PR. Le prove
console/API del 25 ottobre non sono una validazione completa dell'adattatore
Rust integrato su ogni gioco. Prima del merge sul ramo principale condividere
modifiche e risultati con l'autore, come concordato. Nessuna pubblicazione
autorizzata da questo documento.
