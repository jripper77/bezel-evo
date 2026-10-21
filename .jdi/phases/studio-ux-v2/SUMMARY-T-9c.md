# T-9c — Dialogo Profili e galleria temi

21 ottobre 2026. Worktree `F:/dev/bezel-ux2-t9c`, branch `ux2/t9c`, base richiesta `34d950d`.
**T-9c completata.** Esportazione nativa confermata dall’utente il 21 ottobre; la rettifica della tavola è descritta sotto. Nessun merge, push, build del prodotto, installazione o cambio di versione; `dist/` non è stato toccato.

## Fatto

- Profili: dialogo con modalità Automatico/Manuale nella colonna sinistra, lista e miniature reali, profilo di base, intestazione del profilo e menu Rinomina/Duplica/Elimina. Tre sezioni indipendenti e persistenti: Schermi e temi, Si attiva quando, Comportamento.
- Salvataggio automatico dei soli documenti profilo validi: debounce, scritture serializzate, ultima modifica prevalente, errore ritentabile e conferma per scartare una bozza incompleta. Il documento Studio aperto rimane indipendente.
- Tutte le sette sorgenti di T-6 sono modificabili: app in primo piano, processo, media, sensore con soglia e attesa, orario/giorni, inattività e blocco. Frasi con controlli nativi; combinazione Qualsiasi/Tutte e priorità. Il processo resta nel selettore della sorgente; i sei pulsanti di aggiunta seguono D-12.
- “Attivo ora” deriva da `Overview.status.active`; le regole vere derivano da `Overview.status.rules[id][indice]`. Il confronto con la bozza normalizza i campi facoltativi omessi da Rust: una bozza cambiata non riceve un risultato vecchio, una regola identica non perde il risultato per l'ordine delle proprietà o per `forSeconds: 0`/`days: []` omessi.
- Nuovo comando nativo `try_themes`, operazione runtime `TryThemes`, bridge `tryThemes(themes, profile?)`: prova di 30 secondi sulle scene effettivamente in diretta e compatibili. Ripristino gestito dal servizio Profili; documento salvato e configurazione invariati. Una nuova prova sostituisce la precedente; budget degli asset rispettato. `Status.preview` è facoltativo e omesso quando assente. Resta disponibile anche il comando preesistente di prova della sola transizione.
- Galleria: la stessa griglia, ricerca e filtri della libreria vengono spostati nel dialogo; nessuna seconda implementazione della griglia. Tre colonne e dettaglio destro con anteprima, numero degli oggetti, sensori e letture formattate, utilizzi reali, apertura nell'editor, duplicazione ed eliminazione protetta. Schede con prova di 30 secondi e menu; nuovo tema vuoto o copia di un predefinito. Ricerca anche per sensori, filtri richiudibili e ordinamento dai più recenti.
- Metadati `ThemeEntryDto.sensors` e `objects` calcolati dal tema salvato, senza aprirlo nell'editor. Distintivi calcolati da catalogo e configurazione/stato Profili; prima che il catalogo arrivi non si dichiarano sensori mancanti. I feed gestiti da Meteo/Media non vengono scambiati per hardware mancante. La galleria richiede le letture dei sensori del tema selezionato e le rilascia alla chiusura.
- Cache delle richieste delle miniature condivisa fra libreria, galleria e Profili, per backend/percorso/revisione. Nativo: PNG dal renderer Rust già esistente. Browser demo: PNG da `demo-render`, lato massimo 360 px, con confronto pixel; il vecchio SVG resta soltanto come fallback dei test Node senza canvas.
- Nuovo comando nativo `delete_theme`: soltanto temi personali leggibili, percorso canonico strettamente nella libreria personale; rifiuta temi aperti, assegnati o in prova. I test backend usano ora `TempDir`, con guardia dopo backend e lettori.

## Esportazione — rettifica della tavola confermata dall'utente

Il 21 ottobre l'utente ha confermato: **esportare in `.bezeltheme`, formato nativo completo**. «`.turtheme`» nella tavola è un errore di disegno. Il pulsante Importa e l'importazione TURZX esistenti sono rimasti invariati; nessun writer TURZX è stato aggiunto. Rettifica annotata anche in D-13.

Il comando `export_theme`, esposto come `bridge.exportTheme(location)`, apre il selettore nativo e usa il `ThemeStore` esistente per copiare il tema **salvato selezionato** e tutte le risorse referenziate. Non apre il tema in Studio, non esporta per errore la bozza attualmente aperta e non modifica il tema sorgente. L'annullamento restituisce `null` e non scrive; risorse mancanti impediscono un'esportazione incompleta. Il demo mantiene l'esportazione nella propria memoria, come già fa per il salvataggio simulato; i test Rust verificano il vero pacchetto riaperto, risorse comprese.

## Commit di codice

- `621fc6a` — prova runtime di 30 secondi e ripristino.
- `64ba8f2` — comandi Studio, metadati, eliminazione protetta e cartelle temporanee dei test.
- `f2c9554` — salvataggio automatico e modelli puri di stato/ricerca/cache, con unitari.
- `cde5060` — dialoghi, primitivi condivisi, bridge/demo e test dell'area.
- `4946301` — dicitura del ritardo di ritorno e profilo attivo non più disponibile.
- `7d5faf6` — confronto delle regole con i valori predefiniti omessi dal nativo.
- `0736a92` — attesa del catalogo prima dei distintivi di sensori mancanti.
- `83e5e66` — Salva con nome del documento aperto fuori dalla galleria.
- `fff89d4` — esportazione nativa completa, cancellazione e test di indipendenza del documento.

Ogni commit ha scope `studio-ux-v2` e trailer ChatGPT. Il commit successivo registra ricette/evidenza e la conclusione della task.

## Verifiche

| Verifica | Esito / evidenza locale |
| --- | --- |
| `npm ci` | verde, zero vulnerabilità |
| `cargo test -p bezel-runtime` | **69 verdi**; prova di ripristino ripetuta da sola sull'ultimo codice, verde (`runtime-final.log`, `runtime-trial-final.log`) |
| `cargo test -p bezel-studio` | **232 verdi**, più 1 doctest verde; 1 test manuale ignorato (`export-rust.log`) |
| Clippy dei due crate, `--all-targets -- -D warnings` | **verde** (`runtime-clippy-fixed.log`, `export-clippy.log`) |
| `cargo fmt --all -- --check`; `git diff --check` | verde |
| Unitari dei moduli toccati, i18n e guardrail statici | **190 verdi** (`export-unit.log`); nessuna suite UI completa |
| E2E Profili e galleria, area completa finale | **60 verdi / 4 rossi**: unica asserzione preesistente descritta sotto (`delivery-e2e.log`) |
| E2E nuovi dialoghi/profili/prova, quattro progetti | **32 verdi** (`changed-final.log`), inclusa indipendenza tra selezione e attivo e scadenza dei 30 s |
| E2E miniature reali / dettaglio / confronto pixel, quattro progetti | **12 verdi** (`rendered-previews.log`) |
| E2E dettaglio galleria dopo la gestione del catalogo iniziale | **4 verdi** (`catalog-ready.log`) |
| Confronto it chiaro/scuro, ProfilesNew e ThemesNew | eseguito con `shoot.mjs`, HTML offline e dimensioni da `boards/index.json`; [confronto](evidence/T-9c.md) |

I log sono sotto `target/logs/t9c/`. Non è stata lanciata la suite completa né la copertura: competono a T-10.

Prima delle modifiche è stata eseguita la base `34d950d`: **18 verdi, 2 rossi**, stop a `--max-failures=2` (`base-e2e.log`). La spec Temi “an empty result…” cerca nell'inspector `inspector.canvas`; quella dicitura era già stata sostituita nelle attività precedenti. Lo stesso punto fallisce nei quattro progetti nel controllo finale. L'asserzione resta intatta; inspector, shell e relative traduzioni non sono stati modificati. Le altre spec di aree diverse non sono state modificate. Il caso System già contenuto in `profiles.spec.mjs` è conservato; la sua immagine diagnostica va ora nei log della task, non in `dist/`.

Il primo giro Studio aveva inoltre un confronto esatto fra float nel test di recupero dopo crash e la nuova autorizzazione del comando ancora da registrare. Autorizzazione corretta; il giro del crate dopo le modifiche necessarie è verde, senza indebolire il confronto del test esistente.

## Primitivi riusati

`accordionSection`, `createSectionMemory` con chiave/ids/aperture proprie, `chipButton`, `iconButton`, `iconChoice`, `numberField`, `screenThumb`, `sentence`, `openMenuAt` e `searchField` della libreria.

`filtersToggle` è stato estratto **una sola volta** da `ui/library.js` in `ui/filters-toggle.js`; libreria e galleria importano la stessa implementazione, riesportata anche da `library.js`. Nessuna variante locale. `openMenuAt` usa il menu comune: per funzionare nei dialoghi nativi il menu viene montato nel dialogo aperto dell'ancora, altrimenti nel body come prima.

## Differenze dalle tavole e note per T-10 / T-9b

- I nomi dei temi, le proporzioni delle miniature, i nomi degli schermi e le letture sono quelli del demo/bridge; i temi orizzontali d'esempio della tavola non sono inseriti come dati finti. Le miniature portrait sono contenute nel riquadro, senza tagli.
- Le tavole HTML hanno colori scuri fissi: i mock chiaro/scuro restano entrambi scuri. L'app è confrontata in entrambi gli schemi tramite i token. I dialoghi reali hanno altezza limitata alla finestra e scorrimento; la tavola Profili estesa è alta 1500 px.
- Dissolvenza non aggiunta: il modello deliberato delle transizioni di profilo usa coperture (`none`, `curtain`, `wipe`, `blinds`, `mosaic`). I valori di ritorno per regola già salvati restano intatti finché l'utente non cambia il controllo comune, che li imposta per tutte le regole.
- Eliminazione disabilitata per predefiniti e temi assegnati/in prova. Il backend protegge anche il documento aperto, compresi i casi senza display. Le assegnazioni vanno rimosse prima dell'eliminazione.
- Esportazione in `.bezeltheme`, per rettifica esplicita dell'utente: nessuna funzione rimane bloccata sul formato TURZX. Il vecchio Salva con nome dell'intestazione, che salva il documento aperto, resta nel pannello laterale ma è nascosto nella galleria per non confonderlo con l'esportazione del tema selezionato.
- **File di T-9b intatti:** `src/index.html`, `src/app.js`, `src/styles/shell.css`, `src/i18n/areas/shell.js`.
- Collegamento disponibile per T-9b: `library.openThemes()` seleziona il pannello Temi e apre la galleria; `library.closeThemes()` la chiude. `createProfilesDialog(...).open()` resta disponibile; sono esposti anche `refresh()` e il callback facoltativo `onStatus`. I pulsanti esistenti continuano a funzionare; il collegamento del nuovo toolbar è in `app.js`, di T-9b.
- T-10: integrare/verificare questi collegamenti, adattare la vecchia asserzione dell'inspector nella spec Temi, approvare la fedeltà visiva e provare i 30 secondi su hardware reale con Studio/Light aggiornati. Nessuna prova hardware è stata dichiarata eseguita.
- Il controllo automatico ha rifiutato la rimozione delle sole cartelle intermedie `target/logs/t9c/area-e2e` e `themes-e2e`, con motivo «blocked by policy». Non è stata ritentata: risultati e tracce restano nella directory della task. Le evidenze della base e del rosso preesistente sono conservate.
