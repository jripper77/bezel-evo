# T-0 — Fondazione dell'inspector completata

Data: 20 ottobre 2026. Branch: `ux2/t0`, worktree `F:\dev\bezel-ux2-t0`.
Partenza: `04556ff` di `windows-improvements` (log controllato prima di iniziare).

**T-0 completata e verificata**, pronta per il merge `--no-ff` in `windows-improvements`.
L'estensione approvata dall'utente include il supporto completo di ▶ Prova in Rust, bridge e demo.

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

- `283c11b` — `feat(studio-ux-v2): run isolated inspector trials in native and demo previews`:
  opzione transitoria `trialMs` nel bridge e nel protocollo del motore condiviso;
  `bezel-runtime::inspector_trial::Trial` prepara tema isolato, campione, cronologia e tempo solo per l'anteprima.
  Valori/barre/anelli/lancette vanno dalla lettura corrente al massimo e ritorno in tre secondi;
  l'orologio percorre un giorno; il meteo visita le otto famiglie in otto secondi;
  il player riparte da zero nell'anteprima, la Card prova la faccia successiva, il grafico usa una forma d'onda,
  lo Spettro usa il proprio segnale di prova e le immagini animate ripartono dall'inizio.
  Nessun controllo del player reale, modifica del documento, voce Undo, lettura sensori artificiale nella scena live
  o modifica del fotogramma pubblicato. Gli input sostitutivi passano al renderer Rust esistente.
  Il demo usa gli stessi tempi e sequenze; il meteo ora usa anche geometrie e palette statiche del renderer Rust.
  La prova si interrompe a fine sequenza, al cambio di selezione, con pagina nascosta e con movimento ridotto.
  Lo stato del renderer per la prova è separato da quello della normale anteprima e della diretta.
- `7bdc279` — `fix(studio-ux-v2): restart animated children in inspector trials`:
  anche le GIF contenute in Card o gruppi ripartono da zero nel demo, come nel renderer Rust.
  Verifica pixel sul primo fotogramma della GIF figlia, con orologio reale diverso dal tempo di prova.
- `6aac95e` — `refactor(studio-ux-v2): extract inspector controls into widget modules`:
  controlli esistenti estratti in `ui/inspector/{value,clock,weather,bar,ring,needle,graph,player,spectrum,shape,image,card,theme,multi}.js`;
  campi comuni in `common.js`, schemi delle anteprime in `art.js`. Il guscio è sceso da circa 1.550 a 724 righe.
  Nessun controllo eliminato e nessuna asserzione e2e modificata. La navigazione resta momentaneamente quella esistente:
  le API `panels(ctx)`/`panel(ctx)` sono il passaggio intermedio verso il contratto `sections(ctx)` da completare in T-0.
  `inspector-card.js` è un semplice re-export del modulo nuovo, per conservare il chiamante diretto di una spec
  che verrà migrata alla verifica T-10 (G4). Il selettore di sfondo esistente è ancora riusato dal modulo Tema.

Primitivi riusati: `el`, `icon`, `numberField`, icone esistenti, token esistenti, `createPreviewAnimation`,
`createPreviewScheduler`, renderer nativo e `ThemeRuntime::preview_detached` (esteso con input temporanei).
**Nessuna duplicazione:** memoria in un solo modulo puro; markup accordion in un solo builder;
stile accordion nel solo catalogo condiviso `base.css`.

## Test eseguiti

Verifica finale della fondazione:

- **188/188 test unitari UI mirati verdi**: memoria, modello delle sezioni, prova, bridge/demo,
  anteprime, visualizzazioni, meteo, parti player, guardrail e i18n. Dopo l'ultimo perfezionamento
  dei riassunti, altri **54/54** controlli mirati verdi. `inspector-sections.js`: righe/rami/funzioni **100%**.
- `node scripts/e2e-slot.mjs 'tests/e2e/inspector-.*.spec.mjs'`: **92/92 verdi**
  (en/pt-BR × chiaro/scuro), inclusi i nuovi componenti. Log: worktree,
  `target/logs/studio-ux-v2-t0/inspector-verified.log`.
- Dopo l'ultimo perfezionamento dei riassunti: `inspector-redesign` e `inspector-sections`,
  **20/20 verdi**. Ogni sezione di ogni tipo ha un riassunto non vuoto da chiusa;
  nessun valore enum tecnico esposto senza traduzione.
- Verifica finale dei prefissi brevi e dei componenti: **24/24 e2e verdi**, inclusi quattro casi
  in italiano con numeri a quattro cifre nel pannello stretto. In totale **96 casi e2e distinti**
  dell'area verificati. Ultimo controllo statico/i18n: **51/51 verdi**.
- Verificati dimensioni collegate, Undo singolo, collegamento che non sporca il documento,
  selezione multipla, Tema, tastiera, menu condiviso (duplica/elimina/nascondi/blocca),
  stato per tipo e persistenza dopo riavvio. `expectAccessible` verde anche sui componenti opzionali.
- `git diff --check`: verde. Nessuna asserzione ignorata o eccezione axe introdotta.

Seguono le verifiche dei passi precedenti della stessa attività (Rust invariato dopo quei controlli).

- `node --test --experimental-test-coverage --test-coverage-include=src/inspector-sections.js --test-coverage-lines=100 --test-coverage-branches=100 --test-coverage-functions=100 tests/ui/inspector-sections.test.mjs`:
  **9/9 verdi, righe/rami/funzioni 100%**.
- `node --test tests/ui/static-guards.test.mjs tests/ui/inspector-sections.test.mjs tests/ui/i18n.test.mjs`:
  **60/60 verdi**; token, assenza duplicazioni CSS/SVG, CSP e parità i18n inclusi.
- `node scripts/e2e-slot.mjs tests/e2e/inspector-primitives.spec.mjs --output=../../target/logs/studio-ux-v2-t0/primitives`:
  **8/8 verdi**, chiaro/scuro × en/pt-BR. Verificati memoria controllata, Alt+clic,
  conservazione input, altezza, assenza di overflow, tastiera e `expectAccessible`.
- `git diff --check`: verde.
- Dopo `283c11b`: `cargo test -p bezel-core -p bezel-runtime -p bezel-studio --locked`:
  **595 test verdi**, inclusi integrazione e doc-test dei soli tre crate toccati.
  Nuovi test: input temporanei isolati nel core, scenari di prova per tipo, protocollo senza modifica di live/tema/campione,
  test pixel nativo (barra al 100% e ritorno, movimento ridotto, cadenza 33 ms).
- `cargo clippy -p bezel-core -p bezel-runtime -p bezel-studio --all-targets --locked -- -D warnings`: **verde**.
  `cargo fmt -p bezel-core -p bezel-runtime -p bezel-studio -- --check`: **verde**.
- Moduli JS nuovi `inspector-trial.js` e `weather-icon.js`: **4 test verdi, righe/rami/funzioni 100%**.
- Test UI mirati (prova, sezioni, anteprime, bridge, demo, visualizzazioni, meteo, parti player, guardrail e i18n): **165/165 verdi**.
- `inspector-trial.spec.mjs`: **12/12 verdi**; `inspector-preview.spec.mjs` e `inspector-redesign.spec.mjs`: **32/32 verdi**.
  Compresi accessibilità chiaro/scuro, annullamento al cambio selezione e al cambio della preferenza di movimento,
  conservazione del salvataggio, resa di tutti i tipi e otto condizioni nei quattro stili meteo.
- Dopo l'estrazione: `node scripts/e2e-slot.mjs 'tests/e2e/inspector-.*.spec.mjs' tests/e2e/card-inspector.spec.mjs --output=../../target/logs/studio-ux-v2-t0/modular-regression`:
  **112 casi distinti verificati**. Nella prima esecuzione 100 verdi e 12 falliti per un collegamento errato del modulo Player
  (restituiva il gruppo Avanzamento invece dell'intero pannello); corretto il codice e rilanciati solo quei 12 casi,
  tutti verdi, senza modificare i test. In più, 4 casi mirati verdi per il replay delle GIF figlie.
- Ultimo controllo unitario dell'estrazione: sezioni, prova, guardrail e i18n, **64/64 verdi**.
- Per i test Rust è stato riusato `F:\dev\bezel\target` tramite `CARGO_TARGET_DIR` (solo profilo test/dev);
  nessun eseguibile prodotto o installato sostituito.
- Nessuna suite completa, build prodotto, modifica di VERSION, setup, push o pubblicazione.

## Chiarimento risolto

L'accettazione T-0 richiede ▶ Prova con escursione del valore e ritorno, equivalente per orologio,
meteo e player, e cita `preview-animation.js` come funzione esistente da riusare.
Il codice verificato offre invece soltanto pianificazione dei fotogrammi da `nextMs`:

- `src/preview-animation.js`: timer, stop, movimento ridotto; nessuna sequenza di valori di prova.
- `src/bridge.js`, `renderElement`: opzioni solo `max` e `motion`.
- `src/demo-backend.js`, `renderElement`: le stesse opzioni.
- `src/app.js`: inoltra al bridge il tema corrente; nessun contesto di prova.

Le modifiche bridge/Rust non erano tra i file di T-0. Il 20 ottobre l'utente ha risposto
«Includere il supporto completo in T-0». Estensione del perimetro registrata anche nel PLAN;
il supporto è ora implementato nel commit `283c11b` e non è rinviato a T-10.

## Completamento

- `812fbaf` — componenti condivisi in `fields.js` e `base.css`, icone offline nel catalogo centrale.
- `29de8a3` — sezioni persistenti collegate a tutti i moduli, intestazione da 72 px, riga Tipo,
  menu che riusa `objectMenuItems`, interruttori come pulsanti nativi `aria-pressed`.
- `374a14c` — migrazione delle spec dell'inspector, `openSection`, prove dei componenti e ricetta di fedeltà.
- `fc4cf6b` — prefissi brevi L/A (W/H in inglese) per leggere interamente i valori anche in italiano;
  etichette accessibili complete conservate. Catture lunghe senza il piè di pagina sovrapposto.
- Tutti i moduli espongono `sections(ctx)`; il guscio ordina e ricorda le aperture.
  Le precedenti funzioni `panels` restano dettagli interni, così T-7 può sostituire i campi per tipo.
- [Catalogo dei componenti](INSPECTOR-COMPONENTS.md): API, responsabilità e riuso per T-7.
  **Nessuna duplicazione:** angoli riusano `cornerRadii`; trasparenza, scelte con anteprima,
  scelte a icone e intestazione hanno un solo builder; colori/font restano nei picker esistenti.
- Una collisione CSS con `.icon-choice` della libreria è stata corretta usando `.field-choice`
  per il componente del pannello; contrasto e dimensioni nuovamente verificati, senza deroghe ai test.

## Tavole e T-10

Letti gli HTML e visti i PNG InspectorComponents, RingInspectorOpen e MainNew.
Il browser ha rifiutato l'apertura `file:` per la sua policy; nessun aggiramento effettuato.
Eseguito `node scripts/fidelity/shoot.mjs --area inspector --only app
--state ux2-ring-closed,ux2-ring-open,ux2-main,ux2-theme --scheme both` in italiano:
otto stati catturati, con dettaglio del pannello. Confronto visivo eseguito sui PNG offline.

Evidenze conservate: [anello chiuso scuro](verification/t0/ring-closed-dark.png),
[chiaro](verification/t0/ring-closed-light.png), [aperto scuro](verification/t0/ring-open-dark.png),
[chiaro](verification/t0/ring-open-light.png). Originali e log nella worktree sotto
`target/logs/studio-ux-v2-t0/fidelity-final/`.

Differenze intenzionali rispetto alle tavole complete:

- Dati, nomi e dimensioni provengono dal tema/demo, non dagli esempi delle tavole.
- I controlli attuali rimangono nei loro sottogruppi all'interno delle nuove sezioni.
  La disposizione specifica dei campi, le nuove opzioni e le nuove funzioni dei widget sono
  esplicitamente oggetto di T-7a/b/c e dei relativi prerequisiti T-1…T-5.
- Il primitivo Trasparenza conserva il modello attuale (lineare/radiale, centro, estensione,
  punto medio, opacità iniziale/finale); T-5 estenderà il modello per i nuovi usi dell'immagine.
- L'anteprima mantiene un fondo scuro dai token per rendere leggibile anche testo bianco nel tema chiaro.

Nessun rinvio funzionale di T-0 a T-10. Restano la verifica trasversale e la pulizia previste dal piano:

- Togliere `inspector-tabs.js` dopo aver trasferito i suoi helper puri ancora usati e le chiavi obsolete.
- Togliere il re-export temporaneo `inspector-card.js` dopo la migrazione del chiamante diretto.
- Per G4, non modificate le spec di altre attività che interrogano direttamente le vecchie schede
  (`card-inspector`, `corner-radii`, `editing-017`, `text-case`, `value-color`), né i selettori diretti
  dei vecchi switch/Mostra come nelle altre aree. Migrarli alla verifica T-10; il compatibile
  `openInspectorTab` ora apre le nuove sezioni per i chiamanti che già usano l'helper.

## Consegna

Nessun chiarimento pendente. Nessun push, build prodotto, setup, pubblicazione o modifica di VERSION.
T-0 è l'unica attività completata in questa consegna; le altre restano nello stato del PLAN.
