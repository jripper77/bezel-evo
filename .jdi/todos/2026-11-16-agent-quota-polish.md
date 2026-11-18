# Logo Pixel, dimensione quote e risposte vuote Claude

Richieste: ripristinare logo Bezel perso nel nuovo template, ridurre i testi
quote, risolvere indisponibilita intermittente Claude.

Diagnosi: il client nativo get_usage rispondeva success=true e
rate_limits_available=true ma rate_limits=null. Il vecchio codice salvava
entrambi i valori null, cancellando le finestre valide. Riprodotto con chiamate
dirette al client installato; non sono state lette o esportate credenziali.
Successivamente il client ha nuovamente restituito dati reali validi,
verificati nell'anteprima (residuo osservato 4%/87%).

Correzione 0.2.34:
- Risposte senza finestre valide restituiscono errore e non modificano la cache.
  Risposte parziali conservano le finestre precedenti senza ringiovanirne la data.
- Ultimi dati validi utilizzabili fino a 15 minuti; dopo tre minuti il renderer
  indica l'eta di aggiornamento. Scadenza della finestra sempre rispettata.
- Lock di file e timestamp dell'ultimo tentativo condivisi Studio/Light:
  massimo un tentativo al minuto anche in caso di errore. Il worker registra
  l'errore nei log senza includere credenziali.
- Test con TempDir su risposta vuota/parziale, scadenza, separazione account,
  esclusione reciproca e intervallo tra tentativi.
- Campo quotaSizePercent 50-100, default 100 per compatibilita e controllo
  i18n nei Dettagli. Nel template personale 70% (circa 16px contro 23px dei progetti).
- Ripristino emblema originale Pixel e scritta Bezel nel bordo inferiore,
  senza spostare CPU/GPU/progetti. Script tema idempotente, da rieseguire
  sulla copia appena salvata prima di installare.

Test/clippy dei quattro crate toccati e controlli UI/browser mirati.
Anteprima reale target/logs/agent-quota-polish/preview.png.
Build 0.2.34 e setup/ZIP/SHA256 pronti e verificati (175 file, eseguibili
uguali alla build). INSTALLATA dopo conferma di salvataggio/chiusura: setup silenzioso codice 0,
hash LOCALAPPDATA/dist/bezel uguali alla build. Tema rigenerato dalla copia
appena salvata: logo nel footer, quote 70%, resto della disposizione preservato.
Preferenze e profilo invariati, Studio riaperto (PID osservato 17552). Quote
Claude nuovamente disponibili nel rendering installato, con osservazione fresca
4%/87%; nessun ripristino manuale della cache necessario.
Backup F:\Dev\bezel\target\scratch\agent-quota-polish\backup-20261116-160106.
Manifest target/logs/agent-quota-polish/installed-0.2.34.json.
Nessun push o pubblicazione.
