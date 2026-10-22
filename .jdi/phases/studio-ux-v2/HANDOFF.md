# Passaggio — Studio UX v2

**Blocco Testo verificato:** `fid/inspector-text`, vedi `SUMMARY-FIDELITY-TEXT.md`.
Colore già integrato in `0710d65`; proseguire con geometria e composizioni.

**Ripresa fedeltà, 22 ottobre:** T-12 integrata in `593fc81`. Primo blocco Colore
completato su `fid/inspector-paint`; vedi `SUMMARY-FIDELITY-PAINT.md`. L’utente ha
autorizzato tutte le correzioni del registro prima delle proprie verifiche.
Prossimi blocchi: testo/misure, composizioni specifiche, shell e dialoghi.
Le note sotto su T-12 «pronta per integrazione» sono il passaggio precedente.

**T-12 completata, ChatGPT (22 ottobre):** branch `ux2/t12`, worktree
`F:/dev/bezel-ux2-t12`, base `fb6b42f`, pronta per l'integrazione. Codice
`0a94011`, `6efc7bf`, `d36310f`, `13361fc`, `ee50b37`; test e galleria in
`SUMMARY-T-12.md`. Nessun merge, push, build o installazione eseguito.
**Il riallineamento generale resta aperto:** seguire `PLAN-FIDELITY-FOLLOWUP.md`
e il registro `AUDIT-FIDELITY-2026-10-22.md`, senza chiedere all'utente di individuare
i singoli difetti. La fase aspetta ancora l'approvazione manuale.

**T-8 completata e integrata** (`c694a83`, vedi `SUMMARY-T-8.md`).
**T-9a completata e integrata** (vedi `SUMMARY-T-9a.md`).
**Assegnazioni (21 ottobre, decise dall'utente):** T-9c → **Codex/ChatGPT**
(worktree `F:/dev/bezel-ux2-t9c`, branch `ux2/t9c`): **T-9c completata e integrata**
(codice finale `fff89d4`, esportazione `.bezeltheme` confermata dall'utente, vedi `SUMMARY-T-9c.md`);
T-9b → **Claude**: completata e integrata (vedi `SUMMARY-T-9b.md`).
T-9d avviata da Codex e completata da Claude: integrata (vedi `SUMMARY-T-9d.md`). **Wave 2 chiusa.** T-11 (video negli oggetti Immagine) **rinviata a tempo indeterminato** dall'utente il 21 ottobre.
**T-10 completata e integrata** (Claude, vedi `REVIEW.md`): fase in APPROVED_PENDING_MANUAL, in attesa che l'utente approvi `evidence/final/index.html` e faccia le prove a vista.
T-9c non tocca `index.html` né `app.js` (di T-9b): espone funzioni e annota
il collegamento nel SUMMARY. Se T-9c richiede Rust (▶ prova di 30 s del
profilo), avvisare: Claude in T-9b non tocca crate Rust.

21 ottobre 2026. T-0…T-7c completate; T-7c in `ux2/t7c`, con riepilogo
in `SUMMARY-T-7c.md` e confronto visivo in `evidence/t7c/README.md`.

Attività corrente nell’ordine del piano: **T-8**, worktree già esistente
`F:/Dev/bezel-ux2-t8`, branch `ux2/t8`. T-0 e T-3, sue dipendenze, sono integrate.
T-11 resta separata per i video negli oggetti Immagine.

D-21 conferma che la Rotazione della tavola Card è un errore di disegno:
Card, gruppi e membri restano esclusi. Nella selezione multipla il campo compare
solo se tutti gli oggetti possono ruotare. Usare `canRotate` anche per le
maniglie T-8; `rotateSelection` è un solo Undo e omette il campo a zero.

D-20 aggiorna il contratto: voci delle tavole mancanti dal modello si includono
nell'attività corrente con compatibilità e test, chiedendo solo per lavori grandi
o contrasti con decisioni. Non riproporre le autorizzazioni già date.

Riutilizzare gli editor e le nuove opzioni di `INSPECTOR-COMPONENTS.md`.
D-20 aggiunge coverFade/iconColor e angoli delle immagini, con default omessi.
T-7c collega anche Card/Tema ai componenti comuni; la transizione Card è nel
Comportamento e l’Archivio si estende solo dal suo pulsante di Manutenzione.

Per T-10: riesaminare i controlli preesistenti conservati oltre le tavole, completare
la verifica visiva nativa dei file immagine e adattare le vecchie spec che si
aspettano slider/checkbox o schede sostituite. Nessuna spec di un’altra task è stata modificata in T-7a/T-7b/T-7c;
`card-inspector.spec.mjs:294` cerca ancora il tabpanel Trigger per la cattura,
dopo aver passato le verifiche funzionali. Vedere il SUMMARY T-7c. Suite completa e copertura solo a T-10.

Per T-9b/T-9c: usare `Overview.status.active` e `Overview.status.rules[id][indice]`,
non la riga selezionata nel dialogo. Vedere `SUMMARY-T-6.md` per dati mancanti.

Nessun push, build del prodotto o installazione. Versione e `dist/` invariati.
