# Anteprima Anello senza riquadro

22 ottobre 2026. Worktree `F:/dev/bezel-inspector-preview-surface`, branch
`fid/inspector-preview-surface`, base `f69fa8bc` di `windows-improvements`.

## Correzione

La segnalazione dell’utente è confermata nell’HTML di `RingInspectorOpen`:
l’SVG da 72 × 72 è direttamente nell’intestazione, senza fondo o angoli.
Lo Studio applicava a tutte le anteprime lo stesso riquadro scuro arrotondato.
Ora il componente esistente espone il tipo dell’oggetto come dato e il CSS
rende trasparente il fondo dell’Anello, senza arrotondamento. Dimensioni,
centratura, pixel del renderer e comandi rimangono quelli esistenti.

La tavola `MainNew` contiene invece un riquadro anche per l’Anello:
si segue la tavola specifica `RingInspectorOpen`, come richiesto dall’utente.
Le tavole di Valore, Testo, Barra, Lancetta, Grafico, Orologio, Meteo,
Player e Forma mostrano il fondo: non è stato rimosso da questi tipi.
Nessun nuovo primitivo, colore, stringa, campo del tema o comportamento runtime.
Nel tema chiaro la traccia bianca al 15% dell’Anello di esempio si confonde
con l’intestazione bianca: osservato nella cattura, non si è sostituito
il colore reale del tema con un colore inventato dalla UI.

## Commit e verifica

- `b6ead42` — `fix(studio-ux-v2): remove the ring preview tile`.
- `952e012` — `test(studio-ux-v2): retain the text preview backdrop`.
- Unitari `element-preview`, `preview-animation`, `static-guards`, `i18n`:
  **73 verdi**.
- `node scripts/e2e-slot.mjs tests/e2e/inspector-preview.spec.mjs`:
  **24 verdi**, chiaro/scuro, en/pt-BR. Aggiunte asserzioni sul fondo
  trasparente e sull’assenza di arrotondamento, mantenuti i controlli sui
  pixel trasparenti, sui colori, sull’Undo e sull’accessibilità.
  Aggiunto anche un controllo sul mantenimento del fondo per il Testo:
  rieseguiti i suoi quattro casi chiaro/scuro en/pt-BR, tutti verdi.
- Rigenerazione della galleria finale italiana chiaro/scuro e controllo
  integrità: **verde**, 108 stati, 208 PNG leggibili, tutti i collegamenti
  validi e hash SHA-256 aggiornati in `evidence/final/verification.json`.
  Catture Anello ispezionate in chiaro e scuro; nota della divergenza
  tra le tavole visibile nella galleria.

Nessuna build, installazione o pubblicazione. La fase resta in attesa
della verifica manuale; il limite delle immagini nel canvas demo è invariato.
