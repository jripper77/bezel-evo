# Agenti per progetto e quote compatte - 16 novembre 2026

L'utente segnala quote Claude assenti e due finestre VS Code per progetti
separati. Chiede come opzione un riepilogo del solo stato: verde durante il
lavoro, rosso a lavoro finito o quando serve un intervento, con quote ridotte.

## Causa e correzione quote

Le sessioni reali sono claude-vscode, versione estensione 2.1.325. La status
line configurata in precedenza non viene eseguita nei pannelli dell'estensione.
Nessun JSON era stato esportato. La raccomandazione di riavviare Claude era
inadeguata per questa superficie.

Verificato il protocollo di controllo del client nativo gia autenticato,
initialize seguito da get_usage con skip_behaviors=true. E' lo stesso percorso
usato dall'estensione, marcato sperimentale nel suo codice. Nessun prompt o
messaggio viene inviato al modello. MCP, strumenti, impostazioni progetto e
persistenza sessione disabilitati per questo client di lettura. Nessuna
credenziale viene letta, esportata o gestita da Bezel. L'eseguibile nativo
resta responsabile del login. Dati reali restituiti nella prova: 16% consumato
nelle 5 ore e 83% settimanale, residui rispettivamente 84% e 17%.

Nuovo worker separato per le quote, un aggiornamento al minuto mentre un
widget Claude lo richiede, cache minima con le sole due finestre, TTL 180s.
Timeout, cancellazione e guardia del processo garantiscono la chiusura del
client di lettura prima di liberare il suo TempDir. Nessun processo agente
utente viene terminato. La configurazione statusLine precedente e stata
ripristinata, modificando soltanto il collegamento Bezel introdotto prima.
Client CLI autonomo o estensione nativa devono essere installati e autenticati.

## Progetti e visualizzazione

Aggiunto percorso completo del progetto ai dati della sessione. Filtro per
progetto nel widget, confronto Windows senza distinzione maiuscole e con
confini di directory. Ogni progetto segue la sua ultima sessione, anche quando
cambia ID. Sessione precisa ancora selezionabile. Il primo cwd Claude viene
conservato per non spostare il widget quando il lavoro entra in una sottocartella.

Opzioni statusOnly e compactQuotas con default compatibili con temi precedenti.
Indicatore verde Working, rosso Completed/Waiting/Error, grigio Unknown.
Richieste esplicite AskUserQuestion, ExitPlanMode e request_user_input marcate
come attesa utente. La lettura dei registri non garantisce la rilevazione di
ogni dialogo di permesso interno non registrato dal client.

Anteprima del tema con quattro pannelli: Codex e Claude per bezel, Codex e
Claude per EIDOLON. Fascia hardware superiore conservata. Quote compatte con
percentuale residua, reset sulla stessa riga e barra residua. Le quote sono
condivise per account, quindi uguali nei due progetti. Anteprima reale riuscita
con tutti e quattro i collegamenti e quote corrette per entrambi gli agenti:
target/logs/agent-projects/compact-projects.png e render-real.log.

## Verifiche e consegna

Test dei sei crate modificati superati. Ripetuti i test pertinenti dopo gli
ultimi adattamenti, 111 test sensori superati, 8 hardware/rete ignorati per
default. Prova reale esplicita del client Claude superata. Test pixel dei colori
verde/rosso e regressioni progetto/modalita compatta superati. Clippy sui sei
crate, tutti i target, warning negati, superato. 98 test UI e 8 browser verdi.
Build 0.2.31 completata, pacchetti setup/ZIP/SHA256SUMS verificati con 175 ingressi
nel portabile e binari corrispondenti alla build.

In attesa della conferma di salvataggio/chiusura Studio prima di copiare gli
eseguibili e aggiornare il tema personale. Il tema e pronto soltanto nell'area
di staging target/scratch/agent-projects/. Nessun push o pubblicazione.
Controllo desktop ritentato, compreso reset del runtime, ancora indisponibile
con errore native pipe not found. La verifica visiva usa il renderer nativo.


## Consegna completata

Dopo conferma di salvataggio/chiusura Studio, installata la 0.2.31 tramite
setup silenzioso e aggiornato dist/bezel in loco. Hash CLI/Studio installati
uguali alla build, CLI risponde 0.2.31. Evidenza installed-hashes-0.2.31.json.
Tema rigenerato dal file personale appena salvato per conservare gli ultimi
ritocchi hardware, poi sostituito atomicamente con backup del precedente.
Quattro pannelli con project esplicito, statusOnly e compactQuotas attivi.
Profilo VS Code verificato, selezione editor COM6 sul tema, Studio riavviato.
Prova finale sul tema installato riuscita: entrambi gli agenti selezionano
il progetto corretto, quote Claude 84% residuo cinque ore e 17% settimana.
Render finale in target/logs/agent-projects/installed-theme.png. I quattro
pannelli condividono correttamente le quote del rispettivo account. Nessuna
verifica visiva della finestra per indisponibilita della pipe desktop.
