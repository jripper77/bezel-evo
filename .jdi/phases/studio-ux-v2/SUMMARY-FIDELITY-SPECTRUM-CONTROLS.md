# Spettro: opzioni leggibili e Angoli compatti

22 ottobre 2026. Branch `fid/spectrum-controls`, worktree
`F:/dev/bezel-spectrum-controls`, base `0176809b` di `windows-improvements`.

## Fatto

- `2bda767e`: Linea centrale e Tieni il picco riusano `toggleField` con
  disposizione iniziale, label a 8 px dai pulsanti e icone distinte della
  tavola: linea orizzontale, barre con segni di picco. Le label separate
  restano, come accettato dall’utente. Lo spazio di 8 px appartiene al
  primitivo condiviso e vale anche per gli altri toggle iniziali.
- `12567cd8`: aggiornato `cornerRadii`/`cornersField` comune, dopo conferma
  esplicita dell’utente (D-2026-10-22-studio-ux-v2-22). Raggio uguale in
  campo compatto con icona, un solo toggle Angoli singoli. Il disegno
  proporzionato resta nella modalità singola, con quattro campi, ordine
  di tabulazione e illuminazione dell’angolo in modifica. I tipi senza
  supporto ai singoli angoli mantengono il solo raggio.
- Nessuna variante locale, campo del tema, stringa, geometria di rendering
  o nuova interpretazione dei raggi. Riutilizzati `prefixedField`,
  `iconButton`, `toggleField`, `fieldGrid` e il catalogo SVG comune.
- Aggiornati guida utente e catalogo dei componenti.

## Verifiche

- 71 unitari mirati verdi: `corner-radii`, `spectrum`, `static-guards`,
  `i18n`. Dopo gli ultimi ritocchi ai simboli, 16 guardie statiche verdi.
- 32 e2e Spettro/Grafico verdi prima della modifica Angoli.
- 76 e2e verdi dopo la modifica condivisa: `corner-radii`, `audio-spectrum`,
  `editing-017`, `inspector-layout`, `inspector-components`, `inspector-plots`,
  quattro progetti chiaro/scuro, en/pt-BR.
- 28 e2e `inspector-media` verdi: copertina, Forma, immagini/GIF e controlli
  Spettro, compresi i controlli sui pixel già esistenti.
- 4 casi Angoli Spettro rieseguiti dopo nuove verifiche di tastiera e Undo:
  verdi. La prima versione del nuovo test presumeva erroneamente che Undo
  ripristinasse la modalità di modifica dello Spettro: questa è memoria UI
  esistente, mentre il documento conserva sempre quattro raggi. Il test
  ora verifica l’Undo del raggio e del singolo angolo e la conservazione
  della modalità UI; nessun comportamento del prodotto cambiato per il test.
- Le verifiche geometriche del vecchio disegno nella modalità uguale sono
  trasferite alla modalità singola: proporzioni, raggio minimo visibile,
  confronto tra raggi diversi, angoli illuminati e copertina quadrata
  restano verificati. Il limite assoluto legato al vecchio disegno da 40 px
  è sostituito dal confronto tra raggio più piccolo e più grande nel disegno
  da 72 px. Nuove verifiche: assenza del disegno nella modalità uguale,
  un campo con icona, un solo pulsante, label vicine e non spezzate a 320 px.
- `editing-017` migra soltanto il ritorno alla modalità uguale sul nuovo
  toggle; conserva tutte le verifiche di modifica e Undo.
- Catture italiane chiaro/scuro aggiornate per i controlli interessati:
  **38 stati ricatturati** (11 inspector, 6 shell, Impostazioni schermo
  e Livelli, ciascuno chiaro/scuro). Spettro esaminato nei due temi.
  Galleria rigenerata con `--reuse`: 108 stati, 208 PNG validi, collegamenti
  verificati e hash SHA-256 aggiornati. Nessuna cattura di altre aree
  sostituita senza necessità.

Nessuna build, installazione, suite completa o pubblicazione. Verifica
manuale ancora aperta; resta il limite delle immagini nel canvas demo.
