# T-8 — confronto parziale: selezione, Testo, Card e immagine nelle Forme

21 ottobre 2026. Demo reale italiana, chiaro/scuro. Ricette in
`apps/bezel-studio/scripts/fidelity/recipes/shell.mjs`:
`ux2-handles-selection`, `ux2-text-inline`, `ux2-card-entry`,
`ux2-shape-image-entry`. Per ogni stato: schermata completa e dettaglio della
workspace (16 PNG); `manifest.json` conserva percorsi relativi.

Riproduzione da `apps/bezel-studio`:

```text
node scripts/fidelity/shoot.mjs --area shell --only app --state ux2-handles-selection,ux2-text-inline,ux2-card-entry,ux2-shape-image-entry --out ../../.jdi/phases/studio-ux-v2/evidence/t8-selection-text
```

Riferimenti offline: CanvasHandles e MainNew, PNG/HTML letti nel checkpoint
iniziale. Ispezionate le schermate generate: riquadro comune, valore vicino
alla maniglia e campo evidenziato; editor multilinea con il testo corrente;
confine della Card con le facce e uscita; modifica dell’immagine con offset
illuminato nell’inspector. I dati vengono dalla demo e dalle aggiunte effettuate
con i controlli dell’app, non da nomi o letture delle tavole nel prodotto.

Differenze motivate:

- Nessuna Rotazione per Card, gruppi, membri o selezioni che li comprendono:
  D-21 prevale sul disegno errato delle tavole.
- Le linguette mostrano i veri nomi delle facce, anziché i numeri di esempio;
  riusano le scelte native comuni. Aggiunto un pulsante di uscita per rendere
  accessibile la stessa azione di Esc.
- L’editor del Testo è un textarea nativo: supporta più righe, selezione e
  annullamento senza una seconda implementazione dei controlli di testo.
- L’immagine della Forma nella demo usa il segnaposto già esistente nel
  demo-render. L’asset scelto è il vero SVG importato dall’app; questo passo
  non modifica renderer o decodifica. Non è una prova di parità pixel nativa.
- Manca ancora la barra mobile richiesta e il confronto di tutti i widget:
  queste immagini documentano solo il passo completato, non l’accettazione
  finale di T-8. Elenco residuo in `CHECKPOINT-T-8.md`.

Primitivi riusati: segmented, iconButton, geometria/rotazione, comandi del
modello, burst della rotella. Nessuna duplicazione di componenti.
