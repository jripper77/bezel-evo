# T-5 — Immagine e GIF

Date: 20–21 ottobre 2026. Branch `ux2/t5`, worktree `F:\dev\bezel-ux2-t5`, partenza `0aa377a` (T-1…T-4 unite).
Decisioni: D-2026-10-20-studio-ux-v2-04 (Immagine), **D-2026-10-21-studio-ux-v2-16** (video, decisa durante la task).
Tavola: ImageInspectorOpen.

**T-5 completata e verificata** per immagini e GIF, pronta per il merge `--no-ff` in `windows-improvements`.
I video negli oggetti Immagine sono rimandati alla nuova T-11 (vedi sotto).

## Fatto

- `44ad938` — dominio, file del tema, renderer:
  - `ElementKind::Image.anchor`: nove punti di ancoraggio (`Anchor::shares`) che decidono cosa resta visibile con
    «Riempi» (Cover); con Contieni/Allunga non cambia nulla. Il centro dà esattamente il ritaglio di prima.
  - `ElementKind::Image.playback`: `once` (una volta, poi resta sull'ultimo fotogramma) e `speed` (0,25–4; la tavola usa
    0,5×/1×/2×). `Playback::moment` e `Playback::next_change` (puri, testati) danno il fotogramma e la scadenza; il
    runtime pianifica i fotogrammi delle GIF per oggetto (`Theme::shown_pictures`) e smette di chiederne finita la
    riproduzione singola.
  - DTO: `anchor` (`topLeft` … `bottomRight`, omesso se `center`), `once` (omesso se falso), `speed` (omesso se 1).
  - Renderer: `images::plan_at`/`ImageCache::placed_at` con l'ancoraggio, `Scaled::frame_playing` con la riproduzione.
  - Trasparenza radiale con centro, estensione e punto medio: **esisteva già** da renderer a inspector (`fadeField` di T-0);
    coperta dal test pixel esistente `radial_image_mask_keeps_assets_and_multiplies_pixels`.
- `c27d678` — demo: la GIF segnaposto del demo rispetta velocità e «una volta» (`demoGifFrame`, `demoGifDue`), e il
  demo-backend chiede fotogrammi solo mentre una GIF sta ancora andando.

## Decisione presa durante la task (D-2026-10-21-studio-ux-v2-16)

La tavola mostra un .mp4 dentro un oggetto Immagine, ma gli oggetti Immagine non riproducono video (solo lo sfondo lo
fa) e nessuna decisione lo chiedeva. L'utente ha scelto: T-5 si chiude con immagini e GIF; nuova **T-11 «Video negli
oggetti Immagine»** dopo la Wave 2, che parte misurando sull'8,8″ gli fps di un video in una zona piccola, media e
grande (dati del 18 ottobre in `docs/experiments/scene-streaming.md`), poi limiti decisi dall'utente. Fino ad allora in
T-7b l'inspector mostra le voci di riproduzione solo per le GIF: **differenza voluta rispetto alla tavola**.

## Test eseguiti

- `cargo fmt --all -- --check` verde; `cargo check --workspace --all-targets` verde; clippy `-D warnings` pulito su core,
  themes, render.
- `cargo test`: bezel-core **327**, bezel-themes **88**, bezel-render **124**, bezel-runtime **58**, verdi.
- Nuovi: ancoraggi e riproduzione nel dominio (velocità, una volta, prossimo cambio, limite 4×), `shown_pictures` con la
  riproduzione, DTO (ogni ancoraggio, valori omessi, limiti, rifiuto), pixel: Riempi con ancoraggio sinistra/centro/
  destra e angoli, Contieni che ignora l'ancoraggio, GIF a 2×, a 0,5×, in ripetizione e «una volta» sull'ultimo fotogramma.
- UI unitari (`image-playback`, `demo-backend`, `static-guards`): **62/62**.
- e2e `inspector-components`, `inspector-trial`, `object-rotation`, `scenarios-library`, `gif-search`: 68 verdi; rosse le
  stesse 6 che sono rosse anche sulla base `0aa377a` (verificato su `dark-en`: `object-rotation` ×4 cercano il campo
  «Rotation» dell'inspector di prima, `gif-search` ×2 vanno in timeout) → T-10. In più «demo render … parity fixture at
  30°» era rossa sotto carico e **passa rilanciata da sola** (4/4).

## Differenze e scelte da segnalare

- Video negli oggetti Immagine: rimandati a T-11 (sopra).
- «Angoli» dell'immagine (tavola: 12 px, angoli singoli) non è nell'acceptance di T-5 e non esiste ancora per le immagini:
  annotato per T-7b/T-10 (si può riusare `cornersField` e la maschera delle forme).
- Velocità consentite 0,25–4 nel file; l'inspector offre 0,5×/1×/2×.
- Nel demo le immagini sono segnaposto (non decodifica i file): l'ancoraggio non si vede, la riproduzione sì.

## Riuso e duplicazione

Riusati `Timeline` (frame e prossimo cambio), `plan`/cache delle immagini (ora con ancoraggio, chiave di cache invariata),
`image_change` del runtime, `fadeField` e la dissolvenza radiale esistenti. **Nessuna duplicazione**: `plan` chiama
`plan_at`, `placed` chiama `placed_at`, `frame_at` chiama `frame_playing`.

## Per le task successive

- T-7b: Adatta (Contieni/Riempi/Allunga), griglia dei 9 punti (attiva solo con Riempi), Trasparenza radiale (esistente),
  Riproduzione/Velocità solo per GIF; nessun «Audio» finché non c'è T-11.
- T-11: video negli oggetti Immagine (misura sull'8,8″ prima di tutto).
- T-10: migrare `object-rotation` ×4 e `gif-search` ×2.
