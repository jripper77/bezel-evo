# Apertura file senza invio hardware e richiamo sul controllo Live — 24 ottobre 2026

## Decisioni dell'utente

Dopo la 0.2.2 JTR2 Black si apre nel canvas, ma l'utente segnala che non compare sul display fisico. Propone quindi di aprire i file disattivando Live. Per evidenziare lo spegnimento preferisce un effetto sul tasto alle alternative con nuovi stati, pulsanti o avvisi.

## Implementazione

Commit 51ee9cea:
- Apertura da libreria e import: caricamento e validazione del file prima di fermare Live; poi il solo documento selezionato passa offline prima di essere sostituito. Letture file fallite e import annullato mantengono il tema e il Live precedenti.
- La selezione degli altri schermi e le relative scelte Live non cambiano. I percorsi interni di ripristino all'avvio restano separati.
- Lo stato UI si aggiorna subito; i campioni iniziati per un documento precedente non riaccendono lo switch con dati obsoleti.
- Il controllo In diretta mostra un alone ambrato con due pulsazioni da 1,4 s; rimosso dopo 3 s. Con riduzione del movimento solo alone statico. Effetto anche quando l'utente ferma manualmente l'invio, interrotto alla riattivazione.
- Nessun testo UI aggiunto; guida inglese aggiornata.

## Verifiche

- Studio: 234 casi verificati; 233 verdi al primo giro, un confronto f64 di engine::tests::remote_canvas_is_binary_scaled_and_unchanged_until_document_edit instabile all'ultimo bit (1.5249/1.5249000000000001), passato rieseguito da solo, senza modificare il test. Nessuna ripetizione dell'intera suite.
- Clippy Studio, fmt e diff --check verdi.
- Unitari demo-backend: 48/48 verdi.
- E2E scenarios-library e multi-screen: 36/36 verdi, inclusi spegnimento automatico, durata dell'effetto e reduced-motion.

## Consegna

Build 0.2.3 avviata; primo tentativo bloccato da bezel.exe ancora aperto. L'utente conferma salvataggio e chiusura di Studio e Light; nessun processo rilevato e retry -Version 0.2.3 avviato. Nessun processo terminato forzatamente, nessun setup/installazione/push/pubblicazione.
Il problema dell'invio hardware segnalato sulla 0.2.2 non è dichiarato risolto: log con ScreenError/other durante il vecchio Live, causa specifica non acquisita. Da riprovare attivando manualmente In diretta dopo aver controllato il tema aperto offline.
## Build finale

Aggiunte le verifiche della galleria: themes.spec.mjs 36/36 verde; totale e2e mirato 72 casi distinti. Dopo la conferma di chiusura dell'utente, build-evo.ps1 -Version 0.2.3 completata in 1m27s. CLI e metadati Studio verificati 0.2.3. Commit versione 81a19dc. Hash CLI 0090F8D5A13D40F6355EF8A88827D8C2183706AF2EC1EDBFB16CAA1329B04111; Studio 7A8215BB68B3248C542716F3F3CE2CADE1311664B094A84815D914F0FC5E033B. Verbale JSON in target/logs/live-off-attention/build-verification-0.2.3.json. Avviata la copia locale target/release; nessuna installazione o pubblicazione. Restano all'utente la prova a vista dell'effetto e l'eventuale riattivazione manuale di Live sul tema importato.