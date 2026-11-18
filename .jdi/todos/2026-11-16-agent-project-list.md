# Widget per agente e template Pixel adattivo

Richiesta: organizzazione per strumento, dettagli separati per Codex e Claude,
progetti rilevati automaticamente e quote cumulative per account. Template
Pixel con CPU/RAM e GPU/VRAM raggruppate, numeri a matrice e barre segmentate.

Implementazione 0.2.32:
- Due voci nella palette, Codex e Claude Code. Moduli Dettagli separati con
  controlli condivisi; solo Claude offre aggiornamento esplicito delle quote.
- Nuovi widget in modalita elenco progetti. Compatibilita con widget salvati
  prima della modifica e scelta di un singolo progetto/sessione mantenuta.
- Un progetto corrisponde al percorso completo normalizzato Windows; viene
  mostrata la sessione piu recente, senza confondere cartelle omonime.
- Ordine: attenzione/errore, lavoro, conclusi o stato ignoto recenti. Permanenza
  configurabile 0-1440 minuti (default 10; 0 nasconde conclusi/ignoti).
- Pagine calcolate dall'altezza disponibile e dal font, durata 1-300 secondi
  (default 10); nessuna rotazione con movimento disabilitato. Esclusioni per
  progetto configurabili, conservate anche senza sessioni rilevate.
- Quote lette dall'osservazione account piu recente indipendentemente dal
  progetto selezionato, dalle esclusioni o dalla presenza di righe visibili.
- Tema personale a quattro colonne: CPU/RAM, GPU/VRAM, Codex, Claude.
  Temperature, potenza, occupazione, capacita e mini-storici 60s. Percorsi
  dei progetti non fissati nel file. Profilo VS Code e posizione del tema invariati.

Verifica mirata: test e clippy bezel-core, bezel-themes, bezel-render; test UI
agenti/libreria/i18n e editor; browser agenti e libreria. Test della libreria
aggiornato da 14 a 16 voci (obsoleto gia con l'agente generico), rieseguito
isolatamente nei quattro progetti browser. Anteprima con renderer nativo e
sessioni reali, quote disponibili da entrambi i fornitori. Evidenze sotto
 target/logs/agent-project-list/ e sorgente layout sotto
 target/scratch/agent-project-list/build-theme.py.

Consegna completata: build e setup 0.2.32 locali, SHA256 setup/ZIP e binari
verificati. Aggiornati dist/bezel e installazione LOCALAPPDATA, tema personale
sostituito atomicamente e Studio riaperto (PID osservato 34444). Impostazioni e
profili identici ai backup. Manifest in target/logs/agent-project-list/installed-0.2.32.json.

Il primo setup silenzioso e' stato rifiutato correttamente per Light ancora
attivo: la verifica Get-Process con nomi multipli non lo aveva segnalato.
Confermato con CIM, inviato il file supportato bezel-light-quit nella directory
dell'installazione aa83473fe41eefd8 e attesa uscita cooperativa; il secondo
setup ha atteso la chiusura del helper sensori ed e' terminato con codice 0.
Nessun processo terminato forzatamente. Nessun push o pubblicazione.
