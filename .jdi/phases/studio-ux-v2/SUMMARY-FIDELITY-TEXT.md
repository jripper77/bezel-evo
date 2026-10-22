# Riallineamento — Testo e parti

Commit di implementazione: `f457411`.

22 ottobre 2026. Branch `fid/inspector-text`, worktree `F:/dev/bezel-inspector-text`, base `0710d65`.

## Fatto

- Un solo `partsRow`: pulsanti con il contenuto reale del Valore e dell’Orologio, nomi accessibili delle parti conservati. Le parti seguono l’ordine del formato, compresi i due separatori; quelle assenti restano configurabili con «—». Stessa implementazione per il centro della Lancetta.
- Letture/formattazione riusano `partPieces` del canvas; aggiornamento con cache del campione/secondo e modifica del DOM soltanto se il testo cambia. Nessun valore esemplificativo nel prodotto.
- Editor della parte compatto: famiglia, colore e dimensione sulla stessa riga, peso e baseline affiancati; ereditarietà, reset e Undo mantenuti. Peso/dimensione del testo comune ora su una riga con coda da 96 px.
- Contenuto prima del sensore, Decimali/Unità affiancati. Ombra in una sola riga, campi disabilitati da spenta come nella tavola, senza intestazioni duplicate. Opacità dell’oggetto dopo gli altri colori. Rimossi titoli duplicati in parti dell’orologio, lampeggio e overflow.
- Nessuna modifica a temi, renderer o runtime. Restano tutte le modalità ereditate di overflow e transizione; la ricomposizione delle cinque scelte di movimento resta aperta nel blocco successivo.

## Primitivi

`partsRow`, `iconChoice`, `fontPicker`, `colorPicker`, `prefixedField`, `fieldGrid`, `propertySection`. Estensioni comuni: `fieldGrid(..., {tail:true})`, `fontPicker(..., {hideLabels:true})`, `propertySection(...,{heading:false})`. Nessuna variante locale del picker. Stili delle parti spostati dal foglio inspector a base.css; token della dimensione del campione in tokens.css. Chiavi corte dimensione/blur in tutte le lingue.

## Verifiche

- 63 unitari mirati verdi: i18n, static-guards, inspector-edit, text-layout.
- 92 casi e2e esistenti dell’area: clock, font-picker, inspector-linked, inspector-media, inspector-paint, text-case. Prima esecuzione: 87 verdi e 5 rossi (quattro asserzioni Ombra ancora «nascosto» invece di «disabilitato», un errore di traccia causato da due processi Playwright sullo stesso output). Aggiornata l’asserzione per il nuovo stato visibile/disabilitato, conservando pixel/Undo; il gruppo inspector-linked è poi verde nei quattro progetti (28 casi).
- 8 nuovi casi su contenuto effettivo delle parti, ereditarietà/reset/Undo, ordine dei separatori e aggiornamento dell’ora senza modificare il pattern. La nuova fixture orologio richiedeva il ruolo combobox del datalist e l’apertura di Forma: corretti, senza attenuare le asserzioni.
- Ultimo gruppo di conferma: **36/36 verdi** (nuova spec e inspector-media), dopo la correzione delle intestazioni e dei separatori. Insieme alle esecuzioni precedenti tutti i 100 casi distinti hanno esito verde; non è una nuova esecuzione completa del prodotto.
- Confronto italiano chiaro/scuro in `evidence/fidelity-text/` per Valore, Orologio, Lancetta. Esaminata la composizione rispetto alle tavole; dati/font effettivi dei documenti non sostituiti con quelli dimostrativi.
- `git diff --check` verde. Nessuna build, installazione, push o suite Rust.

## Seguito

Proseguire con misure, angoli e composizione di Forma/Comportamento; poi media, Card, multipla, shell e dialoghi. Le presenti catture documentano questo blocco e non attestano la conformità dell’interfaccia intera.
