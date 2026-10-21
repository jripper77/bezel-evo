# Confronto visivo T-7a

21 ottobre 2026. Catture dell'app in italiano, inspector largo 360 px, sezioni aperte,
nei due temi. Ogni `*-detail.png` mostra tutto il pannello; il file senza `-detail`
mostra il contesto dell'app. Le tavole di riferimento sono in `../../design/png/`;
le misure e i controlli sono stati confrontati anche con il sorgente HTML offline.

| Inspector | Scuro | Chiaro |
| --- | --- | --- |
| Valore | [cattura](dark/app/inspector/ux2-linked-value-detail.png) | [cattura](light/app/inspector/ux2-linked-value-detail.png) |
| Testo fisso | [cattura](dark/app/inspector/ux2-linked-text-detail.png) | [cattura](light/app/inspector/ux2-linked-text-detail.png) |
| Anello | [cattura](dark/app/inspector/ux2-ring-open-detail.png) | [cattura](light/app/inspector/ux2-ring-open-detail.png) |
| Barra | [cattura](dark/app/inspector/ux2-linked-bar-detail.png) | [cattura](light/app/inspector/ux2-linked-bar-detail.png) |
| Lancetta | [cattura](dark/app/inspector/ux2-linked-needle-detail.png) | [cattura](light/app/inspector/ux2-linked-needle-detail.png) |
| Grafico | [cattura](dark/app/inspector/ux2-linked-graph-detail.png) | [cattura](light/app/inspector/ux2-linked-graph-detail.png) |
| Selezione multipla | [cattura](dark/app/inspector/ux2-linked-multi-detail.png) | [cattura](light/app/inspector/ux2-linked-multi-detail.png) |

Differenze deliberate: nomi, sensori, letture, font e geometrie sono quelli del
modello e del catalogo demo. La serie inferiore del Grafico rimane vuota finché
non si sceglie una sorgente reale; non si duplica la serie superiore. La Lancetta
mostra il valore centrale, una zona e il massimo attivati tramite i controlli;
il Grafico mostra due serie, barre arrotondate e larghezza esplicita di 3 px.

I pannelli mantengono le funzioni precedenti non rappresentate dalle tavole:
modalità e controlli dei preset, precisione della finestra storica, note sui dati,
allineamento, duplica/elimina, opzioni importate e configurazione della rotazione
nel pulsante di collegamento. Questo aumenta l'altezza rispetto alle tavole.
I font con più di sei facce mantengono il selettore nativo completo; quelli più
piccoli usano i campioni Aa. Non sono state copiate letture d'esempio delle tavole.

La verifica complessiva dello shell, dei pannelli delle altre task e sul renderer
Windows installato resta a T-10; queste sono catture del codice sorgente in demo.
