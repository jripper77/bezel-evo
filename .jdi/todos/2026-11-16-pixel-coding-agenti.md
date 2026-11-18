# Pixel V2 Coding Agents - 16 novembre 2026

Richiesta utente: derivare da Pixel V2 Coding un tema per VS Code, con Codex,
Claude, consumo RAM/VRAM, utilizzo CPU/GPU e temperature. Scelta autonoma della
composizione autorizzata dall'utente.

Creato Pixel-V2-Coding-Agents.bezeltheme nella libreria personale RoamingAppData.
Base originale Copia-di-Pixel-V2-1.bezeltheme (nome interno Pixel V2 Coding).
Nuova composizione 1920x480 con 33 elementi modificabili, finitura puntinata
originale, quattro pannelli hardware superiori e due pannelli agenti inferiori.
CPU/GPU: carico percentuale, temperatura, potenza e modello. RAM/VRAM: memoria
occupata in GiB, percentuale, capacita e barra a segmenti; RAM libera aggiuntiva.
Agenti: ultima sessione aggiornata per ciascuno, due righe messaggio scorrevoli,
attivita, stato e quote cinque ore/settimanali con residuo e reset.

Aggiornato il profilo VS Code gia esistente: COM6 usa il nuovo tema e la regola
osserva il processo code, priorita 20, ritorno 3 secondi. La regola precedente
osservava erroneamente il blocco del computer. Priorita sopra Music Player (10)
per dare precedenza al coding quando VS Code e aperto. Backup del profilo e del
tema sorgente in target/scratch/pixel-coding/backup-20261116-004801.
Manifest di verifica in target/logs/pixel-coding/theme-profile-verification.json.

Collegata la status line Claude tramite la funzione di setup gia disponibile,
con backup automatico e conservazione del comando precedente. Quote disponibili
dopo il prossimo riavvio di Claude Code e una nuova risposta.

La prova nativa ha rilevato un difetto del widget consegnato nella 0.2.29:
Windows conserva un LastWriteTime vecchio mentre il writer tiene aperto il
registro. Corretto bezel-sensors usando i timestamp RFC3339 degli eventi,
con fallback per i vecchi registri senza timestamp. Test regressione con
mtime fermo un'ora indietro e sessioni ordinate correttamente. 108 test sensori
passati, 7 test hardware ignorati, clippy su tutti i target superato.
Anteprima reale verificata: Codex Working, aggiornato 2s; Claude Completed,
ultimo progetto EIDOLON. Tutti i sensori scelti risultano disponibili sul PC.

Build Windows 0.2.30 completata. Setup/ZIP/SHA256SUMS creati e verificati, 175
file nel portabile, binari corrispondenti alla build. Attesa conferma di
salvataggio/chiusura Studio prima dell'aggiornamento degli eseguibili.
Il controllo UI nativo e indisponibile (pipe non trovata) anche dopo i tentativi
di ripristino documentati. Verificata l'anteprima con il renderer reale e il
profilo salvato, senza modificare i documenti non salvati in Studio.


## Consegna completata

Dopo conferma utente di salvataggio/chiusura, aggiornato dist/bezel in loco e
installata la 0.2.30 con il setup gia verificato, in modalita silenziosa.
CLI installata risponde 0.2.30. CLI e Studio installati hanno hash identici
alla build, registrati in installed-hashes-0.2.30.json. Copie binari precedenti
in target/scratch/pixel-coding/bin-backup-0.2.29.
Impostata la selezione editor COM6 sul nuovo tema, con backup delle preferenze
nella directory di recupero del task. Riavviato Studio dall'installazione
LOCALAPPDATA. Ultima prova renderer direttamente sul nuovo archivio riuscita.
Anteprima finale in target/logs/pixel-coding/final-theme.png.
La verifica visiva della finestra o del display fisico non e disponibile per
il guasto del collegamento di controllo desktop. Nessun push o pubblicazione.

Pulizia finale del solo file temporaneo del tentativo cross-volume respinta
dal controllo automatico con blocked by policy. File conservato nel percorso
scopato target/scratch/pixel-coding/Pixel-V2-Coding-Agents.tmp. Nessun tentativo
di eliminare dati o cartelle runtime dell'applicazione.
