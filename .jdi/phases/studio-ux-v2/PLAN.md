# Fase 21: Studio UX v2 — Piano  (slug: studio-ux-v2)

## Obiettivo
Vedi `CONTEXT.md`. In una riga: Studio uguale alle tavole di `design/` (inspector a sezioni, pannelli, barre, profili,
galleria) più le funzioni nuove dei widget, con renderer, file del tema, demo e test.

## Per chi esegue (Claude o ChatGPT) — leggere prima di ogni task
1. Leggere `AGENTS.md` (regole condivise: test solo di ciò che si tocca, worktree, commit, versioni) e `CLAUDE.md`
   (lingue: codice e commit in inglese, documenti `.jdi/` in italiano, UI solo via i18n en/pt-BR/it).
2. Leggere `CONTEXT.md` e le decisioni `.jdi/decisions/D-2026-10-20-studio-ux-v2-*.md` citate dalla task.
3. Aprire le tavole della task: `design/png/<Tavola>.png` (immagine) e `design/boards/<Tavola>.html` (HTML statico da aprire
   nel browser: misure, colori, testi esatti). Il canvas su claude.ai non serve. Icone: `design/svg/icons/*.svg`
   (24×24, tratto 1,75, `currentColor`), meteo animato: `design/svg/weather/*.svg`, logo e simboli maniglie: `design/svg/misc/`.
   Indice tavola → task: `design/README.md`.
4. Lavorare in una worktree propria: `git worktree add -b ux2/<task> /f/dev/bezel-ux2-<task> <commit di partenza>`; non fare
   push; chi integra fa `git merge --no-ff`. Prima di toccare file ampi controllare `git log` (più agenti nello stesso repo).
5. Commit Conventional, piccoli, scope `studio-ux-v2`, es. `feat(studio-ux-v2): needle colour zones in the renderer`. Trailer
   dell'agente che scrive: `Co-Authored-By: ChatGPT <noreply@openai.com>` oppure
   `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`.
6. A fine task: `SUMMARY-T-<n>.md` in questa cartella (italiano): cosa è fatto, commit, test eseguiti con esito, differenze
   rimaste rispetto alle tavole con il motivo, primitivi riusati e «nessuna duplicazione», cose lasciate a T-10.
   Aggiornare lo **Status** della task qui sotto.

## Contratti condivisi (fissati qui; T-0 e T-1 li creano, le altre task li usano)

**Struttura dell'inspector (T-0).** `src/ui/inspector.js` (1534 righe) si divide in:
- `src/ui/inspector.js` — guscio: sceglie il modulo per il tipo di oggetto, intestazione, riga Tipo, ordine delle sezioni.
- `src/ui/inspector/<tipo>.js` — un modulo per tipo: `value.js` (valore e testo fisso), `ring.js`, `bar.js`, `needle.js`,
  `graph.js`, `clock.js`, `weather.js`, `player.js`, `spectrum.js`, `shape.js`, `image.js`, `card.js` (da
  `inspector-card.js`), `theme.js` (nessuna selezione, da `inspector-background.js`), `multi.js` (più oggetti).
  Ogni modulo esporta `sections(ctx) → [{ id, title, icon, summary(element), render(body, ctx) }]` con `id` fra
  `layout | data | shape | text | color | behavior` (più quelle proprie della card: `faces | motion | triggers`).
- `src/inspector-sections.js` (puro, testato al 100%): `SECTION_ORDER`, `createSectionMemory()` (aperte/chiuse per tipo,
  persistito in `localStorage` con try/catch), `toggle(kind, id, { exclusive })` (Alt+clic), `summaryOf(...)` helper di
  formattazione dei riassunti (numeri con la lingua, unità).
- Sostituisce `inspector-tabs.js` (CARD_TABS/OBJECT_TABS): da togliere quando nessuno lo usa più (T-10).

**Componenti (T-0)** in `src/ui/fields.js` + `src/styles/base.css`, un'implementazione ciascuno (D-04, tavola
InspectorComponents): `accordionSection`, `numberField` (prefisso testo o icona, unità, ⚡ facoltativo), `iconChoice`
(segmented a icone o testo, `wide`), `previewChoice` (card con anteprima), `cornersField`, `fadeField` (nessuna/lineare/
radiale + direzione o centro/morbidezza), `imageField`, `sizeField`, `bindButton` (⚡), `objectHeader`, `typeRow`,
`partsRow` (parti cliccabili + editor della parte). Colore e font restano `colorPicker`/`fontPicker` esistenti.
Icone nuove in `src/ui/icons.js` prese da `design/svg/icons/` (nome file = nome icona).

**File del tema (T-2…T-6, Rust).** Campi nuovi sempre facoltativi, `#[serde(default, skip_serializing_if = ...)]`,
compatibili con `deny_unknown_fields`; un tema vecchio si apre e si salva identico (test di andata e ritorno per ogni campo).
Nomi JSON in camelCase. Ogni campo nuovo: dominio in `crates/bezel-core/src/domain/`, DTO in `crates/bezel-themes/src/dto.rs`
(+ `native.rs` se serve), resa in `crates/bezel-render/src/`, stessa resa in `apps/bezel-studio/src/demo-render.js`.

**i18n.** Chiavi nuove solo in `src/i18n/areas/<area>.js` (`inspector.` → `insp.`, library, shell, screen, pickers);
basi `en/pt-BR/it.js` non si toccano. Testo comune = una chiave.

## Guardrail (valgono per OGNI task, citati come G)
- G1 test unitari UI del modulo toccato verdi (`npm run test:unit -- <file>` o equivalente); logica nuova pura fuori da `src/ui/`, con test.
- G2 e2e solo dell'area: `node scripts/e2e-slot.mjs tests/e2e/<spec>` da `apps/bezel-studio`; una spec instabile si rilancia da sola.
- G3 confronto con le tavole: `node scripts/fidelity/shoot.mjs --area <a>` (it, scuro e chiaro) contro `design/png/`.
- G4 nessuna asserzione indebolita; una spec di un'altra task non si modifica (si annota per T-10).
- G5 solo dati veri dal bridge; nessun nome o valore delle tavole nel codice (le tavole usano dati d'esempio).
- G6 colori solo da `src/styles/tokens.css` (`static-guards` verde). G7 i18n a parità. G8 controlli nativi, tastiera, aria, `expectAccessible` chiaro e scuro.
- G9 riuso (D-04): nessuna variante locale di un componente; serve un componente nuovo comune → annotarlo nel SUMMARY per T-10.
- G10 Rust: `cargo test -p <crate>` e `cargo clippy -p <crate> --all-targets -- -D warnings` dei crate toccati; `cargo fmt`.
- G11 prestazioni: nessun lavoro in più a ogni fotogramma quando la funzione è spenta; animazioni con `next_ms` come i GIF,
  ferme con «movimento ridotto» e quando l'oggetto non è visibile.

## Tasks

### Wave 1 — Fondazione UI e motore (in parallelo)

#### T-0: Fondazione dell'inspector — sezioni apribili, componenti, divisione per tipo
- **Decisioni:** 01, 02, 03, 04 · **Tavole:** InspectorComponents, RingInspectorOpen, MainNew (pannello destro)
- **Files:** `src/ui/inspector.js`, `src/ui/inspector/*.js` (nuovi, spostamento del codice esistente SENZA funzioni nuove),
  `src/inspector-sections.js` (nuovo), `src/inspector-tabs.js`, `src/ui/inspector-card.js`, `src/ui/inspector-background.js`,
  `src/ui/fields.js`, `src/ui/icons.js`, `src/styles/{base,inspector}.css`, `src/i18n/areas/inspector.js`,
  `tests/ui/inspector-sections.test.mjs` (nuovo), `tests/ui/*.test.mjs` toccati, `tests/e2e/inspector-*.spec.mjs`, `tests/e2e/helpers.mjs`,
  `scripts/fidelity/recipes/inspector.mjs`
- **Estensione approvata dall'utente (20 ottobre):** includere in T-0 il supporto completo di ▶ Prova nel bridge,
  nel motore Rust condiviso e nel demo; file aggiuntivi `src/{bridge,demo-backend,demo-render,inspector-trial,weather-icon}.js`,
  `src-tauri/src/{backend,commands,studio}.rs`, `crates/bezel-core/src/app/runtime.rs`,
  `crates/bezel-runtime/src/{inspector_trial,lib,wire}.rs` e test delle aree toccate. Restano vietate build prodotto e pubblicazione.
- **Acceptance:**
  - Ogni tipo di oggetto mostra le sue voci di oggi raggruppate nelle sezioni nell'ordine D-01, con riassunto da chiuse;
    Alt+clic esclusivo; stato per tipo ricordato tra selezioni e riavvii.
  - Intestazione D-02 (anteprima 72 px da `renderElement`, ▶ prova come l'anteprima animata esistente `preview-animation.js`,
    ⋯ con le azioni del menu oggetto esistente) e riga Tipo per valore/barra/anello/lancetta/grafico (cambia `kind` come oggi «Mostra come»).
  - Componenti del contratto in `fields.js`, documentati nel catalogo dei primitivi; gli interruttori dell'inspector diventano pulsanti-icona.
  - Nessuna funzione persa: le spec e2e dell'inspector esistenti passano (adattando solo come si apre una sezione al posto di una scheda, in `helpers.mjs`: `openSection(page, id)`).
- **Dipendenze:** nessuna · **Test:** G1–G11 secondo i file toccati · **Status:** completed — sezioni e componenti collegati, intestazione 72 px, Tipo e menu condiviso; ▶ Prova completo nativo/demo. 595 test Rust, clippy/fmt, 188 unitari UI e 96 e2e distinti dell'area verdi; riassunti e leggibilità italiana verificati, confronto it chiaro/scuro registrato. Vedi `SUMMARY-T-0.md` e `INSPECTOR-COMPONENTS.md`. Ultimi commit di codice/verifica: `812fbaf`, `29de8a3`, `374a14c`, `fc4cf6b`.

#### T-1: Testo — cifre fisse, parti con stile, testo troppo lungo, transizioni del numero
- **Decisioni:** 05, 06 · **Tavole:** ValueInspectorOpen, ClockInspectorOpen (parti digitali)
- **Files:** `crates/bezel-core/src/domain/{theme,clock,transition}.rs`, `crates/bezel-render/src/{text,fonts,renderer}.rs`,
  `crates/bezel-themes/src/dto.rs`, test Rust nei rispettivi `tests/` o moduli, `apps/bezel-studio/src/{demo-render,clock-format,value-transition}.js`, `tests/ui/*.test.mjs` relativi
- **Acceptance:**
  - `tabularDigits: bool` (predefinito vero per testi con numeri; omesso se vero) — tnum via shaping se il font lo offre,
    altrimenti avanzamento uguale per 0–9. Test: larghezza di «11» = «88» con IBM Plex Sans e con Inter.
  - Parti: `parts: { prefix?, number?, unit?, suffix? }` per il Valore e `{ hours?, minutes?, seconds?, separators? }` per l'orologio,
    ciascuna `{ font?, sizePercent?, weight?, color?, baseline: 'base'|'super' }`; separatori `blink: bool`.
  - Testo troppo lungo: `overflow: 'shrink'|'ellipsis'|'scroll'` (scroll orizzontale con `next_ms`).
  - Transizione del numero: oltre a Salta/Fluida/Inerzia esistenti, `count` (il numero scorre) e `roll` (cifre a rullo);
    `average: n` letture (1 = spento).
  - Andata e ritorno DTO per ogni campo; pixel test per parti e cifre fisse; demo-render uguale.
- **Dipendenze:** nessuna · **Test:** G10, G1 · **Status:** completed — cifre fisse (tnum + avanzamento uguale), parti con stile e separatori lampeggianti, Rimpicciolisci/Taglia/Scorre, Conta/Rullo/Media nel dominio, nel file del tema, nel renderer e nel demo. 314+80+108 test Rust, clippy/fmt, 81 unitari UI verdi; e2e dell'area 56 verdi, 28 rossi già rossi sulla base (spec ancora a schede, per T-10). Vedi `SUMMARY-T-1.md`. Commit `7465262`, `ba0a6d0`, `39a0650`, `5d5689e`.

#### T-2: Orologio analogico
- **Decisioni:** 07 · **Tavole:** ClockInspectorOpen (colonna Analogico)
- **Files:** `crates/bezel-core/src/domain/{theme,clock}.rs`, `crates/bezel-render/src/` (nuovo `analog_clock.rs` o in `gauges.rs`),
  `crates/bezel-themes/src/dto.rs`, `apps/bezel-studio/src/demo-render.js`, test
- **Acceptance:** stile `analog` con tutte le opzioni D-07; secondi a scatti (ridisegno al secondo) o fluidi (`next_ms` ≈ 33 ms solo
  se la lancetta dei secondi è visibile); data nel quadrante; test di geometria delle lancette a 3:00, 6:30, 9:45:30; demo uguale.
- **Dipendenze:** nessuna · **Test:** G10, G11 · **Status:** pending

#### T-3: Lancetta estesa
- **Decisioni:** 08 · **Tavole:** NeedleInspectorOpen, CanvasHandles (scheda Lancetta)
- **Files:** `crates/bezel-core/src/domain/theme.rs` (needle), `crates/bezel-render/src/gauges.rs`, `crates/bezel-themes/src/dto.rs`,
  `apps/bezel-studio/src/demo-render.js`, test
- **Acceptance:** zone (`zones: [{ from, to, color }]`, max 8), arco di fondo unico/zone/sfumatura, colore lancetta dalla zona,
  tacche, numeri della scala (formattati con la lingua), forma/lunghezza/coda, cappuccio, estremità, valore al centro (riusa
  la resa del Valore di T-1 se già unita, altrimenti testo semplice e annotazione), segno del massimo con ritorno dopo N s
  (stato nel motore come `sensor_motion`). Test pixel per zone e cappuccio, test di stato per il massimo.
- **Dipendenze:** nessuna (valore al centro completo dopo T-1) · **Test:** G10, G11 · **Status:** pending

#### T-4: Icone meteo animate
- **Decisioni:** 09 · **Tavole:** WeatherAnimations, WeatherInspectorOpen (Comportamento)
- **Files:** `crates/bezel-render/src/weather.rs`, `crates/bezel-core/src/domain/{theme,weather}.rs`, `crates/bezel-themes/src/dto.rs`,
  `apps/bezel-studio/src/demo-render.js`, test
- **Acceptance:**
  - `iconMotion: 'still'|'gentle'|'lively'` (predefinito `still` per i temi esistenti, `gentle` per i nuovi oggetti meteo),
    `iconSpeed` 0.5–2, dissolvenza 300 ms al cambio di famiglia.
  - Movimenti per famiglia come in `design/weather-motion.md` (tempi e ampiezze) e nei riferimenti `design/svg/weather/`;
    stessi movimenti nei 4 stili; il 3D aggiunge l'ombra che segue e il lampo sul riquadro.
  - Funzione pura `pose(family, t, amplitude) → trasformazioni` testata (periodicità: pose(t) = pose(t + periodo)); `next_ms`
    ≤ 50 ms solo con movimento attivo; fermo con movimento ridotto.
- **Dipendenze:** nessuna · **Test:** G10, G11 · **Status:** pending

#### T-5: Immagine e video
- **Decisioni:** 04 (Immagine), tavola ImageInspectorOpen
- **Files:** `crates/bezel-core/src/domain/{theme,framing}.rs`, `crates/bezel-render/src/images.rs`, `crates/bezel-themes/src/dto.rs`,
  `apps/bezel-studio/src/demo-render.js`, test
- **Acceptance:** punto di ancoraggio (9 posizioni) per Riempi; trasparenza radiale con centro e morbidezza (lineare esiste);
  GIF/video: riproduzione in ripetizione/una volta e velocità 0,5×/1×/2×. Test pixel dell'ancoraggio e della trasparenza radiale.
- **Dipendenze:** nessuna · **Test:** G10 · **Status:** pending

#### T-6: Profili — nuove regole e profilo attivo
- **Decisioni:** 12 · **Tavole:** ProfilesNew
- **Files:** `crates/bezel-runtime/src/profiles.rs`, `crates/bezel-sensors/src/windows/activity.rs` (inattività: `GetLastInputInfo`;
  schermo bloccato: sessione/desktop di input), `crates/bezel-core/src/domain/sensor.rs` (Snapshot), `apps/bezel-studio/src-tauri/src/{commands,backend,dto}.rs`,
  `apps/bezel-studio/src/{bridge,demo-backend,profiles-model}.js`, test
- **Acceptance:** `Source` += `Sensor { key, above|below, value, forSeconds }`, `Time { days, from, to }`, `Idle { minutes }`, `Locked`;
  `Profile.match: 'any'|'all'` (predefinito any, omesso); il comando dei profili restituisce `active: { profile, reason }` e
  per ogni regola `trueNow`; test del motore per ogni tipo e per any/all; Light usa lo stesso codice (nessuna duplicazione).
- **Dipendenze:** nessuna · **Test:** G10, G1 · **Status:** pending

### Wave 2 — UI per area (in parallelo; parte dal merge di T-0 e delle task Rust già finite)

#### T-7a: Inspector dei widget collegati — Valore/Testo, Anello, Barra, Lancetta, Grafico
- **Decisioni:** 01–06, 08 · **Tavole:** ValueInspectorOpen, RingInspectorOpen, BarInspectorOpen, NeedleInspectorOpen, GraphInspectorOpen, MultiInspectorOpen
- **Files:** `src/ui/inspector/{value,ring,bar,needle,graph,multi}.js`, `src/styles/inspector.css`, `src/i18n/areas/inspector.js`, test unitari e e2e dell'area
- **Acceptance:** sezioni e campi esattamente come le tavole, incluse le funzioni di T-1 e T-3 (parti del valore, cifre fisse, soglie,
  Conta/Rullo, Media su; zone, tacche, cappuccio, segno del massimo); selezione multipla con soli campi comuni e «misto».
- **Dipendenze:** T-0, T-1, T-3 · **Test:** G1–G9 · **Status:** pending

#### T-7b: Inspector di Orologio, Meteo, Player, Spettro, Forma, Immagine
- **Decisioni:** 01–07, 09 · **Tavole:** ClockInspectorOpen, WeatherInspectorOpen, PlayerInspectorOpen, SpectrumInspectorOpen, ShapeInspectorOpen, ImageInspectorOpen
- **Files:** `src/ui/inspector/{clock,weather,player,spectrum,shape,image}.js`, css, i18n, test dell'area
- **Acceptance:** come le tavole, con orologio digitale/analogico (T-1, T-2), animazione meteo (T-4), immagine/video (T-5),
  forma con immagine ripetuta (esistente), copertina del player con trasparenza (esistente).
- **Dipendenze:** T-0, T-1, T-2, T-4, T-5 · **Test:** G1–G9 · **Status:** pending

#### T-7c: Inspector di Card e Tema; pannelli Schermo
- **Tavole:** CardInspectorOpen, ThemeInspectorOpen, ScreenPanelsOpen
- **Files:** `src/ui/inspector/{card,theme}.js`, `src/ui/{screen-panel,storage,manager,standby}.js`, `src/styles/{card,screen}.css`, i18n, test dell'area
- **Acceptance:** come le tavole; Card senza la doppia «Transizione tra le facce»; nomi delle facce non tagliati; Schermo con riavvio e
  rilascio della porta nel menu ⋯.
- **Dipendenze:** T-0 · **Test:** G1–G9 · **Status:** pending

#### T-8: Canvas — maniglie per oggetto e barra mobile
- **Decisioni:** 10 · **Tavole:** CanvasHandles, MainNew (canvas)
- **Files:** `src/ui/canvas.js`, `src/editor/{geometry,snap,rotation}.js`, nuovo `src/editor/handles.js` (puro: quali maniglie per tipo,
  da posizione a valore e ritorno), `src/ui/object-menu.js`, `src/styles/shell.css`, i18n shell, test
- **Acceptance:** maniglie di ogni scheda della tavola (anello, lancetta, barra, forma, grafico, valore/testo, orologio analogico, immagine,
  player, card, spettro, più oggetti) con valore vicino al cursore e campo illuminato; barra mobile (sensore, ▶, duplica, ⋯); pulsante
  «valori di prova» nella barra degli strumenti; tastiera: frecce sulla maniglia attiva. `handles.js` 100% coperto.
- **Dipendenze:** T-0 (illuminazione campo), T-3 (confini zona) · **Test:** G1–G9 · **Status:** pending

#### T-9a: Pannelli Widget, Sensori, Livelli, Media
- **Decisioni:** 14 · **Tavole:** LibraryPanelsNew, MainNew (pannello sinistro)
- **Files:** `src/ui/{library,layer-drag,collection,icon-picker,gif-search}.js`, `src/{library-model,sensor-probes}.js`, `src/editor/layers.js`,
  `src/styles/library.css`, `src/i18n/areas/library.js`, test dell'area
- **Acceptance:** come le tavole; nessun nome tagliato nelle righe Sensori alle larghezze 240/272/360 (test `scrollWidth ≤ clientWidth`);
  «in uso: …» calcolato dai riferimenti reali del tema (stesso controllo del cestino esistente).
- **Dipendenze:** T-0 · **Test:** G1–G9 · **Status:** pending

#### T-9b: Barra in alto e barra di stato
- **Decisioni:** 11 · **Tavole:** MainNew (barra in alto), StatusBarNew
- **Files:** `src/index.html` (header e footer), `src/app.js`, `src/ui/{sensor-status,memory-status}.js`, `src/{libre-status,memory-status,relative-time}.js`,
  `src/styles/shell.css`, `src/i18n/areas/shell.js`, test
- **Acceptance:** tre chip con i casi della tavola e i tre riquadri; selezione/trascinamento a sinistra; salvataggio accanto al nome del tema;
  profilo attivo nel chip dello schermo (da T-6, finché manca: nascosto); versione nel menu ⋯.
- **Dipendenze:** T-0; T-6 per il profilo · **Test:** G1–G9 · **Status:** pending

#### T-9c: Dialogo Profili e Galleria temi
- **Decisioni:** 12, 13 · **Tavole:** ProfilesNew, ThemesNew
- **Files:** `src/ui/{profiles,themes-dialog}.js`, `src/{profiles-model,theme-filter}.js`, css, i18n, test
- **Acceptance:** come le tavole; regole come frasi con parti cliccabili; ▶ attiva per 30 s (comando esistente di prova se c'è, altrimenti nuovo
  e annotato); galleria con anteprime reali (render piccolo in cache per tema), distintivi, sensori mancanti calcolati dal catalogo.
- **Dipendenze:** T-0, T-6 · **Test:** G1–G9 · **Status:** pending

#### T-9d: Temi predefiniti 1920×480
- **Tavole:** ThemePixel, ThemeNebula, ThemePaper, ThemeNowPlaying
- **Files:** `themes/<nome>/` (nuove cartelle, formato come i temi moderni esistenti), `crates/bezel-cli/tests/bundled_themes.rs`, asset in vettoriale se servono
- **Acceptance:** quattro temi orizzontali 1920×480 fedeli alle tavole, solo widget nativi e font inclusi (Doto, IBM Plex Mono, ...); test dei temi inclusi verde.
- **Dipendenze:** T-1…T-4 per le funzioni che usano (altrimenti versione senza e annotazione) · **Test:** G10 · **Status:** pending

### Wave 3 — Integrazione

#### T-10: Integrazione, pulizia e verifica
- **Files:** quelli annotati nei SUMMARY; `docs/user/*.md`, `CHANGELOG.md` (voce «Unreleased», niente versione), `.jdi/phases/studio-ux-v2/REVIEW.md`
- **Acceptance:** componenti comuni promossi, chiavi i18n orfane e `inspector-tabs.js` tolti, galleria di fedeltà completa e approvata
  dall'utente, guida utente aggiornata (inspector, maniglie, profili, meteo animato), suite completa verde (AGENTS.md), elenco
  delle prove a vista per l'utente (DoD 6).
- **Dipendenze:** tutte · **Status:** pending

## Ordine consigliato e parallelismo
- Wave 1: T-0 (UI) in parallelo con T-1…T-6 (Rust + demo), fino a 4 agenti alla volta. T-0 è la più urgente perché sblocca la Wave 2.
- Wave 2: T-7a/b/c, T-8, T-9a/b/c/d in parallelo, ognuna con i propri file (vedi **Files**). `src/index.html` è solo di T-9b;
  `src/app.js` solo di T-9b (le altre espongono funzioni e annotano il collegamento).
- Wave 3: T-10, un solo agente.
