# Materiale di design — fase studio-ux-v2

Copia offline della pagina «Attuale» del canvas https://claude.ai/artifact/7muP7mTQAwBH8diy2AWZ6p (versione 89,
20 ottobre 2026). Basta questa cartella: il canvas non serve per lavorare.

## Come si usa
- `png/<Tavola>.png` — immagine della tavola (a grandezza naturale).
- `boards/<Tavola>.html` — la stessa tavola in HTML statico: aprirla nel browser per leggere misure, colori, testi e
  struttura (gli stili sono in linea; i font arrivano da Google Fonts, gli stessi inclusi in Studio). I valori mostrati
  (CPU 38%, Roma, Steam…) sono esempi: in Studio vengono dai dati veri.
- `boards/index.json` — titolo e dimensioni di ogni tavola.
- `svg/icons/*.svg` — icone 24×24, tratto 1,75, estremità arrotondate, colore `currentColor`. Il nome del file è il nome
  da usare in `src/ui/icons.js`. Prefissi: `type-` riga Tipo, `widget-` pannello Widget, `layer-` pannello Livelli,
  `rule-` tipi di regola dei profili, `fade-` / `transition-` / `corners-` scelte dell'inspector.
- `svg/weather/<n>-<famiglia>-<stile>.svg` — 8 famiglie × 4 stili, animate (SMIL) come riferimento dei movimenti;
  specifica numerica in `weather-motion.md`.
- `svg/misc/logo-pixel.svg` — logo Pixel; `handle-*.svg` — i quattro simboli delle maniglie (D-10).
- Motivi puntinati per le forme: `img/patterns/` nella radice del repository (esportati il 20 ottobre).

Colori e misure: i colori delle tavole corrispondono ai token di `apps/bezel-studio/src/styles/tokens.css` (tema scuro;
accento #FF9248). Un colore della tavola che non ha un token → usare il token più vicino e annotarlo nel SUMMARY.

## Tavole → task
| Tavola | Titolo | Task |
|---|---|---|
| MainNew | Editor — nuovo stile (anello selezionato) | T-0 (pannello destro), T-8 (canvas), T-9a (pannello sinistro), T-9b (barre) |
| StatusBarNew | Barra di stato di Studio — nuovo schema | T-9b |
| Connect | Primo avvio — nessuno schermo | invariata (riferimento di stile) |
| LibraryPanelsNew | Sensori, Livelli e Media — nuovo schema | T-9a |
| ThemesNew | Galleria temi — nuovo schema | T-9c |
| ProfilesNew | Profili e automazioni — nuovo schema | T-6, T-9c |
| Logo | Logo e icona app | riferimento |
| CanvasHandles | Maniglie sul canvas — per ogni oggetto | T-8 |
| DeviceBoot | Schermata di caricamento 1920×480 | riferimento (sezione Schermo esistente) |
| ThemePixel, ThemeNebula, ThemePaper, ThemeNowPlaying | Temi predefiniti 1920×480 | T-9d |
| RingInspectorOpen | Anello — tutte le sezioni aperte | T-0 (modello), T-7a |
| PlayerInspectorOpen | Media player — tutte le sezioni aperte | T-7b |
| CardInspectorOpen | Card — tutte le sezioni aperte | T-7c |
| GraphInspectorOpen | Grafico — tutte le sezioni aperte | T-7a |
| ThemeInspectorOpen | Tema (nessuna selezione) — aperto e chiuso | T-7c |
| ScreenPanelsOpen | Schermo — Impostazioni e Archivio | T-7c |
| InspectorComponents | Componenti dell'inspector — specifiche | T-0 |
| WeatherAnimations | Meteo — icone animate | T-4, T-7b |
| ClockInspectorOpen | Orologio — digitale e analogico | T-1, T-2, T-7b |
| WeatherInspectorOpen | Meteo — tutte le sezioni aperte | T-4, T-7b |
| NeedleInspectorOpen | Lancetta — tutte le sezioni aperte | T-3, T-7a |
| BarInspectorOpen | Barra — tutte le sezioni aperte | T-7a |
| SpectrumInspectorOpen | Spettro audio — tutte le sezioni aperte | T-7b |
| ShapeInspectorOpen | Forma — tutte le sezioni aperte | T-7b |
| ValueInspectorOpen | Valore e testo fisso | T-1, T-7a |
| ImageInspectorOpen | Immagine, GIF e video | T-5, T-7b |
| MultiInspectorOpen | Più oggetti selezionati | T-7a |

Le tavole «tutte le sezioni aperte» mostrano ogni sezione aperta per far vedere tutto; in Studio le sezioni si aprono e si
chiudono (D-01) e da chiuse mostrano il riassunto come nel pannello destro di MainNew.
