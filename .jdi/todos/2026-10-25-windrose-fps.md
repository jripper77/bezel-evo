# Windrose: confronto console/API con Frame Generation — 25 ottobre 2026

## Scenario

Utente conferma Windrose pronto, FG attiva. Processo effettivo 37864,
Windrose-Win64-Shipping.exe; launcher 30132 escluso. Scena e stato FG dichiarati
dall'utente, non verificati entrando nelle impostazioni del gioco. Nessuna
modifica al gioco, a Bezel, allo storage degli schermi o alla PR #2.

## Prima acquisizione: 20261025-002901

45 s baseline di contatori CPU/memoria, 45 s console, 45 s API con lettore
normale e raccoglitore elevato. La baseline non misura FPS senza raccolta ETW.
Console: 5902 righe, mediana finestre 170,63 FPS presentati, un solo swap chain.
API: dati arrivati dopo 16,55 s, poi valori congelati ~103,77 FPS presentati,
displayed zero. Utente conferma di essere passato alla chat/desktop durante
l'ultima fase. Serie esclusa dal confronto prestazionale/FG.

Riscontro utile: una query dinamica riuscita può continuare a restituire dati
vecchi quando il gioco non produce frame visibili. Il prodotto deve distinguere
disponibilità e freschezza: nessun numero congelato presentato come misura live.
Non attribuire questo caso a un difetto universale del servizio.

## Ripetizione con focus registrato: 20261025-003403

Attesa iniziale 10 s per riportare il gioco in primo piano, poi 25 s console e
25 s API. Foreground del PID misurato ogni 250 ms: **100% in entrambe le fasi**.
Valori seguenti sono mediane delle finestre; fasi successive, non simultanee.

| Misura | Console | API |
|---|---|---|
| FPS presentati | 172,485 | 173,437 |
| Intervallo finestre presentati | 162,073..183,841 | 171,160..194,802 |
| FPS visualizzati | non interrogati nel lettore CSV minimo | 173,461 |
| FPS applicazione | non interrogati nel lettore CSV minimo | 173,466 |
| CPU frame time (API) | — | 5,765 ms |
| Campioni | 4267 frame, copertura 24,957 s | 99 finestre valide |
| Primi dati validi API | — | 251 ms dalla richiesta |
| Picco working set raccoglitore | 16,02 MiB | 109,01 MiB |
| CPU raccoglitore durante la finestra attiva | 0,881% di un core | 1,780% di un core |
| CPU lettore Python | 0,377% di un core | 0,252% di un core |

Raccoglitore + lettore: circa 0,063% contro 0,102% della capacità totale sui
20 processori logici. Valori brevi, non prova di overhead sugli FPS del gioco.
Per il servizio sono state selezionate solo le osservazioni dentro la finestra
del client API, evitando di sommare inizializzazione e attesa del client.
working set include pagine condivise; crescita iniziale non è un test di leak.
Scostamento delle mediane fra le due fasi circa 0,55%, ma variazione della scena
e acquisizione successiva impediscono di attribuirlo ad accuratezza/overhead.

Stessa swap chain riconosciuta da entrambi. 99 valori distinti del CPU frame
time nelle 99 finestre API: dati aggiornati, non la replica del dato congelato.
Servizio chiuso graceful, codice 0; nessun raccoglitore rimasto o servizio
registrato. Resta l'avviso ETL opzionale/ACL della modalità console elevata,
già descritto nel verbale generale; nessun ACL modificato.

## Cosa dimostra e cosa manca

Entrambe le strade leggono il gioco e risultano sufficientemente reattive nel
caso provato. La console è più leggera nel processo raccoglitore; API espone
più metriche strutturate e funziona da client non elevato.

Con FG dichiarata attiva, application/displayed/presented dell'API coincidono
quasi perfettamente (rapporto mediano displayed/application ~1,00006). Questo
**non dimostra** la separazione fra frame nativi e generati in questo gioco.
Non etichettare automaticamente application_fps come FPS renderizzati nativi.
Le possibilità includono limiti dell'instrumentazione/driver o FG non effettiva
nel percorso osservato. Nessuna diagnosi definitiva da questi soli dati.

## FG spenta e confronto finale: 20261025-003945

Utente conferma FG spenta e scena pronta. Stesso PID, swap chain ricreata dal
gioco (0x28ED5873370). Ritardo iniziale 10 s, 25 s console e 25 s API con lettore
normale. Entrambi chiusi senza errori; servizio graceful, codice 0.

Foreground osservato 87% in entrambe le fasi. Console: gioco fuori primo piano
fra 5,52 e 7,28 s e dopo 23,60 s; API: fuori primo piano fino a 3,26 s.
Conservate tutte le statistiche grezze, ma il confronto seguente usa finestre
da un secondo interamente dentro intervalli osservati in primo piano. Escluso
anche un secondo dopo il ritorno e nessun collegamento fra campioni mancanti.
Lo stesso filtro è stato applicato alla prova FG attiva per coerenza.

| Mediane delle finestre qualificate | FG attiva | FG spenta |
|---|---|---|
| Console, FPS presentati | 172,352 (95 finestre) | 117,041 (77 finestre) |
| API, FPS presentati | 173,449 (96 finestre) | 116,049 (83 finestre) |
| API, FPS visualizzati | 173,445 | 116,252 |
| API, FPS applicazione | 173,460 | 116,056 |

Nel ritorno al gioco l'API grezza aveva un picco displayed di circa 525 FPS:
non incluso nel confronto perché transitorio fuori dalla finestra qualificata.
I filtri non dimostrano da soli la freschezza del dato; il prodotto necessita
ancora di tempo/età dell'ultimo frame sorgente, non del semplice successo poll.

Entrambi i lettori rilevano la variazione fra gli stati FG dichiarati. Le fasi
sono successive e la scena non è un benchmark automatizzato: nessuna stima
di FPS persi per il raccoglitore, errore percentuale o moltiplicatore esatto FG.
Non dividere gli FPS per due per inventare la frequenza nativa.

API application/displayed/presented coincidono in entrambi gli stati: manca una
separazione nativi/generati verificata in questa configurazione. Questo non
prova che PresentMon non possa distinguerli in altri giochi/driver o con altra
instrumentazione; la prova non ha stabilito quel beneficio del servizio API.

Picco working set del raccoglitore FG spenta: console 17,24 MiB, servizio
109,38 MiB. La maggiore leggerezza del processo console si conferma nei due
stati; questi numeri non misurano memoria privata/proporzionale. CPU e reader
del prototipo non rappresentano un'implementazione Rust ottimizzata.

## Raccomandazione e seguito

Per il contatore FPS principale di Bezel, procedere con la console PresentMon
della PR corretta, mantenendo RTSS come fallback. Prima dell'integrazione:
risolvere conflitti/test già noti, gestire PID/swap chain del gioco, tempi QPC
sorgente, freschezza/Alt-Tab, ciclo processo e console nascosta; etichette e
messaggi nelle tre lingue. Condividere cambiamenti e validazioni con l'autore
prima del merge, come concordato nella discussione PR.

Tenere servizio/API come evoluzione opzionale se emergeranno metriche verificate
o necessità di più client che compensino memoria/distribuzione aggiuntive.
Restano non provati: fallback RTSS reale, lungo periodo/idle/leak, altri giochi,
installazione/aggiornamento e tutte le combinazioni FG. Nessun merge o rilascio
eseguito in questa valutazione; prodotto 0.2.5 invariato.

Strumenti: 7 test Python, parser PowerShell e diff --check verdi. Nessuna suite
completa del prodotto ripetuta. Log ed evidenze conservati nei tre run indicati.
