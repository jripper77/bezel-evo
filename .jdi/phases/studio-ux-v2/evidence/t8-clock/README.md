# T-8 — orologio analogico e distanze delle guide (confronto parziale)

21 ottobre 2026. Italiano, chiaro/scuro. Ricette `shell/ux2-clock-length` e
`shell/ux2-alignment-distance`; otto PNG (intero e dettaglio per stato/tema),
percorsi relativi nel manifest. Confrontati i gesti con CanvasHandles e MainNew:
rombi sulle punte, valore al cursore, campo evidenziato visibile; guide rosa
con distanza reale fra i riquadri.

Da `apps/bezel-studio`:

```text
node scripts/fidelity/shoot.mjs --area shell --only app --state ux2-clock-length,ux2-alignment-distance --out ../../.jdi/phases/studio-ux-v2/evidence/t8-clock
```

La lunghezza è relativa alla lunghezza originale di ciascuna lancetta: 100%
conserva i pixel precedenti e non si salva nel tema. Le ore, i minuti e i
secondi restano all’ora reale, con la politica del moto ridotto già esistente.
L’inspector scorre al campo illuminato senza prendergli il focus. Le guide
misurano il vicino separato più prossimo che condivide l’allineamento;
sovrapposizioni e allineamenti al solo canvas non producono distanze inventate.

Le immagini dimostrano la disposizione e l’interazione; la conservazione dei
pixel è verificata separatamente nei test Rust/browser (tre stili originali,
nuove lunghezze, invariabilità delle altre lancette e della coda dei secondi).
Primitivi e geometria riusati: numberField, maniglie comuni, comandi di Undo,
clock-format/handAngles, token --guide. Nessuna duplicazione di componenti.

Differenze ancora aperte: barra mobile/prove e gli altri widget elencati nel
CHECKPOINT-T-8.md. Questo confronto non chiude G3 né autorizza il merge di T-8.
