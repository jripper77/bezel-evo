# Widget Agente ? 16 novembre 2026

Implementato il widget dedicato, secondo la scelta dell?utente, con una istanza
per agente. Selezione Codex/Claude Code, sessione automatica o fissata, messaggio,
attivit? corrente, stato osservato e sezioni facoltative per quota cinque ore,
quota settimanale, percentuali consumate/residue e conto alla rovescia.

I registri locali sono letti in un worker dedicato solo quando richiesti. La
lettura ? incrementale e limitata, con gestione delle righe parziali e dei file
troncati. Sono considerate fino a 40 sessioni recenti per agente, escludendo i
sottoagenti di Claude. Il testo mostrato ? l?ultimo messaggio registrato, con
numero di righe configurabile. I formati dei registri sono interni ai prodotti
e possono richiedere adeguamenti quando cambiano. Dopo cinque minuti senza
attivit? una sessione precedentemente attiva passa a stato sconosciuto.

Le quote provengono dai token_count di Codex e, per Claude, da un collegamento
esplicito della status line nelle propriet?. Il collegamento effettua una copia
delle impostazioni e una scrittura atomica. La precedente status line riceve lo
stesso input attraverso la shell originale. Lo script esporta soltanto ID e
quote. Per Claude serve riavviare il programma dopo il collegamento e generare
una risposta. I limiti scaduti/mancanti vengono indicati come non disponibili.
Le quote sono condivise fra le sessioni nella stessa directory di configurazione
dell?agente. Non ? prevista una selezione separata di account nello stesso home.

## Verifiche

- Test dei sei crate modificati. I 248 test Studio passati nella prima esecuzione
  sono stati integrati dal superamento isolato dei tre controlli corretti.
- Corretto il permesso Tauri del nuovo comando di collegamento.
- Riallineata la guardia di dipendenze al controllo aggiornamenti gi? presente
  in bezel-runtime dalla 0.2.26, mantenendo le verifiche sugli altri adattatori.
- 186 test UI e 32 prove browser dell?area widget/media superati.
- Pixel test del renderer per consumo, aggiornamento della barra e scadenza.
- Prova reale dello script PowerShell, sovrascrittura atomica e passaggio dello
  stdin alla status line precedente riusciti.
- Sonda locale esplicita riuscita, con sessioni e messaggi di entrambi gli agenti
  e quote Codex. La configurazione personale Claude non ? stata modificata.
- Clippy sui sei crate, con tutti i target e warning negati, superato. Formato
  Rust verificato. Nessuna suite completa o pubblicazione eseguita.
- Build Windows 0.2.29 completata con scripts/windows/build-evo.ps1. I risultati
  verificabili sono raccolti in target/logs/agent-widget/.

## Consegna

Dopo la conferma utente di salvataggio/chiusura e la verifica del processo
terminato, aggiornati in loco i due eseguibili in dist/bezel. Hash SHA256 di
entrambi uguali ai risultati della build, verificati in installed-hashes-0.2.29.json.
La CLI copiata risponde bezel 0.2.29. Copie di recupero conservate nella directory
univoca target/scratch/agent-widget/backup-*. L?installazione in LOCALAPPDATA
non ? stata aggiornata. Per provare questa build aprire dist/bezel/bezel-studio.exe.
Nessun installer avviato, push o release pubblica. Le schermate di verifica sono
in target/logs/agent-widget/browser. Il controllo automatico ha respinto la pulizia
degli intermedi con motivazione generica blocked by policy, quindi i risultati
restano raggruppati nella directory del task.


## Installazione richiesta successivamente

Creati setup, ZIP e SHA256SUMS della stessa build 0.2.29 tramite package-evo.ps1
con SkipBuild. Verificati checksum e 175 ingressi del portabile. Setup avviato
su richiesta e installazione completata in LOCALAPPDATA\Bezel Evo. Hash dei
binari installati uguali alla build. Studio 0.2.29 aperto, Light avviato.
