# Integrazione FPS PresentMon — stato locale, 25 ottobre 2026

## Chiusura FPS 0.2.10 pronta, conferme pendenti

Utente conferma widget gioco continuo e vicino a Steam con Windrose FG attiva
su 0.2.9. Probe esplicito FrameType: 3081 righe tutte Application, mediana
foreground qualificata 211,98 FPS, focus solo 33/80 campioni: niente etichetta
DLSS affidabile da questa sorgente, niente prova lunga dichiarata. Nuovo bug
utente max circa4500 riprodotto: intervallo0,2095ms ->4773FPS istantanei.
35cd45c5 cambia min/max in medie complete1s, etichette tradotte; replay stesse
righe massimo212,92FPS, senza limite artificiale. Lows invariati. 9173c42
corregge propagazione percorso PresentMon; 925b81b probeFrameType opt-in.
98 test sensori/77 runtime,46 UI,16 e2e,8 Python e clippy/fmt mirati verdi.
Ultima build 0.2.10 riuscita2m24s, versioneCLI/metadati/sensori simulati e hash
verificati, f5b1482. Ancora non consegnata: installata0.2.9, attesa conferma
salvataggio/chiusura Studio/Light. Seconda domanda facoltativa pendente:
permesso Performance Log Users al solo account, avvio/collegamento canonici
con backup, oppure mantenere amministratore. Nessuna modifica account o
startup senza risposta esplicita. Dettagli .jdi/todos/2026-10-25-fps-final.md.
Bozza risposta autore e note release aggiornate; nessun invio/push/merge base.

## Consegna locale 0.2.9 completata

Utente conferma salvataggio e chiusura; verificata assenza processi, copiati
solo CLI/Studio in dist/bezel dopo controllo hash della build. Versione CLI
0.2.9, hash copie uguali ai sorgenti, record installed-hashes-0.2.9.json.
Studio elevato avviato da dist/bezel PID 17520. Necessaria conferma visiva
della diagnostica sul pannello con Live attivo: non dichiarare la prova reale
completata. Main sorgenti e ultima release pubblicata 0.2.0 invariati;
nessun push/commento autore/pubblicazione. Seguono stati precedenti.

## Diagnostica display: 0.2.9 pronta

Utente segnala preferenza 'Mostra le letture sullo schermo' senza effetto.
Causa: painter/preferenza ancora nel percorso legacy Studio, assenti nel
runtime condiviso Light. Fix 09805cf1 trasferisce il painter, sincronizza
opzione runtime e ripristino Light, usa completamenti I/O limitati a 20 e
tempo sampler; dipinge solo copia inviata al pannello, aggiorna temi statici
e forza il ripristino frame pulito quando disattivata. Nessun dato nel tema
o nel canvas. È diagnostica delle consegne Bezel, non FPS gioco.
Regressione su FakeConnector rossa senza painter e verde con fix; runtime
finale 77 test passati/1 ignorato, test CLI/Studio e clippy dei tre crate verdi;
4 e2e debug verdi. Guide en/pt-BR e changelog aggiornati. Build 0.2.9 riuscita
1m52s, CLI/metadati e sensori simulati verificati; versione 5861d77. In attesa
chiusura confermata Studio 38836/Light 10848 per copia in dist/bezel; installata
ancora 0.2.8. Prova hardware pending, nessun push o invio autore.
Dettagli .jdi/todos/2026-10-25-diagnostic-overlay-runtime.md, log overlay-*.

## Consegna locale 0.2.8 completata

Dopo conferma utente di chiusura Light/Studio, verificata assenza di processi
prima della copia. Aggiornati solo i due eseguibili in dist/bezel, preservando
dati e strumenti. CLI e metadati Studio 0.2.8; hash fonte/copia uguali, registrati
in target/logs/fps-integration/installed-hashes-0.2.8.json della worktree.
Studio avviato elevato dalla cartella canonica, PID 11936. Prova Windrose e
ritorno da Alt-Tab ancora da confermare: non dichiarare il difetto hardware
risolto in base al solo replay. Main sorgenti invariato; nessun push, invio
all'autore o pubblicazione. Il seguito conserva gli stati precedenti.

## 0.2.8 pronta: media e freschezza separate

Segnalazione utente: anche 0.2.7 intermittente. Dati reali dei probe precedenti
mostrano consegna console ritardata p95 1,26/1,33 s, oltre il secondo usato
dal vecchio filtro. Replay ai poll foreground dopo avvio: vecchio calcolo
25/88 e 21/75 indisponibili, nuovo 0 negli stessi punti. È replay dei probe,
non validazione hardware dell'eseguibile integrato.
Fix 5914d0bb: media nell'ultimo secondo sorgente, con scadenza separata a tre
secondi dall'ultimo frame; altri PID e timestamp futuri non rinfrescano il dato.
Regressione rossa prima e verde dopo, 97 test sensors passati, 5 hardware
ignorati, clippy/fmt verdi. Guide, risposta autore e note release aggiornate.
Build 0.2.8 riuscita 1m55s, CLI/metadati e sensori simulati verificati;
commit versione 6df1559. Chiesta chiusura Light 0.2.7 PID 12028 prima della
copia: conferma pendente. Nessuna consegna 0.2.8 ancora, né push/pubblicazione.
Dettagli: .jdi/todos/2026-10-25-fps-batch-arrival.md; evidenze batch-* e
build-hashes-0.2.8.json in target/logs/fps-integration della worktree.

## Consegna locale 0.2.7 completata

Dopo conferma utente di salvataggio/chiusura, retry build-evo.ps1 -Version
0.2.7 riuscito (1m18s), versione CLI e metadati Studio corretti. Prova sensori
simulati riuscita. Copiati solo i due eseguibili in dist/bezel, senza cancellare
dati; hash fonte/copia uguali, registrati in installed-hashes-0.2.7.json.
Release locale d29541b; avviato Studio elevato da dist/bezel PID 9252,
Light PID 12028. Nuova prova reale Alt-Tab/ritorno al gioco da confermare:
nessuna dichiarazione di risoluzione hardware basata sui soli test.
Nessun push, merge sul ramo principale, commento all'autore o pubblicazione.
Il seguito conserva il tentativo bloccato precedente per tracciabilità.

## Correzione ritorno da Alt-Tab: 0.2.7 in preparazione

L'utente conferma lettura intermittente passando a Studio/chat, sensore già
nel tema, e mancato ritorno tornando al gioco. Verificati entrambi i processi
0.2.6 elevati: non è una mancanza di privilegi in questa prova. Osservato
PresentMon figlio del runtime; questo non prova da solo letture corrette.

Commit 0aca0452: il ritorno a un candidato in primo piano supera solo l'attesa
per inattività e riarma il watchdog del raccoglitore mantenuto; errori di avvio
e permessi mantengono il backoff. Il vecchio FPS non viene riutilizzato.
Riprodotta regressione disattivando temporaneamente il nuovo aggancio:
test fallisce senza, passa con la correzione. Tutti i 96 test sensori passano,
5 hardware ignorati; clippy all-targets -D warnings e fmt verdi.
Aggiornate guide, changelog e bozza risposta all'autore; note draft 0.2.7
in dist del repository principale.

Build-evo.ps1 -Version 0.2.7 tentata ma bloccata da accesso negato al file
target/release/bezel.exe: utente ha riaperto Studio PID 536 e Light PID 50660
da target/release durante la compilazione. Chiesto salvataggio e chiusura
tramite conferma asincrona, senza terminare applicazioni. VERSION della
worktree già 0.2.7: ritentare con -Version 0.2.7, non incrementare ancora.
dist/bezel resta alla 0.2.6; nessuna consegna 0.2.7 e nessun push.
La prova reale del ritorno al gioco rimane necessaria: il test copre due
percorsi difettosi, senza dimostrare che esauriscano il caso hardware.

## Aggiornamento: consegna locale 0.2.6

Su richiesta dell'utente preparati risposta all'autore (bozza inglese in
docs/experiments/presentmon-pr2-review.md), credito esplicito README/changelog
e note cumulative in F:/Dev/bezel/dist/Bezel-Evo-0.2.6-release-notes.md.
Nessun messaggio inviato. Commit documentazione 60bf856; versione d5cdd78.
Il seguito descrive il precedente stato prima della consegna.

Build Windows 0.2.6 riuscita con build-evo.ps1 -Version 0.2.6; versione CLI e
metadati Studio verificati, prova CLI con sensori simulati riuscita. Studio e
Light risultavano chiusi prima della copia: dist/bezel aggiornata in posto,
senza cancellazioni. Hash verificati in target/logs/fps-integration/
installed-hashes-0.2.6.json. Copiata in tools la console Intel 2.6.0 già firmata
e verificata, esclusivamente per la prova locale; nessun servizio installato.
Studio avviato elevato, PID 40400, rispondente e titolo Bezel Evo 0.2.6;
runtime Light PID 32384. Permessi ETW non presenti nella sessione normale;
nessun gruppo o permesso permanente modificato. Attesa conferma utente
Windrose pronto e stato FG per la prova integrata. Non aggiunta dicitura DLSS:
nessun riconoscimento affidabile della tecnologia o separazione nativi/generati.
Ramo principale, GitHub e ultima release pubblicata 0.2.0 invariati.

## Ramo e ambito

fid/fps-presentmon in F:/Dev/bezel-fps-presentmon, base 08f42569. Merge locale
della PR #2: 0e8a2ff6, con commit e autore DarkK3y-v2 preservati. Correzioni
adattatore: 5796ba3. Ramo principale e installazione 0.2.5 invariati. Nessun
push, commento GitHub, merge della PR o eseguibile nuovo distribuito.

## Completato

- Conflitti catalogo/funzioni Windows risolti, fixture e test runtime corretti.
- PID candidato e una sola swap chain; tempi QPC originali, dati vecchi
  scartati, code limitate e storico dichiarato incompleto dopo overflow.
- Console nascosta, esclusività globale, job kill-on-close e reader separati
  fra catture. Il job è associato prima di completare l'avvio; non è garantita
  atomicità nel piccolo intervallo fra spawn e associazione.
- RTSS dopo errori e uscita del processo; retry limitati, backoff resettato
  solo dopo frame validi; nessuna cattura quando FPS non richiesti.
- Lows con almeno 100/1000 campioni; limite CPU/GPU/cap solo stimato e assente
  se ambiguo. Quantità testo e demo coerenti con i nuovi sensori.
- Etichette e disponibilità tradotte in en/pt-BR/it, dettagli originali
  preservati, probe PresentMon identificata. Support del README ripristinato;
  guide e changelog Unreleased con credito all'autore.

## Verifiche

Test di core, sensors, runtime, CLI e Studio: complessivamente 867 test Rust
e 3 doctest verificati. Sensori finali 93 passati, 5 hardware ignorati; test
aggiuntivo di esclusività globale ripassato dopo la modifica del namespace.
Clippy dei cinque crate con tutti i target e -D warnings verde; fmt e diff check.
Nessuna suite completa del workspace o copertura: nessun push richiesto.

55 unitari UI e 28 casi e2e distinti verdi (PresentMon e picker), inclusi casi
italiani. Cattura italiana scura controllata visivamente. Evidenze nel
target/logs/fps-integration di questa worktree. Cargo usa il target condiviso
F:/Dev/bezel/target; node_modules è una junction verso il repository principale.

## Limiti e seguito

Le prove Windrose precedenti confrontavano strumenti di acquisizione separati.
Non dichiarare completata una prova hardware del nuovo adattatore integrato,
un test lungo leak/idle, RTSS reale su questo PC o tutte le combinazioni FG.
Il test del ripiego usa una sorgente RTSS controllata; rimangono le fixture RTSS.
Il catalogo conserva il metadata hardware del contributo originario: descrivere
separatamente le verifiche di questo ramo, senza estenderle automaticamente.
Studio traduce i codici FPS; le letture generiche possono mantenere il prefisso
diagnostico. Prima del rilascio rivedere anche questa presentazione CLI.

Restano revisione finale e prova della nuova build Windows. Prima del merge
sul ramo principale condividere modifiche e risultati con l'autore, come
concordato nella PR; nessun messaggio inviato finora. Invio, push e pubblicazione
solo su istruzione dell'utente. Prima di un push eseguire una volta suite e
copertura di AGENTS.md. Per la consegna locale usare build-evo.ps1 con versione
nuova (almeno 0.2.6), preservare documenti e verificare gli hash delle copie.
