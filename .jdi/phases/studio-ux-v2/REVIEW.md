# Studio UX v2 — verifica della fase (T-10)

## Aggiornamento fedeltà — 22 ottobre, ChatGPT

**Punto 5: pronto per la verifica visiva dell’utente.** Galleria canonica aggiornata:
[evidence/final/index.html](evidence/final/index.html). Non usare le vecchie catture dei blocchi come stato attuale.

Completati i blocchi di riallineamento di inspector, shell/pannelli e dialoghi. Ultime correzioni in `c0797d9`, `321a295`, `761bc03`, ricette `576ca55`; riepilogo, test e registro delle 30 tavole in [SUMMARY-FIDELITY-FINAL.md](SUMMARY-FIDELITY-FINAL.md).
La galleria comprende **108 stati italiani** chiari/scuri, **208 PNG** validi e tutte le **30 tavole**. Collegamenti e integrità controllati; inventario in `evidence/final/verification.json`.

Correzione successiva alla prima verifica manuale: tolto il fondo quadrato dall’anteprima Anello (`b6ead42`), seguendo `RingInspectorOpen.html`. `MainNew` contiene invece un riquadro: per questa divergenza interna si privilegia la tavola specifica. Gli altri inspector mantengono i riquadri delle rispettive tavole. Verifica: 73 unitari/statici/i18n e 24 e2e anteprima verdi; galleria aggiornata, dettagli in `SUMMARY-FIDELITY-PREVIEW.md`.

Seconda correzione dalla verifica manuale: nello Spettro icone distinte per Linea centrale e Tieni il picco, con label accanto ai tasti (`2bda767e`). Angoli uguali ora compatto per tutti i chiamanti del controllo condiviso; disegno proporzionato conservato nella modalità singola (`12567cd8`, D-2026-10-22-studio-ux-v2-22). Riepilogo e test in `SUMMARY-FIDELITY-SPECTRUM-CONTROLS.md`.

Differenze visibili mantenute: disegno proporzionato degli Angoli singoli; Rotazione su riga propria, assente su Card/gruppi/membri; Duplica/Elimina nei menu; Salva secondario D-6; cinque effetti supportati dai profili; dissolvenza meteo D-09; dati dei quattro temi sostituiti come approvato; esportazione .bezeltheme. Controlli e avvisi funzionali esistenti conservati.

**Limite ancora aperto:** nel demo la scheda file ha miniatura/metadati reali, ma il renderer del canvas rappresenta le immagini con contorno/scacchiera. Non è una prova dei pixel nativi delle texture; vedi [attività da pianificare](../../todos/2026-10-22-demo-image-preview-parity.md). Nessun fulmine fittizio per legare Opacità, non esposto dal modello. Connect/DeviceBoot mantengono i limiti di ambito descritti nella galleria; nessuna nuova prova hardware.

I test mirati delle correzioni sono verdi (dettagli nei SUMMARY). I gate generali T-10 sotto sono **storici**, non una nuova esecuzione della suite completa su questa revisione. Nessuna build, installazione, pubblicazione o push. L’approvazione manuale e le prove fisiche restano dell’utente; la fase non viene chiusa automaticamente.

## Verifica T-10 precedente


21–22 ottobre 2026, Claude. Branch `ux2/t10`, worktree `F:/dev/bezel-ux2-t10`, base
`717451f` (Wave 2 integrata, T-11 rinviata). Nessun push, build, setup o installazione.

## Verdetto

**APPROVED_PENDING_MANUAL** — gate automatici: vedi «Gate»; restano la tua
approvazione della galleria di fedeltà e le prove a vista sugli schermi (DoD 1 e 6).

## Ambito

- T-0…T-9d completate e integrate; **T-11 (video negli oggetti Immagine) rinviata a
  tempo indeterminato** per tua decisione del 21 ottobre: fuori dalla chiusura della fase.
- Dati delle tavole senza sorgente (previsioni orarie, cache RAM, variazioni degli
  indici, avanzamento della giornata): fuori fase, in `.jdi/todos/2026-10-21-widget-data-extensions.md`.

## Cosa ha fatto T-10

### Pulizia
- `inspector-tabs.js` → `inspector-helpers.js`, senza la logica delle schede che nessuno
  usava più; tolto il re-export temporaneo `ui/inspector-card.js`.
- 42 chiavi i18n senza lettori tolte da en, pt-BR e it (1950 → 1908).
- Pulsante «Valori di prova» spostato nel markup della barra del canvas.
- Unitario `video-framing` rosso da tempo: il falso canvas del test non misurava il testo.

### Difetti veri trovati dalle spec e corretti
- **«Adatta al contenuto» invisibile**: stava nella testata nascosta della Posizione
  dentro Disposizione. Ora è accanto agli allineamenti.
- **Rotazione illeggibile**: divideva la riga con X e Y e il campo si riduceva a meno di
  1 px. Ora ha una riga sua con il ⚡; X/Y e rotazione mostrano quattro cifre (differenza
  voluta dalla tavola, confermata dall'utente).
- **Contagocce**: durante una scelta sul desktop il pulsante perdeva il focus e Esc non
  chiudeva più il selettore colore. Ora è `aria-disabled` e tiene il focus.
- **Archivio**: nel menu di un file le azioni disattivate non dicevano perché (la vecchia
  barra lo diceva). Ora il motivo è nel `title` della voce.
- **Portoghese**: le cinque modalità della transizione del Valore sbordavano di 10 px
  dall'inspector; le righe di scelte in una sezione ora vanno a capo.
- `iconChoice`: ogni pulsante porta il suo valore in `data-value` (serve ai test, nessun
  effetto visibile).

### Spec e2e portate sulla nuova interfaccia
La prima passata completa aveva **381 esecuzioni rosse (121 casi in 56 file)**: quasi
tutte cercavano l'interfaccia di prima di questa fase. Nessuna asserzione indebolita
nel senso di G4: ogni verifica segue il controllo nuovo che fa la stessa cosa.

- Sezioni chiuse all'avvio (D-01): le spec aprono la sezione come farebbe una persona
  (`openSection`), invece di trovare i campi sempre visibili sotto le schede.
- Schede Dati/Aspetto e Facce/Movimento/Attivazioni → sezioni; la Card ha un solo editor
  della transizione, in Comportamento (T-7c).
- Interruttori → pulsanti a due stati (D-03): nuovi aiuti `toggleButton`/`setToggle`.
- Menu a tendina diventati righe di scelte: aiuti `choose`/`chosen`.
- Cursori diventati campi numerici (durata, spessore, opacità, transizione dell'anello).
- Intestazione dell'oggetto senza `h2`; riquadro comune della selezione multipla; galleria
  dei temi con i più recenti per primi e filtri richiudibili (T-9c).
- Archivio: «Send file» con scelta del supporto e menu di ogni file; il gestore esteso si
  apre a richiesta (T-7c). Schermo: scheda con menu «Azioni dello schermo».
- Demo esteso in T-9d: conteggi dei sensori aggiornati (29 letture, 6 nel processore).

Casi in cui la verifica è cambiata di forma, da guardare:
- `device-video`: «Imposta come sfondo» non è più disattivato prima di scegliere un file,
  perché ora si sceglie dal menu del file stesso; si verifica che sia attivo per quel file.
- `card-animation`: la prova della transizione si guarda nell'anteprima dell'inspector
  (▶ Prova), non più sul canvas («Animate next face» non esiste più).
- `player-parts`: il rosso della barra si cerca sul canvas, perché l'anteprima
  nell'intestazione è una miniatura di 72×30 px in cui una barra di 4 px non si vede.
- `standby`: i «?» non ci sono più (scelta 3 sotto): la spec verifica la riga d'aiuto di ogni
  scelta e lo stato della scelta attiva letto dallo screen reader.

## Gate

| Gate | Esito |
|---|---|
| Unitari UI (`node --test`) | verde: 681/681 |
| Copertura UI (`npm run test:unit`, soglia 80% righe) | verde: 99,21% righe, 96,08% rami, 96,25% funzioni |
| E2E, quattro progetti | seconda passata 1087/1092; i 5 rossi (3 casi, tutti in pt) corretti e verdi da soli: un selettore che sbordava in portoghese (difetto vero, vedi sotto), decimali scritti col punto nel test, un `ERR_NO_BUFFER_SPACE` di rete di Windows |
| `cargo fmt --check` | verde |
| `cargo clippy --workspace --all-targets -D warnings` | verde |
| `cargo test --workspace` | verde: 1415 superati, 0 falliti, 19 ignorati |
| Copertura Rust (`cargo llvm-cov`, ≥ 80%) | verde: 90,25% righe, 89,94% regioni, 86,57% funzioni |
| i18n: parità e nessuna chiave orfana | verde (test i18n; 0 orfane) |

## Scelte dell'utente durante la verifica (22 ottobre)

1. **Preferenze solo nel menu ⋯**: tolta l'icona dalla barra in alto, come nella tavola
   MainNew. Nessuna scorciatoia da tastiera esisteva; chiudendo il dialogo il focus torna su ⋯.
2. **Rotazione su una riga sua** in Disposizione: **differenza voluta** rispetto alla tavola
   (X · Y · rotazione sulla stessa riga), perché a 300 px di pannello il numero non ci stava.
3. **Standby senza «?»**: ogni scelta ha sotto una riga d'aiuto grigia di una riga (o il
   motivo per cui non si può scegliere); il riquadro sotto le scelte spiega quella scelta per
   intero e lo screen reader la legge con il suo stato. Tolti il codice dei «?» nascosti, i
   testi lunghi e quattro chiavi che più nessuno leggeva. La riga «Piano B» ora va a capo
   invece di schiacciare l'etichetta. Guida (en, pt-BR) aggiornata.
4. **Niente temi 480×320 in questa fase**: annotato in
   `.jdi/todos/2026-10-22-temi-predefiniti-3-5.md` (da disegnare prima sul canvas).

## Prove a vista per te (DoD 6)

Sull'8,8″ (e sul 3,5″ dove indicato), con Studio e Light aggiornati:
1. Meteo animato: Leggera e Vivace, cambio di condizione con dissolvenza; fermo con
   «movimento ridotto» di Windows.
2. Lancetta estesa: zone colorate, tacche e numeri, valore al centro, segno del massimo
   che resta e poi ricade.
3. Orologio analogico: lancette all'ora giusta, secondi a scatti e fluidi, data.
4. Cifre fisse: un valore che cambia non deve «ballare»; Conta e Rullo.
5. Testo lungo che scorre o si rimpicciolisce; separatori dell'orologio che lampeggiano.
6. Profili: una regola Sensore o Orario che attiva un profilo; «Attiva per 30 s»; il chip
   Schermo che mostra il profilo attivo.
7. Galleria temi: ▶ prova di 30 s sullo schermo e ritorno al tema di prima.
8. I quattro temi 1920×480 (Pixel, Nebula, Paper, Now Playing) sull'8,8″ orizzontale.
9. Immagine con ancoraggio e GIF «una volta» / velocità.

## Galleria di fedeltà

`evidence/final/index.html`: ogni tavola accanto alle catture dell'app dopo T-10, scuro e
chiaro. Rigenerabile con `node scripts/fidelity/ux2-gallery.mjs`. Da approvare (DoD 1).

## Follow-up T-12 — Ordine e componenti condivisi

22 ottobre, ChatGPT, branch `ux2/t12`, base `fb6b42f`. Vedere `SUMMARY-T-12.md`.

**Differenza tolta:** il blocco separato «Ordine dei livelli» con Duplica/Elimina
è sostituito dalla riga **Ordine** nella Disposizione, dopo «Fa parte di», come
nelle tavole. I due passi singoli sono anche nel menu oggetto piatto condiviso;
selezione multipla con un solo Undo, ambiti e blocchi verificati. I comandi non
disponibili sono disabilitati anche visivamente. Il livello non appare nella multipla.

Riallineati inoltre header, Tipo compatto, Allinea, selezione neutra, icone e
misure delle transizioni e campioni del riempimento. I test funzionali dell'area
e gli unitari sono registrati nel riepilogo; nessuna nuova build o prova hardware.

La galleria viene aggiornata per T-12, conservando le immagini precedenti di
Anello/Card/Multipla in `evidence/t12/`. Il registro
`AUDIT-FIDELITY-2026-10-22.md` resta aperto per le altre differenze strutturali
(parti, gradiente, composizioni specifiche, shell, Profili e Galleria). Il verdetto
T-10 sopra descrive i suoi gate automatici: non attesta che queste differenze
siano già risolte. L'approvazione manuale della fase resta in sospeso.
