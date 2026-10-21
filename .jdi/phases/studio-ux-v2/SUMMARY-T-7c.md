# T-7c — Card, Tema e pannelli Schermo

21 ottobre 2026. Completata in `ux2/t7c`, worktree `F:/Dev/bezel-ux2-t7c`,
dal commit integrato `e0b178f` di `windows-improvements`.

## Fatto

- Card: Disposizione, Facce, Forma, Colore e Comportamento. Un solo editor
  della transizione, nel Comportamento, con quattro effetti reali, durata,
  direzione e sfondo comune. Regole app/tempo, priorità, ritorno e riordino
  rimangono operativi. Nomi delle facce interi anche alla larghezza minima
  dell’inspector (280 px), sfondo comune e modifica con pulsante a matita.
  L’intestazione riporta faccia corrente, numero di facce e nome reale.
- Tema: anteprima del canvas già renderizzato, nome modificabile, schermo e
  dimensioni reali, stato di salvataggio, menu Salva/Salva come/Duplica.
  La copia propone un nuovo nome attraverso il salvataggio esistente; nome e
  stato cambiano solo dopo il successo. «Salva come» esporta il `.bezeltheme`
  con i suoi asset. Orientamento e rotazione di 180° usano `setOrientation`.
  Sfondo e Aggiornamento hanno sezioni e riassunti propri, con memoria condivisa.
- Schermo: intestazione del dispositivo comune alle due sottoschede. Menu ⋯
  con rilascio della porta e riavvio, entrambi con conferma. Luminosità con
  slider e campo numerico sincronizzati; Avvio con il computer usa il pulsante
  a icona. Standby a quattro scelte, riepilogo della scelta, motivi delle
  opzioni disabilitate e timer di riserva; nessuna scrittura senza conferma.
- Archivio: Spazio, All’accensione, File e Manutenzione. Filtri raccolti,
  invio con scelta del supporto e conferma esistente, menu del singolo file,
  ricerca, copie locali e pulizia. Le listbox conservano la tastiera e il
  trascinamento; i menu sono pulsanti nativi esterni alle opzioni. Il medesimo
  gestore si espande su richiesta, senza sostituire il canvas all’apertura.
- Chiarimento D-21: Card, gruppi e membri restano senza Rotazione. Per una
  selezione multipla il campo compare solo se tutti gli oggetti sono ammessi.
  Il nuovo comando JS `rotateSelection` applica la rotazione individuale in
  un solo Undo, con normalizzazione, rispetto dei blocchi e omissione dello zero.

## Compatibilità, riuso e prestazioni

Nessun campo del file tema aggiunto, nessun DTO o renderer Rust modificato.
Non si scrivono valori del tema aprendo le sezioni. La rotazione multipla usa
il campo già esistente e il comando di rotazione singola: i test controllano
il documento originale, Undo, oggetti esclusi e rimozione del campo a zero.

Riusati `accordionSection`, memoria delle sezioni dell’inspector, `objectHeader`,
`iconChoice`, `previewChoice`, `numberField`, `rangeField`, `toggleField`,
`iconButton`, `colorPicker`, sfondo/chooser, editor Forma/Angoli/Trasparenza,
menu e conferme comuni, gestore dei file e relativo motore di selezione.
**Nessuna duplicazione** dei componenti, del renderer o delle operazioni sui file.

L’anteprima Tema copia il canvas già prodotto: non chiede un secondo rendering.
Il riferimento si aggiorna al rifacimento dell’inspector; nessuna copia quando
il pannello è nascosto, nessun ridimensionamento del canvas se le misure non
cambiano. Le nuove impostazioni non aggiungono campionamento o timer di rendering.

## Commit

- `0ab8030` — rotazione delle selezioni ammesse, test puri e decisione D-21.
- `5fb6183` — inspector Card/Tema, Schermo/Archivio, i18n e nuovi e2e.
- `9d899de` — ricette e confronto visivo in chiaro/scuro.

Tutti con scope `studio-ux-v2` e trailer ChatGPT. Riepilogo, stato del piano e
passaggio sono consegnati nel successivo commit documentale.

## Verifica

| Controllo | Esito |
| --- | --- |
| Unitari UI mirati: editor-store, cards, rotation, inspector-edit/model/sections/tabs, card-config/rules, storage/manager/breakdown, standby, i18n e guardrail statici | 204 verdi |
| Unitari demo-backend, image-playback e media-inspector (helper del demo-render) | 48 verdi |
| Nuova `inspector-card-screen.spec.mjs`, en/pt-BR × chiaro/scuro | 20 verdi |
| Ricontrollo finale dopo ottimizzazione miniatura e CSS: Card e Tema della nuova spec | 12 verdi, già compresi nei 20 distinti |
| Regressioni mirate esistenti: flip renderizzato, trigger disabilitati con una faccia, suggerimenti reali app e conferma dello standby corrente | 16 verdi |
| Guardrail i18n/statici ripetuti sul codice finale | 51 verdi, già compresi negli unitari |
| `git diff --check` | verde |
| G3: tre HTML aperte in Chromium; ricette inspector/screen in italiano, chiaro/scuro | 10 catture di stato e 10 dettagli, controllati |

Totale distinto: **252 unitari e 36 e2e verdi**. Log locali raggruppati in
`target/logs/t7c/`; confronto durevole in `evidence/t7c/README.md` e manifest.
Non eseguiti Cargo/test/clippy/fmt Rust perché nessun file Rust è stato toccato.
Nessuna suite completa, copertura, build prodotto, versione, installazione o push.

Una vecchia spec è stata eseguita e resta da adattare a T-10, secondo G4:
`card-inspector.spec.mjs:241`, «triggers read as sentences…», passa tutte le
asserzioni su token, priorità, riordino, Undo e ritorno, poi fallisce nelle
quattro configurazioni alla riga 294: la cattura cerca il vecchio
`role=tabpanel` Trigger. Non è stata modificata né indebolita. Non è inclusa
nei 36 e2e verdi. Un errore iniziale del nuovo test sulla larghezza richiesta
(272, sotto il minimo reale 280) è stato corretto; il test ora verifica anche
la larghezza effettiva, oltre all’assenza di taglio del nome.

## Differenze dalle tavole e T-10

- Rotazione nella tavola Card: **errore di disegno confermato dall’utente**,
  D-2026-10-21-studio-ux-v2-21; omissione intenzionale. La stessa esclusione
  vale nelle selezioni miste. Non si estende il modello di rotazione.
- Dati reali del demo: facce nuove vuote, tema Demo, porta e scheda SD dello
  scenario. Non si copiano le metriche o i file di esempio delle tavole.
- Restano funzioni esistenti ulteriori: movimento dello sfondo comune,
  comandi dei membri, priorità/ritorno, dettagli dei file e aiuti. Le etichette
  tradotte lunghe vanno a capo. Si riusa il medesimo gestore per supporto anche
  nel pannello stretto; la vista estesa conserva gli strumenti avanzati.
- T-10: adattare le vecchie spec che cercano le schede Facce/Movimento/Trigger,
  il vecchio editor compatto della transizione, radio degli effetti, testo
  Modifica e checkbox di avvio. Per Archivio, i filtri ora sono nel controllo
  dedicato e la vista estesa si apre dalla Manutenzione. Nessuna asserzione
  preesistente è stata indebolita.
- T-10: verifica finale nativa/hardware di rilascio, riavvio, standby e archivio;
  suite completa e copertura una sola volta. Nessun nuovo protocollo o permesso.

Prossima attività: **T-8**, in nuova worktree dal nuovo HEAD integrato.

Nota operativa dopo il merge iniziale `cae092f`: il controllo automatico ha rifiutato sia la pulizia ricorsiva sia la successiva operazione limitata di pulizia/spostamento, con il solo motivo «blocked by policy». Gli intermedi PNG in `target/scratch/t7c/boards/` e gli artefatti Playwright in `apps/bezel-studio/test-results/` restano nella worktree; non sono stati eliminati né spostati. I log testuali e le prove finali restano nei percorsi documentati sopra. Nessun effetto sul codice integrato, su `dist/` o sui dati utente.
