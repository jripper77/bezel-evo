# mirrored-graph-scales — contesto

Obiettivo: nei grafici a due serie la scala verticale si sceglie per ciascuna serie.
Decisione bloccata: D-2026-11-11-mirrored-graph-scales-1 (concordata in chat il 2026-11-11).

Perché: con il picco comune non si possono affiancare serie con unità diverse (°C sopra, % sotto), ordini di grandezza diversi (download 80 MiB/s, upload 200 KiB/s) o una serie con limiti naturali (0–100 %) accanto a una automatica.

## Definition of Done
- Modello e formato del tema: `secondaryAutoscale` e `sharedScale` facoltativi; un tema senza i due campi si scrive e si disegna come prima.
- Renderer: picco comune, picco proprio, serie fissa accanto a una automatica.
- Inspector: Automatica/Fissa della serie inferiore con min/max e picco attuale; interruttore «Stessa scala per le due serie» solo con entrambe automatiche; Annulla funziona.
- Testi in en, it, pt-BR con parità di chiavi.
- Test: DTO, renderer, peak watch, e2e visualizzazioni.
