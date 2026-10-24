# Integrazione FPS PresentMon — stato locale, 25 ottobre 2026

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
