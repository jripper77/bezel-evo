---
order: 22
name: Display idle saver
---
- **Slug:** display-idle-saver
- **Goal:** Quando Windows spegne i monitor (GUID_CONSOLE_DISPLAY_STATE) Bezel ferma la cattura audio del loopback (che oggi blocca lo standby del PC), abbassa la luminosità degli schermi, riduce al minimo il traffico verso i dispositivi e mostra un logo che si sposta ogni N secondi contro il burn-in; alla riaccensione dei monitor riprende tema e cattura. Sospensione/risveglio del PC (WM_POWERBROADCAST) applica lo stesso stato e riprende al risveglio.
