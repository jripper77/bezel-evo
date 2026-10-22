# Temi predefiniti per il 3,5″, da disegnare prima sul canvas

Decisione dell'utente del 22 ottobre 2026, durante T-10: **niente versioni 480×320 in
studio-ux-v2**. I quattro temi orizzontali 1920×480 di T-9d (Pixel, Nebula, Paper,
Now Playing) restano solo per l'8,8″.

- Prima si disegnano le tavole 480×320 sul canvas di design
  (https://claude.ai/artifact/7muP7mTQAwBH8diy2AWZ6p), una per tema, e l'utente le approva.
- Solo dopo si realizzano come temi in `themes/<nome>-35/` con lo stesso generatore
  (`scripts/create-ux-v2-themes.py`), solo widget nativi e caratteri inclusi.
- Valgono le stesse regole di T-9d: nessun dato senza sorgente
  (vedi `2026-10-21-widget-data-extensions.md`), test dei temi inclusi con ingressi presenti e
  assenti, confronto tavola / frame nativo / demo.
