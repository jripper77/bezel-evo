# Passaggio — Studio UX v2

21 ottobre 2026. T-0…T-6 integrate; T-7a completata in `ux2/t7a`, con riepilogo in
`SUMMARY-T-7a.md` e confronto visivo in `evidence/t7a/README.md`.

Prossima attività nell'ordine del piano: **T-7b**, nuova worktree dal nuovo HEAD di
`windows-improvements` dopo il merge T-7a. Tutte le dipendenze sono integrate.
T-11 resta separata per i video negli oggetti Immagine.

Per T-7b: usare gli editor condivisi `inspector/text.js` e `inspector/paint.js`,
le opzioni nuove di `fontPicker`/`numberField` descritte in `INSPECTOR-COMPONENTS.md`.
Il centro della Lancetta riusa già il testo con parti; non creare un altro editor.
Il guscio mantiene i percorsi precedenti per gli inspector ancora da migrare.

Per T-10: riesaminare i controlli preesistenti conservati oltre le tavole, completare
la verifica visiva nativa dei file immagine e adattare le vecchie spec che si
aspettano slider/checkbox o schede sostituite. Nessuna spec di un'altra task è
stata modificata in T-7a. Suite completa e copertura solo a T-10.

Per T-9b/T-9c: usare `Overview.status.active` e `Overview.status.rules[id][indice]`,
non la riga selezionata nel dialogo. Vedere `SUMMARY-T-6.md` per dati mancanti.

Nessun push, build del prodotto o installazione. Versione e `dist/` invariati.
