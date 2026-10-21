# T-8 — confronto parziale delle prime maniglie

21 ottobre 2026. Stato reale del demo, lingua italiana. Questo è un punto di
avanzamento: non sostituisce il confronto completo richiesto prima del merge T-8.
I dettagli mostrano il trascinamento dello spessore di un Anello con il suo
valore e il campo dell’inspector evidenziato. I dati vengono dal demo esistente.

- [Scuro](dark/app/shell/ux2-handles-ring-detail.png)
- [Chiaro](light/app/shell/ux2-handles-ring-detail.png)

Confrontati con CanvasHandles/MainNew offline (PNG e HTML aperti in Chromium).
La barra mobile e gli altri controlli elencati nel CHECKPOINT-T-8 non sono
ancora completati. Nessuna build prodotto o modifica dell’installazione.

```powershell
node scripts/fidelity/shoot.mjs --area shell --state ux2-handles-ring --only app --out ../../.jdi/phases/studio-ux-v2/evidence/t8-checkpoint
```
