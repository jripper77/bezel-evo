# Confronto visivo T-7c

21 ottobre 2026. Pannelli di 360 px, italiano, chiaro e scuro. `*-detail.png`
mostra il pannello; il file senza suffisso mostra il contesto dell’app. Le tre
HTML offline CardInspectorOpen, ThemeInspectorOpen e ScreenPanelsOpen sono
state aperte in Chromium tramite Playwright, oltre alla lettura delle PNG originali.
Nessun accesso al canvas remoto.

| Stato | Scuro | Chiaro |
| --- | --- | --- |
| Card | [cattura](dark/app/inspector/ux2-card-detail.png) | [cattura](light/app/inspector/ux2-card-detail.png) |
| Tema aperto | [cattura](dark/app/inspector/ux2-theme-open-detail.png) | [cattura](light/app/inspector/ux2-theme-open-detail.png) |
| Tema chiuso | [cattura](dark/app/inspector/ux2-theme-closed-detail.png) | [cattura](light/app/inspector/ux2-theme-closed-detail.png) |
| Schermo: impostazioni | [cattura](dark/app/screen/ux2-settings-detail.png) | [cattura](light/app/screen/ux2-settings-detail.png) |
| Schermo: archivio | [cattura](dark/app/screen/ux2-storage-detail.png) | [cattura](light/app/screen/ux2-storage-detail.png) |

Il documento demo fornisce nome, contenuti, porte, memoria e file; non sono
stati copiati i dati d’esempio delle tavole. La Card appena aggiunta ha facce
vuote: la sua anteprima mostra il suo vero sfondo, non le metriche della tavola.
Il Tema usa la miniatura del canvas già renderizzato. La scheda SD compare
inserita perché lo è nello scenario demo.

Differenze intenzionali:
- Card senza Rotazione: l’utente ha confermato che quel campo nella tavola è
  un errore di disegno (D-21 e D-2026-10-18-object-rotation-01).
- Restano i controlli esistenti non tutti illustrati: sfondo animato della Card,
  priorità/ritorno e spostamento dei membri; filtri, dettagli dei file e aiuti
  dello schermo. Sono conservati per evitare perdite di funzione.
- Le etichette tradotte esistenti possono essere più lunghe delle abbreviazioni
  della tavola: vanno a capo; i nomi delle facce rimangono completi anche a 280 px.
- L’Archivio riusa le liste per supporto del gestore esistente, con menu del
  singolo file esterni alla listbox; selezione multipla, tastiera e trascinamento
  restano nello stesso gestore. La vista estesa si apre dalla Manutenzione.
- Esportazione e «Salva come» usano l’esistente salvataggio `.bezeltheme`;
  Duplica propone una copia nominata e diventa il documento salvato solo a successo.

Comandi riproducibili, da `apps/bezel-studio`:

```powershell
node scripts/fidelity/shoot.mjs --area inspector --state ux2-card,ux2-theme-open,ux2-theme-closed --only app --out ../../.jdi/phases/studio-ux-v2/evidence/t7c
node scripts/fidelity/shoot.mjs --area screen --state ux2-settings,ux2-storage --only app --out ../../.jdi/phases/studio-ux-v2/evidence/t7c
```

Il manifest riunisce le due aree: 10 stati/tema, 20 PNG inclusi i dettagli.
Le catture verificano la UI demo; prova finale nativa e hardware restano a T-10.
