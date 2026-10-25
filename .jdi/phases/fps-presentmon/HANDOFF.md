# Integrazione FPS PresentMon — stato locale, 25 ottobre 2026

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
