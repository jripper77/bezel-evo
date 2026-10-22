# Riallineamento — Colore condiviso

Commit di implementazione: `933a1cf`.

22 ottobre 2026. Branch `fid/inspector-paint`, worktree `F:/dev/bezel-inspector-paint`, base `593fc81` (integrazione T-12).

## Fatto

- Riempimento e fulmine sulla stessa riga, campioni agli estremi e barra della sfumatura; punto medio dell’arco modificabile da tastiera o trascinamento, un solo Undo. Conservato anche l’input numerico preciso, in una riga compatta sotto la barra.
- Picker degli estremi riusato con campione compatto e nome/descrizione accessibili. Gradienti lineari importati conservano angolo e tutte le tappe intermedie modificando gli estremi.
- Traccia con pulsante visibilità e colore affiancati, senza il titolo duplicato «Mostra traccia». Opacità dopo la traccia, senza il contenitore aggiuntivo «Visibilità» quando esiste Colore.
- Eliminato il titolo interno Colore già presente nell’accordion; stessa composizione per tutti i chiamanti di `gaugeFillFields` (Anello, Barra, Forma/Card e Spettro).
- Ricetta Anello aperto ora con sfumatura, estremi distinti e nessun focus artificiale, per confrontare lo stesso stato della tavola.

## Primitivi e limiti

`iconChoice`, `bindButton`, `colorPicker`, `numberField`, `propertySection` restano le implementazioni uniche. `fieldAction` compone un campo con la sua azione; `gradientField` compone i picker esistenti e un range nativo. Stili in base.css, nessuno stile locale duplicato. Nessun campo del tema o renderer modificato.

Il fulmine dell’opacità delle tavole non ha un modello/runtime corrispondente: non viene mostrato un comando privo di funzione. Questo eventuale ampliamento rimane da censire separatamente (non necessario per la ricomposizione dei controlli). L’input numerico del punto medio resta per conservare l’editing esatto già esistente. Colori/valori degli esempi appartengono soltanto alla fixture.

## Verifiche

- 62 unitari mirati verdi (inspector-edit, color-math, i18n e guardie statiche).
- 44 e2e esistenti verdi: ring-gradient, value-color, inspector-media.
- 44 e2e dei chiamanti verdi: inspector-linked e inspector-components (card.spec non corrisponde a una spec presente; la verifica Card resta nella copertura delle altre spec e nelle prossime correzioni dedicate).
- 8 nuovi e2e verdi: posizione del fulmine, midpoint, Undo singolo, traccia, tappe importate/angolo. Quattro progetti chiaro/scuro, en/pt-BR.
- Una prima esecuzione dei nuovi casi aveva un errore nella fixture (import di colorTexts non pubblico), corretto usando createPickers; nessuna asserzione indebolita. Guardia duplicati CSS inizialmente rossa: eliminata la dichiarazione duplicata.
- Confronto visivo italiano chiaro/scuro esaminato: `evidence/fidelity-paint/`. Non dichiara chiusi Forma, parti del testo o l’audit generale. Nessuna suite completa Rust/prodotto, build, installazione o push.

## Seguito

Proseguire con parti del testo, misure compatte e ricomposizione degli inspector, poi shell e dialoghi. Rigenerare la galleria complessiva dopo questi interventi; la presente evidenza è del solo blocco Colore.
