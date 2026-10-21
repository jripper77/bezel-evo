# T-9b — Barra in alto e barra di stato — riepilogo

21 ottobre 2026, Claude. Branch `ux2/t9b`, worktree `F:/dev/bezel-ux2-t9b`,
base `34d950d`. Decisione D-2026-10-20-studio-ux-v2-11, tavole MainNew (barra
in alto) e StatusBarNew. In parallelo Codex/ChatGPT lavora a T-9c.

## Cosa è fatto

- **Tre chip a destra, in ordine fisso**, numeri a larghezza fissa:
  - **Sensori** «sensori 20/21» (leggibili su tutti, LibreHardwareMonitor
    compreso), verde/giallo/rosso; il riquadro ha il titolo «Sensori · 20 di 21
    attivi», le sonde, Apri Sensori e Riavvia (già esistenti).
  - **Bezel** «Bezel 11 ms · 182 MB»: ultimo rendering e memoria insieme,
    giallo oltre il limite di un fotogramma (33 ms). Riquadro: grafico dei
    rendering dell'ultimo minuto, «render 11 ms (limite 33 ms)», memoria per
    processo, totale, nota e **Copia diagnostica** (appunti).
  - **Schermo** «Turing 8,8″ 30 fps» verde in diretta, grigio «non in diretta»
    o «nessuno schermo», giallo con errori/riconnessione/video mancante.
    Riquadro: cosa fa lo schermo in parole (`#status-device`), collegamento,
    risoluzione e rotazione, fotogrammi, tema sullo schermo, profilo;
    **Impostazioni schermo** e **Ferma la diretta / Vai in diretta**.
- **A sinistra**: oggetto selezionato con icona, nome, X · Y e misure (e
  angolo se ruotato); più oggetti con il riquadro comune; senza selezione
  «Nessuna selezione · clic su un oggetto o trascina un widget». Durante
  spostamento, ridimensionamento o rotazione: «Sposto CPU», posizione dal vivo,
  «allineato al centro / a un bordo» dalle guide reali, e il tasto utile
  («Alt = senza aggancio», «Maiusc = proporzioni», «Maiusc = scatti di 15°»).
  Le scorciatoie indicate sono quelle che il canvas ha davvero.
- **Barra in alto**: lo stato di salvataggio («● modifiche non salvate · ultimo
  salvataggio 4 min fa») sotto il nome del tema, che si adatta al contenuto con
  la matita accanto; tolti briciole «Temi ›», marchio testuale e chip «Non
  salvato» ripetuti. Nuovo **⋯** con Preferenze e la versione.
- **Profilo attivo nel chip dello schermo** («◯ Gioco · manuale/automatico»),
  da `bridge.profiles().status` di T-6, aggiornato ogni 5 s a finestra visibile;
  nascosto quando sono attivi i temi di base o il motore non risponde: nessuno
  stato inventato.

## Commit

`054b353` tre chip · `d715ef9` salvataggio, parte sinistra, ⋯ e profilo ·
più il commit di questo riepilogo e delle evidenze.

## Primitivi riusati, nessuna duplicazione

`pill`, `panelHead`, `iconButton`, popover nativo (come il chip memoria),
`openMenuAt`, `sparkPath` (lo stesso grafico dei sensori), `memoryStatus`,
`sensorChip`/`probeTotals`. Moduli puri nuovi e coperti al 100% delle righe:
`src/status-selection.js`, `src/screen-status.js` (`screenStatus`,
`activeProfile`), `createRenderHistory`/`bezelStatus` in `src/memory-status.js`.
Nuovo modulo d'interfaccia `src/ui/screen-status.js`. Icone `performance`,
`pencil`, `profile`. Chiavi riusate dove il testo esiste già
(`framing.degrees`, `layers.count`).

## Test eseguiti

- Unitari UI: 668/669; l'unico fallimento (`video-framing.test.mjs`) è uguale
  sulla base.
- E2E: nuova `shell-status.spec` (12 verdi); spec dell'area insieme 207 verdi.
  I casi rossi sono gli stessi della base (`scenarios-shell`, `scenarios-screen`,
  `live-screen-controls`, `audio-spectrum`, `memory-status:7` su `storage-wide`);
  `library:54` è stato un timeout di caricamento, verde da solo.
- Confronto G3: `evidence/t9b/` (modifiche non salvate, trascinamento, i tre
  riquadri) in italiano chiaro e scuro, contro MainNew e StatusBarNew.

## Spec adeguate e perché

`memory-status`, `libre`, `shell`, `scenarios-shell` (chip e regioni della
barra cambiati come da D-11); unitari `libre-status`, `sensor-probes`,
`memory-status`, e il controllo `i18n` del marchio (in alto c'è solo il logo).
Nessuna asserzione indebolita: le verifiche seguono i nuovi controlli.

## Differenze rimaste rispetto alle tavole

- Il pulsante Preferenze resta anche accanto a «In diretta» (oltre che nel
  menu ⋯): tre spec di altre aree lo usano; da decidere in T-10 se toglierlo.
- «fotogrammi 30 fps di 30»: il massimo dello schermo non è noto all'interfaccia,
  si mostra solo il valore misurato.
- Il giallo del chip Schermo per pochi fps non c'è: senza il massimo non si può
  dire quando sono pochi; giallo solo per problemi veri.
- «sensori valori demo» della tavola non è usato: la demo mostra le stesse
  regole dell'app vera.

## Da annotare per T-10

- `index.html` e `app.js` sono stati modificati qui come da piano; T-9c non li
  tocca e annoterà i suoi collegamenti.
- Il pulsante «Valori di prova» di T-8 è ancora creato da `ui/trial-values.js`:
  ora si può spostare nel markup.
- Fallimenti preesistenti invariati: vedere `SUMMARY-T-9a.md` e i log in
  `target/logs/t9b/` e `F:/dev/bezel/target/logs/t8-baseline/`.
