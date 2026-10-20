# Passaggio — Studio UX v2

21 ottobre 2026. T-0…T-5 già integrate; T-6 completata in `ux2/t6` recuperando il commit Rust di Claude `8c2c148`.
Completamenti ChatGPT: `ec0cfb7`, `63eb739`. Verifiche e contratto dei profili in `SUMMARY-T-6.md`.

Prossima attività nell'ordine del piano: **T-7a**, da una nuova worktree sul nuovo HEAD di `windows-improvements`
dopo il merge T-6. Tutte le dipendenze di T-7a sono integrate. T-11 resta separata per i video negli oggetti Immagine.

Per T-9b/T-9c: usare `Overview.status.active` e `Overview.status.rules[id][indice]`, non la riga selezionata
nel dialogo. `status`/`active` mancanti hanno un significato preciso: vedere il riepilogo T-6.
`profiles-model.js` valida e descrive tutte le nuove sorgenti; l'editor grafico delle nuove regole resta T-9c.

Nessun push o build del prodotto effettuato. Versione installata e `dist/` non modificati.
