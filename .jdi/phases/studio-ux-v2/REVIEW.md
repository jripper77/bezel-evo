# Studio UX v2 — verifica della fase (T-10)

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
| Unitari UI (`node --test`) | <<UNIT>> |
| Copertura UI (`npm run test:unit`, soglia 80% righe) | <<UICOV>> |
| E2E, quattro progetti | <<E2E>> |
| `cargo fmt --check` | <<FMT>> |
| `cargo clippy --workspace --all-targets -D warnings` | <<CLIPPY>> |
| `cargo test --workspace` | <<TEST>> |
| Copertura Rust (`cargo llvm-cov`, ≥ 80%) | <<COV>> |
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
