# Passaggio — Studio UX v2

21 ottobre 2026. T-0…T-7a integrate; T-7b completata in `ux2/t7b`, con riepilogo in
`SUMMARY-T-7b.md` e confronto visivo in `evidence/t7b/README.md`.

Prossima attività nell'ordine del piano: **T-7c**, nuova worktree dal nuovo HEAD di
`windows-improvements` dopo il merge T-7b. Dipendenza T-0 già integrata.
T-11 resta separata per i video negli oggetti Immagine.

D-20 aggiorna il contratto: voci delle tavole mancanti dal modello si includono
nell'attività corrente con compatibilità e test, chiedendo solo per lavori grandi
o contrasti con decisioni. Non riproporre le autorizzazioni già date.

Riutilizzare gli editor e le nuove opzioni di `INSPECTOR-COMPONENTS.md`.
D-20 aggiunge coverFade/iconColor e angoli delle immagini, con default omessi.
Il guscio conserva il percorso precedente di Card/Tema per la migrazione T-7c.

Per T-10: riesaminare i controlli preesistenti conservati oltre le tavole, completare
la verifica visiva nativa dei file immagine e adattare le vecchie spec che si
aspettano slider/checkbox o schede sostituite. Nessuna spec di un'altra task è
stata modificata in T-7a/T-7b. Suite completa e copertura solo a T-10.

Per T-9b/T-9c: usare `Overview.status.active` e `Overview.status.rules[id][indice]`,
non la riga selezionata nel dialogo. Vedere `SUMMARY-T-6.md` per dati mancanti.

Nessun push, build del prodotto o installazione. Versione e `dist/` invariati.
