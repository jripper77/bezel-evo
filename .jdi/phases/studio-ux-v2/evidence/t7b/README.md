# Confronto visivo T-7b

21 ottobre 2026. Inspector da 360 px, lingua italiana, sezioni aperte, temi chiaro
e scuro. Il file `*-detail.png` mostra tutto il pannello, quello senza suffisso
il contesto dell'app. Riferimenti: `../../design/png/` e HTML offline delle tavole.

| Stato | Scuro | Chiaro |
| --- | --- | --- |
| Orologio digitale | [cattura](dark/app/inspector/ux2-media-clock-detail.png) | [cattura](light/app/inspector/ux2-media-clock-detail.png) |
| Orologio analogico | [cattura](dark/app/inspector/ux2-media-analog-detail.png) | [cattura](light/app/inspector/ux2-media-analog-detail.png) |
| Meteo | [cattura](dark/app/inspector/ux2-media-weather-detail.png) | [cattura](light/app/inspector/ux2-media-weather-detail.png) |
| Player | [cattura](dark/app/inspector/ux2-media-player-detail.png) | [cattura](light/app/inspector/ux2-media-player-detail.png) |
| Spettro | [cattura](dark/app/inspector/ux2-media-spectrum-detail.png) | [cattura](light/app/inspector/ux2-media-spectrum-detail.png) |
| Forma con immagine | [cattura](dark/app/inspector/ux2-media-shape-detail.png) | [cattura](light/app/inspector/ux2-media-shape-detail.png) |
| Immagine SVG | [cattura](dark/app/inspector/ux2-media-image-detail.png) | [cattura](light/app/inspector/ux2-media-image-detail.png) |
| GIF | [cattura](dark/app/inspector/ux2-media-gif-detail.png) | [cattura](light/app/inspector/ux2-media-gif-detail.png) |

I dati e i nomi provengono dai documenti demo e dal catalogo: per Forma/Immagine
si importa l'icona CPU offline, per la GIF si usa lo scenario GIF già esistente.
Nessun brano, lettura o nome d'esempio delle tavole è stato aggiunto al prodotto.

Differenze: i preset, le facce effettive dei font, i controlli di importazione e le
note precedenti rimangono disponibili; aumentano l'altezza. Il Player conserva
le opzioni di segmenti/estremità/pallino. La ricerca meteo conserva il pulsante
esplicito e le risposte reali. Il cambio meteo mantiene la dissolvenza di 300 ms
di D-09. I video nell'oggetto Immagine e l'audio restano T-11 (D-16): riproduzione
solo per GIF. Le anteprime di asset personalizzati nel demo mantengono i
segnaposto precedenti; i nuovi ritagli/sfumature sono verificati anche su canvas
con PNG decodificato e nei pixel del renderer nativo con PNG/GIF reali.

Il browser interattivo non risultava disponibile (inventario vuoto): consultati
il sorgente HTML locale e le PNG; le catture dell'app vengono dalla ricetta di
fedeltà prevista dal piano. La verifica sul prodotto Windows installato e sui
due dispositivi resta a T-10 e alla prova manuale dell'utente.
