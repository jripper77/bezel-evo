# T-12 — punto di ripresa

22 ottobre 2026. Pausa dell'implementazione richiesta dall'utente per valutare un riallineamento sistematico ai mockup. Confronto autonomo successivamente autorizzato e concluso: vedere `AUDIT-FIDELITY-2026-10-22.md` e `PLAN-FIDELITY-FOLLOWUP.md`. L'utente ha chiesto di salvare e spegnere normalmente il PC al termine, senza attenderne risposte.

## Stato

- Worktree `F:/dev/bezel-ux2-t12`, branch `ux2/t12`, base di prodotto `fb6b42f`. Eventuali commit successivi di questo checkpoint riguardano soltanto documenti/evidenze dell'audit.
- Modifiche non committate; nessun merge, push, build o installazione. T-12 non completata.
- Implementati in bozza: comando atomico di passo singolo multiplo e ordine per ambito; riga Ordine nella Disposizione; menu condiviso esteso; rimozione Duplica/Elimina dal pannello; riallineamento parziale di header, Tipo, Allinea, Direzione, Colore e Comportamento.
- Colore non ancora fedele in tutte le modalità: gradiente e posizione del collegamento al sensore da verificare. Nessuna approvazione visiva implicita.

## Verifiche già eseguite

- Unitari mirati ripetuti sul checkpoint attuale: **89/89 passati** (layer-order, editor-store, layers, inspector-helpers, i18n e static-guards), log `target/logs/t12/unit-checkpoint.log`. Non certificano la fedeltà grafica né completano tutte le verifiche di T-12.
- E2e inspector-components, inspector-linked e object-rotation nei quattro progetti: **60 passati, 4 falliti**. Log `target/logs/t12/e2e-first.log`.
- I quattro fallimenti riguardano `copy, paste and duplicate keep the angle`: la migrazione della spec apre il menu due volte, prima direttamente e poi tramite `openInspectorMenu`. Rimuovere l'apertura ridondante e rieseguire la spec mantenendo tutte le asserzioni. Non classificato come difetto preesistente.
- Prima cattura: 8 screenshot, chiaro/scuro, Ring/Card/Bar/Multi, in `target/scratch/t12/visual-first`; log `target/logs/t12/visual-first.log`. Catture precedenti alle ultime correzioni, non evidenza finale.
- Audit: galleria separata in `target/scratch/t12/audit-gallery/index.html`, 96 catture di stato, più 4 per logo/nessuno schermo in `audit-extra/`; confronto dei frame nativi T-9d in `target/scratch/t12/templates-audit.png`. Nessuna approvazione implicita.
- Estratte e confrontate staticamente tutte le 30 tavole dell'HTML completo dell'utente: CSS di design identico alle tavole offline, escluse le dichiarazioni dei font. Inventario in `evidence/audit-2026-10-22/reference-inventory.json`.
- I processi di queste verifiche sono terminati; nessun server di prova deve essere lasciato in esecuzione per lo spegnimento.

## Da completare

Rivedere il codice in bozza e i casi Card/gruppi/membri; rimuovere eventuali helper di allineamento rimasti inutilizzati. Completare la migrazione delle spec richieste (inspector, card, layer, clipboard, scenari) e i test pertinenti. Confrontare sistematicamente i riferimenti, inclusa la pagina HTML completa dell'utente, secondo il piano proposto.

Restano da fare documentazione utente, SUMMARY-T-12, aggiornamenti PLAN/HANDOFF, galleria finale e REVIEW, commit piccoli con trailer ChatGPT. Non segnare completata la task né la fase prima delle verifiche.

## Copia di ripresa

Oltre ai file presenti nella worktree, `target/scratch/t12/checkpoint/` contiene patch binaria Git dei file tracciati, copia dei due nuovi moduli non tracciati e inventario del salvataggio. Non applicare la patch sopra la worktree già modificata: serve solo come recupero. Conservare questi output fino alla chiusura di T-12.
