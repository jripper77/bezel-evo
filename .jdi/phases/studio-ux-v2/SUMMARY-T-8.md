# T-8 — Canvas: maniglie per oggetto e barra mobile — riepilogo

21 ottobre 2026. Branch `ux2/t8`, worktree `F:/Dev/bezel-ux2-t8`, base `c66a247`.
Lavoro di ChatGPT fino a `0f90ef6` (fermata al limite d'uso), poi di Claude
nella stessa worktree. Decisione D-2026-10-20-studio-ux-v2-10, tavole
CanvasHandles e MainNew. Dettaglio dei passi intermedi in `CHECKPOINT-T-8.md`.

## Cosa è fatto

Ogni scheda della tavola CanvasHandles ha le sue maniglie, con valore vicino al
cursore, campo dell'inspector illuminato (sezione aperta e portata in vista),
frecce sulla maniglia attiva (Maiusc = passo grande) e un solo Undo per gesto.

| Scheda | Maniglie |
|---|---|
| Anello | inizio/fine sul cerchio (Maiusc 15°), spessore |
| Lancetta | perno, lunghezza, inizio/fine, confini delle zone |
| Barra | **valore di prova** (tiene il sensore dove la trascini, non salva), angoli |
| Forma | angoli (Alt = uno solo); doppio clic: sposta/scala l'immagine ripetuta |
| Grafico | min/max trascinabili (valori arrotondati), doppio clic = automatica |
| Valore e testo | rombo di dimensione (Ctrl sulla SE scala il testo), **clic su una parte** la sceglie nell'inspector, doppio clic scrive il testo fisso |
| Orologio analogico | lunghezza delle tre lancette, che restano all'ora vera |
| Immagine | **ancoraggio** (nove posizioni, scatto alla più vicina), sfumatura |
| Media player | **sfumatura sulla copertina vera** e **divisore** copertina/testi |
| Card | linguette delle facce sotto la Card, +, doppio clic per entrare |
| Spettro | barre e spazio |
| Più oggetti | riquadro comune proporzionale, rotazione comune se ammessa, guide rosa con distanza |

**Barra mobile** sopra la selezione: sensore collegato (apre il selettore),
▶ Prova (la prova dell'inspector), Duplica, ⋯ (stesso menu del clic destro).
Nascosta durante un gesto; con più oggetti solo Duplica e ⋯.

**Valori di prova** nella barra del canvas (icona provetta, scelta
dell'utente «Ciclo 0→100%»): il canvas mostra l'intera scena con valori di
prova — ogni sensore collegato va dal minimo al massimo della sua scala e
torna in 3 s, gli orologi girano, il meteo passa le sue famiglie, i player
suonano, i grafici disegnano un'onda; ciclo completo 24 s. Lo schermo fisico
resta sui sensori veri e nulla si salva. Disattivato con «movimento ridotto».

## Commit

ChatGPT: `a138a5c` `a251ded` `9456d79` `bbf4424` `48031ec` `1551269`
`a06f8b1` `2e0a040` `fa71016` `aa2e0cc` `966aec8` `0f90ef6`.

Claude: `a17aa9e` copertina Player e divisore · `f528318` ancoraggio
Immagine · `965b2a8` clic sulle parti · `2eec51b` barra mobile ·
`6eb54f7` checkpoint · `c829d84` prova della scena nel runtime ·
`61bcf99` pulsante Valori di prova e maniglia della Barra · `b58858a`
arrotondamenti · `42addde` evidenze finali.

## Funzione nuova nel runtime (Rust)

- `bezel_runtime::inspector_trial::Trial::scene` (puro): la scena intera con
  gli ingressi di prova, oppure `Hold` = un elemento tenuto a una quota della
  sua scala sopra le storie dal vivo. La scala di un indicatore vince su quella
  di un testo con lo stesso sensore (testo: 0–100). La forma d'onda dei grafici
  è ora una funzione sola condivisa con la prova dell'inspector.
- Operazione `RuntimeOp::Trial { max, motion, ms, hold }` e
  `Engine::trial_canvas`: rendering staccato, la scena dal vivo, gli schermi e il
  documento non cambiano (test via protocollo). `ThemeRuntime::histories()`.
- Studio: `render_preview` accetta `trialMs`/`trialHold`
  (`Backend::render_trial`, percorso engine e percorso interno
  `Studio::canvas_trial`). La demo fa lo stesso (`sceneTrialAt`, `heldTrial`).

## Test eseguiti

- Rust: `bezel-core`, `bezel-runtime`, `bezel-studio` — 624 test, tutti verdi
  da soli; clippy `--all-targets -D warnings` e fmt verdi. Nella libreria di
  `bezel-studio` un test del ponte engine sensibile ai tempi
  (`light_crash_…`, `remote_canvas_…`, `the_preview_says_when_its_gif_changes`)
  fallisce a volte nell'esecuzione completa e passa da solo: **la base
  `c66a247` fa lo stesso** (2 fallimenti su 4 esecuzioni). Log
  `target/logs/t8/trial-*.log`, base in
  `F:/Dev/bezel/target/logs/t8-baseline/`.
- Unitari UI: `npm run test:unit` 657/658; l'unico fallimento,
  `video-framing.test.mjs` («the demo preview plays the video turned by Auto»),
  fallisce identico sulla base. `handles.js` e `inspector-trial.js` **100%**
  righe/rami/funzioni. Log `target/logs/t8/unit-claude-final.log`.
- E2E: le sette spec canvas di T-8 più `inspector-trial` e `inspector-linked`
  insieme, **136 verdi** (en/pt-BR × chiaro/scuro, axe compreso). Log
  `target/logs/t8/e2e-claude-final.log`.
- Confronto con le tavole (G3), italiano chiaro e scuro: `evidence/t8-checkpoint`,
  `t8-selection-text`, `t8-clock`, `t8-bar`, `t8-final` — tutte le dodici schede
  di CanvasHandles, la barra mobile e Valori di prova di MainNew.

## Differenze rimaste rispetto alle tavole, con il motivo

- Ancoraggio Immagine: nove posizioni fisse, perché il formato del tema è
  discreto; niente cerchio tratteggiato della tavola.
- Copertina del Player: senza una sessione dal vivo nell'editor si assume una
  sessione con durata (titolo e avanzamento come configurati).
- Valori di prova e maniglia della Barra: lo sfondo video mostra la locandina
  durante la prova (rendering staccato, come l'anteprima dell'inspector).
  Una maniglia tenuta muove tutti gli oggetti con lo stesso sensore: è la
  stessa lettura.
- Il percorso nativo della prova (Tauri → engine) è coperto da test Rust e del
  backend, non ancora visto nell'app vera: prova a vista per l'utente.

## Primitivi riusati, nessuna duplicazione

`segmented`, `iconButton`, `openMenuAt`/`objectMenuItems`, `selectionRoots`,
`resize`, `patchElements`, comandi delle facce, `fitText`/`layoutText` e
`canvasMeasure` della demo, `formatValue`, `clockParts`, `coverShare`,
`IMAGE_ANCHORS`, `createBurst`, prova dell'inspector (`Trial`,
`preview_detached_sample`). Icona `test-values` presa da `design/svg/icons`.

## Da annotare per T-10

- `src/app.js` (di T-9b): collegamenti del canvas (parti, barra mobile,
  maniglia di prova) e del pulsante Valori di prova. Il pulsante è creato da
  `src/ui/trial-values.js` subito dopo `#grid-toggle`, senza toccare
  `index.html`: T-9b può spostarlo nel markup.
- Fallimenti preesistenti, non modificati (G4): 16 casi `object-rotation` e 4
  `canvas-wheel-zoom:140` (campi in sezioni chiuse), 2 casi `editing-017`
  (stesso motivo), `video-framing.test.mjs`, i test a tempo del ponte engine.
- In italiano il prefisso «distanza» del campo Distanza del Player (T-7b)
  restringe l'input fino a nascondere il numero.
- `highlightField` ora riconosce anche i gruppi di scelta e ritrova il campo se
  l'apertura della sezione ridisegna il modulo.
