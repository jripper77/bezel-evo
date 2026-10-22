# Fedeltà — Card, Tema e selezione multipla

22 ottobre 2026. Branch `fid/inspector-containers`, base `ed2e3fe5`.
Implementazione: `5f4ce4c`.

## Interventi

- Card: lista verticale con vere miniature delle facce, nomi completi, numero, conteggio degli oggetti, «in vista», modifica e menu comune esistenti. «Aggiungi faccia» sotto la lista. Eliminata la doppia intestazione Facce; il conteggio è anche nella descrizione accessibile della lista.
- Le miniature usano `bridge.renderElement` già disponibile, su copie isolate del tema con la faccia richiesta: nessun cambio della faccia attiva, comando di store o Undo. Richieste sequenziali, una volta per il montaggio, ferme quando il controllo non è più collegato; nessuna animazione/polling aggiunti. Se il renderer non restituisce una miniatura, non si inventa contenuto.
- Oggetti della faccia come chip selezionabili; «Sposta oggetti» apre i controlli esistenti per scegliere un’altra faccia o lo sfondo comune. La disclosure conserva l’apertura durante la modifica e Undo. Editor del nome senza una seconda scheda imbottita; didascalie interne semplici.
- Transizioni Card con didascalie corte Nessuna/Dissolvi/Scorri/Ribalta, mantenendo i nomi completi per tooltip/accessibilità. Durata compatta, direzione a icone e toggle dello sfondo comune iniziale. Aggiunta trigger tramite chip Evento app/A tempo con gli stessi comandi e limiti precedenti.
- Tema: orientamento e giro di 180° nello stesso riquadro; stato salvato/non salvato come pill comune con punto, testo breve e nome accessibile completo. Dimensioni sempre leggibili; solo il nome lungo del modello si abbrevia, con metadati completi nel tooltip. Risolto l’allargamento del contenitore che tagliava la cattura.
- Multipla: geometria con prefissi nei campi, distribuzione orizzontale/verticale sulla stessa riga, raggruppa/crea Card compatti, titolo con tipografia normale dell’header. «misto», limiti, comandi dello store e vincoli di Rotazione rimangono invariati. Card, gruppi e membri non acquistano Rotazione; Ordine resta in Disposizione, Duplica/Elimina nei menu di T-12.
- Primitive riusate: `chipButton`, `detailsField`, `iconChoice`, `numberField`, `pill`, `previewChoice`, `propertySection`, `objectHeader`, `openMenuAt`. La regola tipografica identica del titolo del chooser è accorpata a quella dell’header in base.css. Nessuna seconda implementazione dei controlli.
- Nove chiavi nuove in en/pt-BR/it; conteggio oggetti riusa `layers.count`. Aggiornato docs/user/studio-editing.md. Nessun nuovo campo del tema, Rust, protocollo, build, installazione o push.

## Verifiche

- **72/72 unitari mirati verdi**: card-config, card-rules, inspector-edit, i18n e static-guards. Il nuovo test del tema isolato verifica faccia richiesta, invalidi, conservazione dei dati e assenza di mutazioni del documento originale. Log `target/logs/fidelity-containers/unit-final.log`.
- E2e dell’area: prima passata ampia **144/152 verdi**, con otto errori dovuti a selettori strutturali precedenti (conteggio nel titolo rimosso, pulsante Aggiungi nel vecchio header, scelta membro ambigua tra chip e disclosure, gruppo orientamento prima di due pulsanti). Migrati i selettori specifici del layout, senza indebolire le verifiche dei valori o dei comandi.
- Dopo le migrazioni **80/80 verdi**: card-inspector, background-chooser, inspector-containers nei quattro progetti chiaro/scuro en/pt-BR. Log `verified-area.log`.
- Dopo l’ultimo intervento sul testo/ingombro dei metadati **28/28 verdi**: inspector-card-screen e inspector-containers. Log `metadata-final.log`. Nel complesso tutti i **152 casi distinti** dell’area hanno esito verde nelle esecuzioni mirate, comprese cards, card-animation, canvas-card-handles e inspector-linked già verdi nella passata ampia; nessuna suite completa.
- Nuove asserzioni: miniature effettivamente disegnate e con pixel diversi tra le due facce, faccia attiva conservata; niente overflow a 320 px, dimensioni visibili, tre comandi orientamento e un Undo; distribuzione su una riga, nomi accessibili completi, raggruppamento reversibile. I controlli su nomi, spostamenti, ordinamento facce, trigger, limiti, animazioni, blocchi e Undo restano.
- Le prime prove del nuovo test assumevano erroneamente un orientamento iniziale non girato: il demo apre reverse-portrait. Ora la verifica legge lo stato iniziale e verifica inversione/ripristino; nessun comportamento del prodotto cambiato per adattarsi al test. Undo di un raggruppamento elimina il gruppo e filtra la selezione, come prima: il nuovo test verifica i tre oggetti ripristinati, poi li riseleziona.
- Otto catture italiano chiaro/scuro esaminate contro CardInspectorOpen, ThemeInspectorOpen e MultiInspectorOpen: `evidence/fidelity-containers/`. Ricetta Card popolata attraverso i normali comandi UI con oggetti distinti nelle due facce; niente disegni campione al posto del renderer. La passata visiva ha individuato e corretto i tagli del Tema e i ritorni a capo di distribuzione/transizioni.

## Differenze motivate e residui

- Rimangono lista verticale, sfondo comune separato, menu e comandi D-24; i segni decorativi di trascinamento della tavola non sostituiscono il riordino già disponibile nel menu. Il disegno degli Angoli rimane quello comune D-22, più ampio della sola icona della tavola.
- La disclosure «Sposta oggetti» mantiene una via concreta allo spostamento già supportato senza occupare tutta la sezione con gli editor aperti. È un accesso in più rispetto alla riga di chip del disegno, senza duplicare il menu oggetti.
- Dati di esempio, nomi delle facce e indicatori campione delle regole non sono copiati nel prodotto. La ricetta mostra le regole e il contesto realmente forniti dal demo. Nessuno stato runtime non esposto viene simulato.
- Il nome completo del modello resta nel tooltip quando non entra accanto a dimensioni e stato. Questo evita di sostituirlo con il nome breve fittizio della tavola.
- Restano shell/pannelli (MainNew, LibraryPanelsNew, StatusBarNew, Schermo/Archivio e fixture R00–R03), Profili/Galleria e chiusura visiva generale. `evidence/final/` è ancora T-12: rigenerare l’intera galleria soltanto dopo gli altri blocchi. Non chiedere ancora la verifica manuale dell’utente.
