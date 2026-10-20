# Fase 21: Studio UX v2 — Contesto  (slug: studio-ux-v2)

## Obiettivo
Bezel Studio deve diventare uguale, tavola per tavola, alle tavole della pagina «Attuale» del canvas di design
https://claude.ai/artifact/7muP7mTQAwBH8diy2AWZ6p (versione 89 del 20 ottobre 2026), e avere tutte le funzioni nuove che
quelle tavole mostrano. Il canvas è privato: **chi esegue lavora sulla copia offline in `design/`**, che basta da sola.

## Da dove nasce
Sessione di design del 20 ottobre 2026 con l'utente. Problema di partenza: «l'inspector è poco chiaro». Si è passati dalle
schede Dati/Aspetto/Movimento a sezioni apribili con riassunto, componenti unici, e si è esteso lo stesso linguaggio a tutto
Studio (pannelli, barra in alto, barra di stato, profili, galleria). Durante la sessione l'utente ha chiesto anche funzioni nuove.

## Decisioni bloccate
Tutte in `.jdi/decisions/` (testo completo lì):

| ID | In breve |
|---|---|
| D-2026-10-20-studio-ux-v2-01 | Inspector a sezioni apribili, ordine fisso, indipendenti, memoria per tipo, Alt+clic = solo questa, riassunto da chiusa |
| -02 | Intestazione oggetto: anteprima 72 px, nome, ▶ prova, ⋯, «dal vivo» allineato |
| -03 | Riga «Tipo» a icone; pulsanti-icona invece degli interruttori; ⚡ collega un campo a un sensore |
| -04 | Un'implementazione per componente (catalogo nella tavola InspectorComponents); niente note per lo sviluppatore nelle schermate |
| -05 | Cifre a larghezza fissa nella sezione Testo, attive di serie |
| -06 | Parti con stile proprio (Valore: prima/numero/unità/dopo; Orologio: ore/separatori/minuti/secondi), separatori lampeggianti |
| -07 | Orologio analogico |
| -08 | Lancetta estesa (zone, tacche, numeri, forma, cappuccio, valore al centro, segno del massimo) |
| -09 | Icone meteo animate, 8 famiglie × 4 stili |
| -10 | Maniglie per oggetto (quadrato/cerchio/punto/rombo) e barra mobile sull'oggetto |
| -11 | Barra di stato a tre chip; salvataggio nella barra in alto |
| -12 | Profili: nuove regole (Sensore, Orario, Inattività, Schermo bloccato), Basta una/Tutte, priorità per regola, profilo attivo + motivo, salvataggio automatico |
| -13 | Galleria temi |
| -14 | Pannelli Widget, Sensori, Livelli, Media |
| -15 | Fuori fase: tray di Windows nuova, pulsante «Stato» nella barra laterale |

Restano valide le decisioni delle fasi precedenti, in particolare: studio-redesign-fidelity D-4 (token di colore, accento
unico), D-5 (chiaro + scuro), D-6 (font locali, CSP), D-7 (i18n en/pt-BR/it, «Card» non si traduce), D-8 (SVG inline,
accessibilità), D-14 (riuso obbligatorio dei primitivi), -27 (solo Windows), -28 (fontPicker unico), -29 (colorPicker unico);
object-rotation 01–08 (rotazione, maniglia di rotazione).

## Cosa esiste già (non rifarlo)
- Colore per soglie: `crates/bezel-core/src/domain/value_color.rs`, `apps/bezel-studio/src/value-color.js`.
- Transizioni del valore Salta/Fluida/Inerzia: `crates/bezel-core/src/domain/transition.rs`, `sensor_motion.rs`, `src/value-transition.js`, `src/sensor-motion.js`.
- Trasparenza (fade) di immagini, forme e copertina: `FadeDto` in `crates/bezel-themes/src/dto.rs`, `src/shape-fade.js`.
- Immagine nelle forme con ripetizione: commit 0a17f1b, 4d713a3.
- Stili icona meteo Contorno/Piena/A colori/3D statici: `crates/bezel-render/src/weather.rs`.
- Anteprima isolata dell'oggetto: `render_element_preview` (`apps/bezel-studio/src-tauri/src/element_preview.rs`, `src/element-preview.js`).
- Regole profilo App in primo piano / Processo / Musica: `crates/bezel-runtime/src/profiles.rs` (`Source`, `Rule.priority`).
- Rotazione degli oggetti e maniglia: fase object-rotation.
- Maiuscole/minuscole: `text_case.rs`, `src/text-case.js`.

## Aperti noti che questa fase chiude
- Nomi tagliati nelle righe Sensori e nelle facce Card (D-14 pannelli, sezione Card).
- Stato «attivo ora» dei profili non esposto dal runtime (D-12).
- Temi predefiniti 1920×480 da realizzare (tavole ThemePixel/Nebula/Paper/NowPlaying).

## Fuori fase
Tray di Windows nuova (tavola TrayMenu, solo nel backup del canvas), protocolli dei dispositivi, firmware dell'8,8.

## Definition of Done (fase)
1. Ogni tavola elencata in `design/README.md` ha la sua controparte in Studio, confrontata con `scripts/fidelity/shoot.mjs`
   (scuro e chiaro, lingua it) e con i PNG di `design/png/`; ogni differenza rimasta è scritta nel SUMMARY della task con il motivo.
2. Le funzioni nuove funzionano con dati veri (bridge + demo `?demo=turing88` con lo stesso comportamento) e sono salvate nel
   file del tema in modo retrocompatibile (campi nuovi facoltativi, omessi al valore predefinito; i temi vecchi si aprono uguali).
3. Renderer: ogni funzione nuova ha test Rust (pixel o geometria) e la stessa resa nel demo-render JS.
4. i18n en/pt-BR/it a parità; accessibilità (`expectAccessible`) chiaro e scuro; nessun colore fuori da `tokens.css`.
5. Suite completa una volta alla verifica di fase (AGENTS.md): fmt, clippy, llvm-cov ≥ 80%, `npm test`.
6. Prova a vista sull'8,8″ e sul 3,5″ (manuale, dell'utente): meteo animato, lancetta, orologio analogico, cifre fisse.
