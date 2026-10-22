# Verifica finale dopo il riallineamento visivo

22 ottobre 2026. Branch `fid/studio-final-verification`, worktree
`F:/dev/bezel-studio-final-verification`, base `b6849e54` di
`windows-improvements`. L’utente ha dato il via libera a procedere dopo
le correzioni Anello, Spettro e Angoli condivisi.

Questa è la nuova verifica di fase prevista da AGENTS.md: i gate completi
di T-10 precedono il riallineamento. Una sola esecuzione generale;
eventuali problemi si correggono e si verificano soltanto nell’area coinvolta.
Nessuna build di prodotto, installazione o pubblicazione.

## Gate

| Gate | Esito |
|---|---|
| `cargo fmt --all -- --check` | verde |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | verde |
| `cargo llvm-cov --workspace --locked --summary-only --fail-under-lines 80 --ignore-filename-regex '(^|/)(main|build)\.rs$'` | verde: 1412 test superati, 19 ignorati; 90,26% righe, 89,95% regioni, 86,57% funzioni |
| `npm test -- --workers=2`: unitari UI con copertura | 692 verdi; 99,24% righe, 96,11% rami, 96,30% funzioni |
| `npm test -- --workers=2`: e2e nei quattro progetti, base invariata | 1143 verdi, 33 falliti su 1176 (23,7 min); tutti i 33 nelle spec corrette qui sotto |
| e2e mirati sulle correzioni (`fid/studio-verification-fixes`, `17a170fc`) | verdi: 140 casi delle 7 spec coinvolte nei quattro progetti |

Log locali in `target/logs/fidelity-final-verification/` nella worktree:
`fmt.log`, `clippy.log`, `ui-full.log`, `rust-coverage.log`.
Per riutilizzare la cache Rust, `CARGO_TARGET_DIR=F:/Dev/bezel/target`.
La suite UI usa una porta propria e due worker.

## Correzioni isolate durante la passata generale

Le correzioni sono nella worktree separata
`F:/dev/bezel-studio-verification-fixes`, branch `fid/studio-verification-fixes`,
sempre da `b6849e54`: la passata generale continua sulla base invariata.

- `e37d6ce2`: difetto reale nel popover Schermo. Il dato del profilo era
  passato al popover prima di aggiornare il profilo corrente, quindi il chip
  poteva mostrare il profilo e il popover rimanere privo della riga fino a un
  successivo aggiornamento dello schermo. Calcolo ora prima della proiezione
  del popover. 9 unitari `memory-status`, 12 e2e `shell-status` verdi;
  ulteriori 4 casi con verifica rafforzata della prima apertura, verdi.
- `e772d6d3`: spec allineate ai controlli approvati: Adatta al contenuto nel
  menu; metadati della scheda Schermo separati dallo stato; ordine delle
  schede con Temi all’inizio; etichette e icone della transizione. Restano i
  controlli su tastiera, Undo, stato attivo, luminosità e transizioni.
- `17a170fc`: nel trascinamento verso Avvio il test scorre il pannello prima
  di misurare il riquadro, che con le schede file più alte può essere fuori
  vista; restano i controlli su evidenziazione e conferma.

I 33 fallimenti della passata generale sono le stesse 9 prove nei quattro
progetti (`storage` solo in pt): `fit-content`, `live-screen-controls` (2),
`scenarios-screen`, `scenarios-shell`, `shell-status`, `storage`,
`value-transition` (2). Nessun altro errore. Dopo le correzioni, le 7 spec
interessate rieseguite insieme: 140 verdi (`final-targeted.log`).

Log delle correzioni in `target/logs/fidelity-final-verification-fixes/`
nella relativa worktree.

## Esito

Gate automatici verdi sulla base corretta. Le correzioni sono integrate in
`windows-improvements` con `git merge --no-ff`. Nessuna build di prodotto,
installazione, push o pubblicazione.

## Verifica manuale

Le ultime correzioni visive sono accettate nel dialogo con l’utente.
Non si deduce da questo una prova hardware: restano le nove prove sugli
schermi fisici elencate in `REVIEW.md`, sezione «Prove a vista per te».
La fase non viene chiusa automaticamente.
