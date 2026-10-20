# T-0 — Fondazione dell'inspector: avanzamento parziale

Data: 20 ottobre 2026. Branch: `ux2/t0`, worktree `F:\dev\bezel-ux2-t0`.
Partenza: `04556ff` di `windows-improvements` (log controllato prima di iniziare).

**T-0 non è completata e non è stata unita a `windows-improvements`.**
Questa nota registra due passi autonomi completati e verificati, prima del chiarimento richiesto all'utente.

## Fatto

- `1697e93` — `feat(studio-ux-v2): add persistent inspector section memory`:
  nuovo modulo puro `src/inspector-sections.js`, ordine stabile, memoria indipendente per tipo,
  persistenza in `localStorage` protetta con try/catch, Alt+clic esclusivo, riassunti con numeri localizzati e unità.
  Stato iniziale: Dati e, per Card, Facce aperte; le altre sezioni chiuse. La memoria resta separata dal tema e da Undo.
  Preferenze corrotte, vecchie o storage negato non impediscono l'editing.
- `bbae69d` — `feat(studio-ux-v2): add accessible inspector accordion primitive`:
  `accordionSection` in `src/ui/fields.js`, unico stile in `base.css`, catalogo dei primitivi aggiornato nel commento iniziale.
  Riga di 48 px, riassunto visibile solo da chiusa, icona accentata da aperta, pulsante nativo con
  `aria-expanded`/`aria-controls`, regione con titolo accessibile, Invio/Spazio, frecce/Home/End sui titoli.
  Chiudere mantiene montati i controlli e conserva i valori immessi.

API del primitivo: `accordionSection({ id, title, icon, summary, expanded, onToggle, render }, ctx)`
restituisce `{ element, setExpanded }`. Il chiamante applica `createSectionMemory.toggle` in `onToggle`
e aggiorna le sezioni con `setExpanded`. `render(body, ctx)` monta il contenuto una volta.

Primitivi riusati: `el`, `icon`, `numberField`, icone esistenti, token esistenti.
**Nessuna duplicazione:** memoria in un solo modulo puro; markup accordion in un solo builder;
stile accordion nel solo catalogo condiviso `base.css`.

## Test eseguiti

- `node --test --experimental-test-coverage --test-coverage-include=src/inspector-sections.js --test-coverage-lines=100 --test-coverage-branches=100 --test-coverage-functions=100 tests/ui/inspector-sections.test.mjs`:
  **9/9 verdi, righe/rami/funzioni 100%**.
- `node --test tests/ui/static-guards.test.mjs tests/ui/inspector-sections.test.mjs tests/ui/i18n.test.mjs`:
  **60/60 verdi**; token, assenza duplicazioni CSS/SVG, CSP e parità i18n inclusi.
- `node scripts/e2e-slot.mjs tests/e2e/inspector-primitives.spec.mjs --output=../../target/logs/studio-ux-v2-t0/primitives`:
  **8/8 verdi**, chiaro/scuro × en/pt-BR. Verificati memoria controllata, Alt+clic,
  conservazione input, altezza, assenza di overflow, tastiera e `expectAccessible`.
- `git diff --check`: verde.
- Nessun Rust o demo-render modificato: test Rust/clippy e test demo-render non necessari per questi due passi.
- Nessuna suite completa, build prodotto, modifica di VERSION, setup, push o pubblicazione.

## Chiarimento necessario

L'accettazione T-0 richiede ▶ Prova con escursione del valore e ritorno, equivalente per orologio,
meteo e player, e cita `preview-animation.js` come funzione esistente da riusare.
Il codice verificato offre invece soltanto pianificazione dei fotogrammi da `nextMs`:

- `src/preview-animation.js`: timer, stop, movimento ridotto; nessuna sequenza di valori di prova.
- `src/bridge.js`, `renderElement`: opzioni solo `max` e `motion`.
- `src/demo-backend.js`, `renderElement`: le stesse opzioni.
- `src/app.js`: inoltra al bridge il tema corrente; nessun contesto di prova.

Le modifiche bridge/Rust necessarie non sono tra i file di T-0 e l'attività prescrive uno
spostamento del codice esistente senza funzioni nuove. È stato chiesto all'utente se includere
questo supporto in T-0 oppure rinviare ▶ Prova a T-10. Nessun comportamento di prova inventato
e nessun pulsante fittizio aggiunto. La scelta è ancora in attesa.

## Restante di T-0

- Collegare il primitivo all'inspector, estrarre i moduli per tipo con il contratto `sections(ctx)`.
- Intestazione da 72 px, riga Tipo, menu oggetto, componenti restanti e conversione degli interruttori.
- Adattare la navigazione delle spec tramite `openSection`, verificare le spec esistenti dell'area,
  fedeltà it chiaro/scuro e aggiornare la ricetta delle catture.
- Risolvere ▶ Prova secondo la risposta dell'utente.
- Solo al completamento: test T-0 verdi, riepilogo definitivo, status completato e merge `--no-ff`.

## Tavole e T-10

Letti gli HTML e visti i PNG InspectorComponents, RingInspectorOpen e MainNew.
Il browser ha rifiutato l'apertura `file:` per la sua policy; nessun aggiramento effettuato.
Il confronto finale G3 resta da svolgere quando l'inspector usa i componenti nuovi: attualmente
la UI di produzione è invariata, quindi non si dichiara fedeltà alle tavole.

Nessun rinvio funzionale a T-10 approvato per ora. Restano i controlli trasversali di T-10 previsti dal piano;
l'eventuale rinvio di ▶ Prova dipende dalla risposta dell'utente.
