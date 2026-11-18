# Pixel workspace e richiamo di stato morbido

Richiesta: ripensare il layout con maggiore cura, superando il logo ingombrante
nel pannello CPU e il lampeggio grezzo del bordo.

Proposta implementata 0.2.35:
- Intestazione indipendente con firma Bezel, contesto coding e orologio.
- Due moduli hardware compatti (CPU/RAM e GPU/VRAM), due moduli agenti ampi.
  Canvas 1920x480, quattro colonne 336/336/576/576 con margini uniformi.
- Numeri Doto mantenuti per hardware, tipografia piu leggibile nei progetti,
  etichette ridotte e quote separate in una fascia account discreta.
- Stato a destra su colonna autonoma, nome progetto a sinistra, separatori sottili.
- Richiamo di due secondi con inviluppo sin^2, sfumatura nel colore dello stato
  e piccolo accento laterale: nessun lampeggio on/off o bordo del pannello.
  Testo sempre fermo. Evidenzia solo la riga del progetto interessato e segue
  il suo riordinamento o pagina; indicazione sull'intestazione se la conclusione
  e' nascosta dal filtro temporale. Movimento disattivato sopprime l'effetto.
- Mantiene il campo serializzato blinkOnChange per compatibilita, etichetta UI
  aggiornata in en/it/pt-BR a Evidenzia cambi di stato.

Verifica: test/clippy bezel-render, 40 test UI mirati, 12 browser area agenti.
Test pixel su isolamento della riga e dell'agente; curva dolce, termine dopo
2s, assenza di eventi su avvio/messaggi/quote/cambio sorgente.
Anteprime native e animazione di evento simulato in
 target/logs/pixel-workspace-redesign/; sorgente tema e probe raggruppati in
 target/scratch/pixel-workspace-redesign/.
INSTALLATA 0.2.35: setup/ZIP/hash verificati, eseguibili installati e dist/bezel
uguali alla build. Studio era chiuso; Light arrestato tramite richiesta cooperativa,
setup codice 0 e tema rigenerato dall'ultima copia salvata. Preferenze/profili
invariati, Light riavviato sul tema COM6. Anteprima nativa installata verificata.
Backup F:\Dev\bezel\target\scratch\pixel-workspace-redesign\backup-20261116-161706. Manifest in installed-0.2.35.json sotto la cartella log del task.

La prima codifica GIF in debug era troppo lenta: interrotto solo il probe
pixel-workspace-preview e i suoi eventuali client diagnostici, quindi generata
anteprima 960x240 a 10fps (31 fotogrammi) con codifica veloce. Nessun processo
Studio/Light terminato forzatamente. Nessuna promessa di pulizia TempDir dopo
l'interruzione del probe. Sorgenti del probe e artefatti utili conservati per QA.
Nessun push o pubblicazione.
