# T-7b — Orologio, Meteo, Player, Spettro, Forma e Immagine

Completata il 21 ottobre 2026 in `F:\Dev\bezel-ux2-t7b`, branch `ux2/t7b`, da
`a2cc866` (T-7a integrata). Nessun push, build del prodotto, cambio versione,
setup o installazione. `dist/` e l'installazione dell'utente sono invariati.

## Cosa è fatto

- Orologio digitale: formato e codici inseribili, lingua, parti ore/minuti/secondi/
  separatori con l'editor condiviso del Valore, ereditarietà, cifre fisse e lampeggio.
  Analogico: tacche, numeri, tre lancette indipendenti, forma, bordo, colori,
  data e secondi a scatti/fluidi. Lo stile analogico non mostra il pannello Testo;
  senza secondi non mostra il movimento della lancetta. Il formato digitale resta
  conservato cambiando stile.
- Meteo: ricerca/località vere, unità, lingua, quattro anteprime disegnate dalla
  funzione effettiva del demo, dimensione automatica o esplicita, distanza, testo
  e cifre fisse. Animazione Ferma/Leggera/Vivace e velocità; senza icona il relativo
  Comportamento si nasconde. D-09 mantiene il movimento ridotto e la dissolvenza.
- Player: app reali dal bridge, scelta quando è fermo in Dati, sei parti su due
  righe, allineamento in Forma, angoli e trasparenza della copertina, geometria
  della barra e suoi colori separati dal testo. Nessuna sezione Comportamento
  vuota: la configurazione di rotazione è nel collegamento della Disposizione.
- Spettro: sorgente e bande/frequenze in Dati, cinque modalità e geometria in
  Forma, riempimento in Colore e movimento in Comportamento; proprietà non
  pertinenti nascoste. Gradienti importati conservati modificando gli estremi.
- Forma: riempimento uniforme/sfumato condiviso, bordo e angoli, immagine dal
  catalogo, sette ripetizioni a icone; scala/spostamento solo dove applicabili,
  opacità dell'immagine distinta da quella dell'oggetto, trasparenza condivisa.
- Immagine: file e accesso al pannello Media, adattamento, nove ancoraggi per
  Riempi, angoli uniformi/singoli, trasparenza. Solo GIF: ripetizione/una volta e
  velocità (valori importati non standard conservati); video/audio restano T-11.
- Layout e componenti comuni estesi a questi tipi, conservando azioni e funzioni
  precedenti. Stringhe nell'area inspector con parità en/pt-BR/it.

## Estensioni approvate, D-20

Il piano indicava come preesistente la trasparenza della copertina, ma non lo era.
Corretto `CONTEXT.md`; aggiunti:

- `content.coverFade` facoltativo: la stessa rampa lineare/radiale degli altri
  oggetti, solo sulla copertina, con nessun effetto su testo e avanzamento.
- `content.iconColor` facoltativo: Contorno/Piena seguono il testo se assente;
  A colori/3D ignorano il campo e ne nascondono la riga, conservandone il valore.
- `kind.radius`/`kind.corners` per immagini e GIF, omessi a zero/assenza; i quattro
  angoli a zero vengono normalizzati all'assenza. Stesso ritaglio della copertina.

Andata e ritorno, omissione dei default e pixel precedenti verificati. Per mantenere
contenute le varianti del DTO, lo stile di `KindDto::Text` è ora boxed: il formato
JSON non cambia. Gli altri crate richiedono solo i nuovi default nei costruttori.

## Commit

- `725d0ee` — modello, DTO, renderer, compatibilità e correzione del contesto.
- `26c0267` — effetti e ritagli nel demo, helper puri per media e test.
- `4329376` — inspector e spec e2e dell'area.
- `04e483a` — etichetta generica del riempimento fuori dall'Anello.
- `1ccb661` — ricette e catture it nei due temi.

## Test con esito

Tutti verdi. Log raggruppati in `target/logs/t7b/` della worktree.

- `cargo test -p bezel-core -p bezel-render -p bezel-themes -p bezel-runtime -p bezel -p bezel-studio`:
  verde. Sono i soli crate toccati; CLI, runtime e test Studio hanno costruttori
  aggiornati ai default, oltre ai tre crate del modello/renderer.
- `cargo clippy -p bezel-core -p bezel-render -p bezel-themes -p bezel-runtime -p bezel -p bezel-studio --all-targets -- -D warnings`:
  verde. Dopo il boxing trasparente del DTO, ripetuti i test di bezel-themes: verdi.
- `cargo fmt --all -- --check` e `git diff --check`: verdi.
- **83 unitari UI** mirati: media-inspector, clock-format, weather-widget,
  weather-motion, player-parts, image-playback, shape-fade, spectrum,
  inspector-edit, inspector-model, i18n e static-guards. Ultime estensioni dei
  componenti: **53** verifiche mirate media/i18n/guard verdi.
- **76 e2e** verdi: inspector-media (28), inspector-linked (28),
  inspector-components (16), inspector-stepping (4), tutti nei quattro progetti
  en/pt-BR e chiaro/scuro. Ultimi ritocchi: 12 casi Orologio/Meteo/Player e 12
  casi di riempimento Forma/Spettro/Barra/Anello verdi. Nessuna spec di un'altra
  task modificata o asserzione indebolita.
- Accessibilità, tastiera, Annulla, visibilità condizionale e movimento ridotto
  inclusi nelle spec. Selezione GIF e combinazione di ancoraggio/angoli/velocità
  provate anche dal pannello. Ripetizione/opacità della Forma provate con SVG
  effettivamente importato dal catalogo offline.
- Pixel: vecchie immagini senza angoli confrontate byte per byte; PNG e GIF
  animata con angoli indipendenti, mantenendo i colori dei fotogrammi. Vecchio
  Player con gli hash già fissati; nuove sfumature lineari/radiali sulla copertina
  con tutti i pixel di testo/avanzamento invariati. Hash meteo registrati prima
  della modifica: Contorno **13710710557535332066**, Piena **14492384110744872271**,
  entrambi invariati in assenza di colore proprio; A colori/3D invariati anche
  con un override memorizzato. Prove corrispondenti del canvas demo, incluso un
  PNG decodificato, GIF in movimento e regione esterna alla copertina immutata.
- G3: **16 catture** italiane chiaro/scuro, otto stati, in
  [evidence/t7b](evidence/t7b/README.md), prodotte con `scripts/fidelity/shoot.mjs`.

Nessuna suite completa o copertura eseguita: restano alla verifica T-10.

## Differenze dalle tavole e lavoro per T-10

Nomi, font, dimensioni, app e asset sono quelli del documento/catologo; non i
valori d'esempio. Rimangono le opzioni precedenti non rappresentate nelle tavole,
le note del provider e l'importazione: i pannelli sono quindi più alti. Le facce
del font dipendono dal catalogo reale, con selettore completo oltre sei facce.
La ricerca meteo conserva il pulsante esplicito; il cambio condizione mantiene
la dissolvenza fissa di D-09. Le GIF non hanno un controllo audio; i video non
sono selezionabili come immagini riproducibili fino a T-11, come richiesto da D-16.

Il browser interattivo non era disponibile (inventario vuoto); consultati i
sorgenti HTML offline e le PNG, confrontandoli con le catture previste dal piano.
La demo mantiene il limite preesistente delle immagini segnaposto per gli asset
personalizzati: i nuovi ritagli e le nuove rampe sono condivisi e provati anche
su bitmap, ma la verifica delle immagini effettive va completata nativamente.

A T-10: confronto visivo complessivo e sul prodotto Windows, prova su schermi
reali, suite completa/copertura una volta, adeguamento delle vecchie spec che
cercano slider sostituiti da campi numerici (es. opacità immagine della Forma).

## Riuso

Nessuna duplicazione: stessi `partsRow`/editor parti, `fontPicker`, `colorPicker`,
`numberField`, `previewChoice`, `iconChoice`, `imageField`, `cornersField` e
`fadeField`. Estensioni in `INSPECTOR-COMPONENTS.md`. Maschera e sfumatura native
condivise, helper equivalenti del demo; nessun secondo decoder o worker media.
