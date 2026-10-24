# Valutazione PresentMon console/API — 25 ottobre 2026

## Ambito

L'utente autorizza la prova comparativa e sceglie Windrose. Strumenti separati sotto
scripts/windows/fps-evaluation; artefatti in target/scratch/fps-evaluation e
target/logs/fps-evaluation. Nessuna modifica al prodotto 0.2.5, ai documenti
aperti o al display. PR #2 invariata; nessun commento, merge, push o rilascio.

PresentMon v2.6.0 ufficiale: digest del rilascio e firme Intel verificati. MSI
estratto con /a dopo verifica delle sole azioni standard in AdminExecuteSequence;
nessuna installazione. Servizio e API estratti dallo stesso MSI, API 3.4.0.
Il prodotto dovrebbe usare il loader Intel del servizio installato.

## Prove effettuate

- smoke-user: console senza diritti ETW rifiutata; API apre la sessione ma un
  raccoglitore non elevato non inizia il tracking.
- 20261025-000830: **serie iniziale non valida per il confronto**. Finestra
  occlusa, in alcuni run solo 15 righe, arresto incompleto del servizio. Il
  driver iniziale scriveva complete anche con uscita 1: non significa successo.
- 20261025-001442/smoke: finestra D3D11 non attivante e sempre visibile,
  ciclo 60/30/60 FPS ogni quattro secondi; API con flush ETW 100 ms. Entrambi
  seguono le variazioni e i raccoglitori escono con codice 0.
- 20261025-001815: raccoglitore elevato, lettore API con utente normale.
  Lettura riuscita; servizio arrestato correttamente (codice 0, graceful).

### Risultati brevi corretti — 12 secondi

| Misura | Console | API/servizio |
|---|---|---|
| FPS presentati: minimo delle finestre | 29,986 | 29,990 |
| FPS presentati: massimo delle finestre | 60,032 | 60,044 |
| Mediana delle finestre FPS | 59,980 | 59,986 |
| Campioni | 590 righe, stesso PID | 47 finestre valide |
| Copertura console | 11,816 s | finestre 1 s, polling 250 ms |
| Primi dati validi API | non confrontabile ancora | circa 250 ms dalla richiesta |
| Picco working set raccoglitore | 13,4 MiB | 109,1 MiB |
| Picco working set lettore Python | 19,3 MiB | 22,1 MiB |
| CPU raccoglitore + lettore | circa 1,06% di un core | circa 1,06% di un core |

CPU normalizzata sui 20 processori logici circa 0,053% in entrambi: durata e
granularità dei contatori insufficienti per distinguerli. Working set include
pagine condivise; non misura memoria privata/proporzionale. Crescita iniziale
non interpretata come leak. Il tempo dei primi dati esclude l'avvio del servizio.
Nessuna misura di perdita di FPS del gioco da questo carico leggero e cadenzato.

Lettore normale separato: 29,986..60,023 FPS presentati, primi dati circa 251 ms.
Frame applicazione/visualizzati interrogati separatamente, senza prova FG.

## Riscontri tecnici e limiti

- Analisi usa CPUStartQPCTimeInMs / 1000: nel CSV 2.6.0 con qpc_time_ms il nome
  storico TimeInSeconds contiene valori in millisecondi. Niente unità indovinate.
- Finestre separate per PID/swap chain, tempi sorgente; arrivo registrato nel
  thread lettore. Righe iniziali con swap chain zero escluse. Assenze/non finiti
  rimangono mancanti. Tutti i dati grezzi conservati.
- Servizio temporaneo: pipe/sessioni uniche, ping/quit per arresto, timeout
  limitato ai figli propri. Nessuna sessione esterna fermata. Nessun servizio
  Windows registrato; dopo le prove nessun processo di cattura/carico rimasto.
- Avviso ufficiale relativo alla directory opzionale PresentMonServiceEtl:
  modalità console elevata incontra ACL solo SYSTEM. TEMP/TMP non reindirizzano
  quella directory. Query live riuscite; export ETL e servizio LocalSystem
  installato non verificati. ACL e directory condivisa non modificati.
- Solo chiavi registro nuove e ancora vuote rimosse dal driver. Dati/chiavi
  preesistenti o non vuoti preservati. Non promettere pulizia dopo crash.
- Nel risultato grezzo del primo lettore normale session_name contiene un nome
  generato dal client; il nome effettivo è nel connection.json del server. Il
  prototipo è stato corretto per le prove successive, senza riscrivere i grezzi.

MSVC /W4 /WX verde; quattro test Python su tempi, attribuzione, swap chain e
assenze/non finiti; parser PowerShell e diff --check verdi. Nessun crate
modificato: nessuna suite completa del prodotto eseguita.

## Decisione provvisoria e seguito

Entrambe praticabili. API: dati strutturati e lettore non elevato; costo in
working set maggiore. Console: candidata seria per un'integrazione leggera.
Non scegliere ancora dai soli dati sintetici.

Mancano Windrose nella stessa scena, ripetizioni, FG se supportata, idle/prova
lunga, recupero, fallback RTSS e installazione/aggiornamento ufficiale. Chiesta
conferma asincrona di scena pronta e stato FG; Windrose non rilevato al momento
di questo verbale. Driver predisposto con -GameProcessId, poi lettore normale
--connect. Attendere la scena pronta prima della fase dipendente.

## Seguito completato su Windrose

Eseguite prove FG attiva e spenta con consenso dell'utente e focus del PID
registrato. Risultati e qualificazione delle finestre nel verbale
2026-10-25-windrose-fps.md. Entrambe praticabili, console più leggera; beneficio
API per separazione nativi/generati non dimostrato nel caso provato. Indicazione:
console corretta con fallback RTSS per il contatore principale, API eventuale
evoluzione. Nessuna integrazione/merge/rilascio eseguiti in questa fase.
