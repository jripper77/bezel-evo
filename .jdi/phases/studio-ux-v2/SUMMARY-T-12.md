# T-12 — Ordine nella Disposizione

22 ottobre 2026, ChatGPT. Branch `ux2/t12`, worktree `F:/dev/bezel-ux2-t12`, base `fb6b42f`.

## Cosa è fatto

- Tolta la sezione autonoma «Ordine dei livelli». La riga **Ordine** segue **Fa parte di** nella Disposizione di ogni oggetto, Card, gruppo e selezione multipla. Quattro comandi nello stesso riquadro: Davanti, avanti di uno, indietro di uno, Dietro. I passi singoli hanno un nome accessibile anche quando mostrano solo l'icona.
- Il dato «livello N di M» riusa `layerFacts` e la chiave `insp.layer`; non compare per la multipla. Le etichette corte visibili conservano nomi accessibili espliciti.
- Nuovo comando puro `orderLayers`, usato dallo store per `stepSelection` e `orderSelection`. Un passo multiplo e uno spostamento agli estremi sono un solo Undo; ordine relativo conservato; Card e gruppi viaggiano con i discendenti. I membri restano nel proprio gruppo o nella propria faccia. Le operazioni ai limiti o sugli oggetti/contenitori bloccati sono disabilitate; i no-op non aggiungono storia.
- `layerOrderAvailability` applica le stesse regole allo store, alla riga e al menu. Duplica ed Elimina sono usciti dal pannello: restano nel menu oggetto già condiviso da testata, barra mobile e clic destro. I due passi singoli sono nel gruppo piatto dell'ordine, fra separatori. Nessuna scorciatoia nuova.
- Allinea riusa la stessa riga di azioni, a sei icone. «Adatta al contenuto» resta nel menu condiviso e la guida ora lo indica correttamente.
- Fondazione visiva corretta durante il follow-up: header su due righe con anteprima 72 px contrastante anche in chiaro, Tipo compatto, scelte neutre con icone delle transizioni, Durata/Media raggruppate con icona dentro il numero, campioni dei riempimenti e didascalia Riempimento, esadecimali visivamente maiuscoli. Le scelte lunghe possono andare a capo; l'aiuto dell'Inerzia è nel tooltip e nella descrizione accessibile, senza paragrafo aggiuntivo.
- Nessun campo del tema, renderer, bridge o protocollo modificato. Le scelte di riempimento mantengono i valori già presenti nel documento.

## Primitivi riusati

`iconChoice`, `objectHeader`, `numberField`, `colorPicker`/`swatchChip`, `propertySection`, `fieldGrid`, `sizeField`, menu `objectMenuItems` e `openMenuAt`, `layerFacts`, funzioni di proprietà e discendenza esistenti. Le estensioni facoltative sono documentate in `INSPECTOR-COMPONENTS.md`. Nessuna seconda implementazione di toolbar, menu, geometria o renderer.

## Commit

- `ac0be45`: confronto iniziale, piano e checkpoint della pausa.
- `0a94011`: comandi atomici dell'ordine e unitari.
- `6efc7bf`: componenti e stili condivisi dell'inspector.
- `d36310f`: Ordine nella Disposizione, menu condiviso, spec e guida.
- `13361fc`: riscontro visivo dello stato disabilitato.
- `ee50b37`: grafia localizzata e simboli nei documenti, preservati in UTF-8.
- Commit di consegna dei documenti/catture: identificabile nel log del branch.

Ogni commit ha il trailer ChatGPT. Nessun push, build, installazione o nuova versione.

## Test con esito

| Verifica | Esito |
|---|---|
| Unitari UI con copertura (`npm run test:unit`) | **686/686 verdi**, 99,23% righe; `target/logs/t12/ui-unit.log` |
| Unitari mirati dello store, layers, layer-order, inspector-helpers, i18n e guardie | **89/89 verdi**; `unit-resume.log` |
| i18n e guardie statiche dopo le ultime modifiche dei componenti | **51/51 verdi**; `guards-delivery.log` |
| E2e area, 24 spec, quattro progetti chiaro/scuro en/pt | Prima passata **364/372 verdi**, otto rossi su due spec che cercavano la vecchia etichetta e il vecchio paragrafo; aggiornate come descritto sotto |
| E2e dei cinque file interessati dagli ultimi aggiustamenti | **52/56 verdi**; quattro rossi nella nuova fixture che conteneva un gruppo vuoto, eliminato dal comportamento preesistente dello store al primo edit; corretta la fixture, mantenendo la verifica dei gruppi popolati |
| Nuova spec Ordine finale, tutti e quattro i progetti | **12/12 verdi**; `e2e-order-final.log`, inclusi gruppo completo, proprietario bloccato, membri Card, selezione multipla e tutti i tipi di widget |

Le **372 esecuzioni distinte** dell'area sono quindi coperte da esiti verdi tra passata iniziale e rilanci pertinenti. Nessuna suite completa Rust/prodotto, nessun rilancio di tutta l'area per un solo errore.

Migrazioni delle spec senza indebolire le asserzioni:

- `openInspectorMenu` in `helpers.mjs` sostituisce il pulsante Duplica del pannello; tolta l'apertura ridondante nella spec Rotazione.
- Appartenenza segue l'etichetta **Fa parte di**. Il cambio di Card e Undo conservano le verifiche precedenti.
- La transizione immediata si chiama **Salta**, come nella tavola. L'aiuto dell'Inerzia viene verificato sia come `title` sia come `aria-description`; modifica del tempo, selezione della modalità e Undo restano verificati.
- Nessuna spec di altre aree modificata; `value-inertia` è interessata direttamente dal componente Comportamento cambiato. Card, layer, clipboard e scenari esistenti sono passati senza cambiarne le asserzioni.

## Confronto con le tavole e differenze residue

**Galleria rigenerata e verificata:** `evidence/final/index.html`, con 96 catture di stato chiaro/scuro. Dopo il riscontro visivo dei pulsanti disabilitati sono state aggiornate di nuovo le 40 catture dell'inspector e rigenerato l'indice con `ux2-gallery.mjs --reuse`. Verificati in particolare Ordine su Anello/Card/Multipla, Tipo, Direzione, Comportamento e il contrasto dell'anteprima Orologio in chiaro. Log `gallery-final.log`, `gallery-inspector-final.log`, `gallery-index-final.log`. Immagini prima/dopo in `evidence/t12/`; le differenze generali del registro restano esplicite.

La correzione Ordine elimina la differenza richiesta. Rotazione resta sulla riga separata già approvata e non appare per Card, gruppi o membri; nella multipla compare solo se tutti possono ruotare. Duplica/Elimina non si ripristinano in fondo alla tavola Multi: prevale la nuova richiesta esplicita T-12.

Il campione decorativo della scelta Sfumatura, prima di avere due colori, usa il token esistente `--rule-media` come secondo estremo: è il colore disponibile più vicino al viola del simbolo; non imposta quel colore nel tema. Quando il documento ha due estremi, il campione li usa.

Il riallineamento generale **non è concluso**: editor della sfumatura ancora diverso dalla barra della tavola, traccia/opacità e collegamenti da ricomporre, editor delle parti del testo, controlli specifici, shell, Profili e Galleria. Sono riscontri espliciti di `AUDIT-FIDELITY-2026-10-22.md`, non eccezioni approvate né funzioni nuove da inventare. Le ricette Valore/Testo e Orologio ora rispettano i pannelli da 380 px; gli altri casi R00 restano nel piano.

## Verifica finale della fase

La fase resta in attesa dell'approvazione manuale. La galleria aggiornata permette di valutare T-12; non certifica da sola l'intera fedeltà né le prove sui display. Le attività del piano di follow-up devono chiudere il registro prima dell'approvazione definitiva. Nessuna nuova prova hardware dichiarata.
