# Riallineamento — Geometria condivisa, Anello e Barra

22 ottobre 2026. Branch `fid/inspector-geometry`, base `49a2a3f`.

- Stile precede Arco in Forma, con ordinamento condiviso nella composizione delle sezioni. Visualizzazione usa la didascalia Stile.
- Anello: Arco compatto, spessore e verso affiancati, Estremi a icone accanto ai blocchi. Blocco attivabile con campi numero/spazio sulla stessa riga; campi disabilitati quando spento. Barra riusa esattamente lo stesso controllo dei blocchi.
- Campi Sensore/Scala con didascalie esplicite; nessun titolo interno duplicato per blocchi e prova 100%.
- Angoli: i due modi hanno icone e nomi accessibili; il modo uguale è affiancato al raggio. Conservato il disegno proporzionato e tutti i singoli angoli secondo D-2026-10-15-studio-redesign-fidelity-22. Questa è una differenza motivata dal riferimento più schematico, non una seconda implementazione.
- L’anteprima geometrica dell’arco esistente resta piccola nella didascalia, anziché occupare 64 px accanto ai numeri; preservati i riscontri sulla geometria e Undo.
- Scelte delle cinque transizioni del Valore su griglia a tre colonne, senza una riga con quattro etichette e una quinta isolata. Nessuna modalità eliminata.
- Corretta la regola comune degli input: il bordo del campo si applica ai figli diretti, non anche ai numeri già racchiusi nei campi prefissati. Nessuna modifica al tema o ai renderer.

## Primitivi e verifica

`iconChoice` (colonne facoltative), `fieldGrid` (colonna iniziale compatta), `numberField`, `prefixedField`, `cornersField`, `propertySection`, icone nel catalogo comune. Stili solo nei fogli comuni/inspector con token.

- **80 unitari mirati verdi**: i18n, static-guards, corner-radii, inspector-model, inspector-helpers.
- **92 e2e esistenti verdi**: inspector-linked, corner-radii, inspector-preview, inspector-components, audio-spectrum, quattro progetti chiaro/scuro en/pt-BR.
- Dopo la correzione finale degli input e delle didascalie: **52/52 verdi**, comprendono i quattro nuovi casi di inspector-geometry e i chiamanti inspector-linked/corner-radii/inspector-components. Confermano raggruppamento visivo, icone, limiti/abilitazione, un Undo per modifica e assenza di bordo annidato.
- Confronto italiano chiaro/scuro in `evidence/fidelity-geometry/` per Anello e Forma. Esaminati struttura, angoli e controlli compatti; Forma/media/bordo/trasparenza restano da ricomporre nel blocco dedicato.
- Nessuna asserzione indebolita, nessuna build/installazione/push o suite completa Rust. `git diff --check` verde.

Prossimo: Grafico/Spettro e media, poi Card/multipla, shell e dialoghi; galleria finale soltanto a correzioni terminate.
