# Rimozione dei media importati

Richiesta utente: manca la possibilità di cancellare un'immagine caricata
nella scheda Questo tema. Aggiunto cestino su ogni file, accessibile con nome
del file. File ancora referenziati protetti con spiegazione localizzata;
controllo backend su Theme.assets(), inclusi sfondo, immagini, aghi, font e
oggetti nascosti. Comando IPC registrato nelle capability esplicite.

La rimozione riguarda il catalogo del documento: originali intatti, byte della
sessione mantenuti per Undo; il salvataggio già esclude i file inutilizzati.
Reimportazione rende nuovamente visibile il file deduplicato. Stato separato
per documento/schermo, azzerato aprendo un altro tema. Demo coerente.

Verifiche: crate Studio 223 test passati e un fallimento iniziale del controllo
capability; aggiunta autorizzazione remove_asset e test fallito ripassato
isolato. Due nuove regressioni Rust verdi (Undo/reimportazione e isolamento
multischermo/reset), clippy all-targets -D warnings verde. UI 78 test verdi,
27 bridge ripassati dopo aggiunta controllo payload; 12 e2e Media verdi nei
quattro progetti. Log in target/logs/media-remove.

Changelog .61 preparato; VERSION resta .60 fino alla build ufficiale. Studio
.60 e Light aperti da target/release (PID16348/55056), documento utente non
salvato nella schermata: attendere chiusura dopo salvataggio prima di linkare
e sostituire gli eseguibili. Nessun setup/push/installazione. Cartella img/
non tracciata comparsa durante il lavoro, non modificata né inclusa.
