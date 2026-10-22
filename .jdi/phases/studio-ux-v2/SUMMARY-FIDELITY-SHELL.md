# Fedelt? ? shell e pannelli

22 ottobre 2026. Branch `fid/studio-shell-panels`, base `0049a0b4`.
Codice `12f7879`.

## Correzioni

- Inspector iniziale 360 px, preferenze salvate conservate; test esplicito del precedente valore 320.
- Rail nell?ordine della tavola, senza titoletti aggiuntivi. Orientamento del canvas a icone, con nomi accessibili invariati.
- Tessere Widget 68 px, gap 6, icona numerica Valore e didascalie corte Player/Spettro; gruppi senza bordi interni aggiuntivi, freccia a sinistra.
- Livelli con solo visibilit?/blocco; riordino da trascinamento, Ordine e menu condiviso. Test Card migrato allo stesso comando avanti di uno, mantenendo le verifiche dell?ambito e Undo.
- Chip operativi Sensori/Bezel neutri, pallini di stato; pallino aggiunto alla testata Prestazioni.
- Schermo con riquadro miniatura 72 px, dati senza porta ripetuta, accensione a sinistra, Luminosit? col campo numerico comune. Stili numerici trasferiti al primitivo comune, frecce native interne rimosse per evitare il taglio delle cifre.
- Standby con didascalie corte e nomi accessibili completi; spiegazioni disponibili ai lettori di schermo, motivi di indisponibilit? visibili.
- Ricette: Libreria 400 px senza rail, Schermo 380 px; Livelli con Card, membri e gruppo; Media con sfondo realmente in uso.

## Verifica

- Unitari `panel-sizes`, `memory-status`, `library`, `standby`, `i18n`, `static-guards`: **104/104**.
- E2E `panels` + `inspector-card-screen`: **32/32**; `shell-status` da sola **12/12**; `cards` + `standby` **28/28**; `shell` + `library` **64/64**. `memory-status`: **12/12** nel primo giro. In totale 148 casi distinti dell?area passati in esecuzioni mirate, non una suite unica.
- Primo giro 115/120: due attese numeriche rimaste al vecchio default, corrette; tre problemi sotto carico di shell-status passati nella riesecuzione isolata. Il test Card usava inizialmente la vecchia chiave della freccia, corretto alla chiave di Ordine. Nessuna asserzione funzionale rimossa.
- Catture italiane chiare/scure: `evidence/fidelity-shell/`, con dettaglio dei pannelli. Log in `target/logs/fidelity-shell/` della worktree.

## Differenze motivate

- Salva resta secondario per D-2026-10-09-studio-redesign-fidelity-6, non arancione come il disegno.
- Conteggi e filtri delle sonde, stato lettura e diagnostica conservati: sono comandi funzionanti gi? introdotti, oltre la semplice lista della tavola.
- Standby conserva il flusso di conferma e configurazione del timer gi? deciso, senza copiare il valore 5 minuti della tavola come nuovo default.
- Archivio conserva scheda SD presente, refresh/stop e spiegazioni del filesystem quando reali; non si simula una scheda assente per copiare il disegno. Media non inventa poster per importazioni demo prive di immagine.
- Nomi dispositivi, porte, oggetti, conteggi e metriche provengono dal bridge; non sostituiti con quelli del mockup.

Primitivi riusati: accordionSection, numberField, toggleField, pill, panelHead, iconButton; menu oggetto e comandi dello store esistenti. Nessuna modifica Rust, build, installazione o push.

Proseguire con Profili/Galleria e poi confronto finale di tutte le tavole. La galleria `evidence/final/` verr? rigenerata alla fine, prima della verifica visiva dell?utente.
