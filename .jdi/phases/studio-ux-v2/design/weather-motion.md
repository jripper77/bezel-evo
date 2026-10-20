# Icone meteo animate — specifica dei movimenti (D-2026-10-20-studio-ux-v2-09)

Riferimento visivo: `svg/weather/<n>-<famiglia>-<stile>.svg` (SMIL, si aprono animati nel browser) e la tavola
`WeatherAnimations`. Il renderer **non** usa SMIL: calcola la posa in funzione del tempo con una funzione pura.

## Convenzioni
- Coordinate nel riquadro 64×64 dell'icona; si scala in proporzione alla dimensione dell'icona.
- `A` = ampiezza: **Vivace** = 1, **Leggera** = 0,5 (moltiplica gli spostamenti e la variazione di opacità attorno al valore medio).
  **Ferma** = posa a t = 0, nessun ridisegno.
- Velocità `v` (0,5–2): ogni periodo si divide per `v`.
- Curva «morbida» = cubic-bezier(0.45, 0, 0.55, 1) tra i punti chiave; «lineare» = interpolazione lineare.
- Ogni movimento è periodico e riparte senza scatti. Movimento ridotto del sistema → Ferma.
- Cambio di famiglia: dissolvenza incrociata di 300 ms tra la vecchia e la nuova icona.
- Lo stesso movimento vale per i quattro stili (Contorno, Piena, A colori, 3D). Il 3D aggiunge: ombra ellittica a terra
  (centro 33,58, raggi 16×2,6) che segue lo spostamento orizzontale del soggetto principale; per il temporale un lampo sul
  riquadro (opacità 0,12 negli stessi istanti del fulmine); per il sereno un alone (raggio +4) con opacità 0,10→0,30→0,10 in 4 s.

## Famiglie (`weather::family`)
| # | Famiglia | Parti e movimento | Periodo |
|---|---|---|---|
| 0 | Sereno | raggi (8, ogni 45°) ruotano attorno al centro, lineare, 360°; disco: raggio r → r+1,2·A → r, morbida | raggi 24 s, disco 4 s |
| 1 | Parzialmente nuvoloso | sole piccolo (r 7, centro 40,20) come il sereno; nuvola: x −3 → +1 → −3 (·A), y fissa +2, morbida | nuvola 7 s |
| 2 | Nuvoloso | nuvola dietro (scala 0,72, spostata 9,−9, colore scuro): x +4 → −2 → +4; nuvola davanti: x −2 → +3 → −2; morbida | 9 s e 7 s |
| 3 | Nebbia | 4 strisce (y 22, 31, 40, 49): x −4 → +4 → −4 e opacità 0,35 → 1 → 0,35, morbida; striscia i sfasata di 0,6·i s | 5+i s |
| 4 | Pioggia | nuvola ferma (y −4); 3 gocce (x 22, 31, 40): traslazione (0,−4) → (−2,+8) lineare, opacità 0 → 1 → 0; sfasate 0 / 0,33 / 0,66 s | 1 s |
| 5 | Neve | nuvola ferma (y −4); 3 fiocchi (x 22, 32, 42): (0,−6) → (2,2) → (−1,10), opacità 0 → 1 → 0; sfasati 0 / 1 / 2 s | 3 s |
| 6 | Temporale | nuvola scura ferma; fulmine: opacità per tempi relativi 0 → 0,55: 0; 0,60: 1; 0,65: 0,2; 0,70: 1; 0,80: 0; → 1: 0 (due lampi) | 3 s |
| 7 | Non disponibile | nuvola (tratteggiata nello stile Contorno) con opacità 0,45 → 0,85 → 0,45, morbida; «?» fermo | 4 s |

## Prestazioni (G11)
- `next_ms` = 33 ms con movimento attivo e oggetto visibile; con Leggera è accettabile 50 ms.
- Nessun ricalcolo dei tracciati: le forme si preparano una volta per stile e dimensione, a ogni fotogramma cambiano solo
  trasformazioni e opacità.
