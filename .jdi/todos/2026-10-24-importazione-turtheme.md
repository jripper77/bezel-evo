# Ripresa importazione .turtheme ? 24 ottobre 2026

## Stato recuperato

- Fase Studio UX v2 conclusa, 0.2.0 pubblicata e installata secondo lo stato registrato.
- Il repository conteneva modifiche non committate all'importatore TURZX: font, sensori e meteo.
- L'utente ha confermato esplicitamente di proseguire questo lavoro.

## Completamento locale

- Adattati i test al nuovo contesto dell'importatore; risolti i nove errori di compilazione iniziali.
- Ricerca dei font nei percorsi vicini al tema, incluso `<app>/fonts`, usando i nomi interni dei file.
- Font trovati inclusi negli asset; nomi file uguali con contenuti diversi ricevono percorsi distinti.
- Studio passa famiglie/alias del catalogo font e sensori del PC. In assenza del catalogo sensori per un errore del motore mantiene i binding, senza impedire l'importazione.
- Font mancanti sostituiti con Inter; supportati i nomi installati con suffisso di peso.
- GPUFAN in RPM, RAMMODEL sul nome memoria, ventole opzionali e modello RAM conservati come testo salvato se manca il sensore.
- Testo meteo convertito nel meteo di Bezel con Roma iniziale e avviso di scegliere la citt?.
- Avvisi aggiornati in en, pt-BR e it, fixture codici allineata; guida e changelog Unreleased aggiornati.
- Test del corpus convertito a TempDir; nessuna pulizia di dati applicativi o artefatti preesistenti.

## Verifiche

- `cargo test -p bezel-themes --locked`: 100 test verdi; corpus esterno ignorato perch? richiede dati locali non presenti nel percorso previsto.
- `cargo test -p bezel-studio --locked`: 232 test e un doctest verdi; test hardware ignorato. Dopo l'ultima modifica al backend, ripassato il solo test di importazione.
- `cargo clippy -p bezel-themes -p bezel-studio --all-targets --locked -- -D warnings`: verde sul codice finale.
- fmt dei due crate e `git diff --check`: verdi.
- UI messaggi backend: 10 test verdi.
- e2e `scenarios-library.spec.mjs`: 20/20 verdi.
- Provati font risolti dal nome interno, font mancante, sensori presenti/assenti, meteo, collisioni dei nomi file e salvataggio/riapertura nativa con font incluso.

## Consegna e prossimo passo

Solo sorgenti locali: VERSION resta 0.2.0; nessuna build di prodotto, sostituzione degli eseguibili, installazione, push o pubblicazione. `img/` preesistente conservata.
Prossimo passo operativo: build locale con nuova versione tramite build-evo.ps1 e prova in Studio su temi reali del produttore. Prima di sostituire eseguibili aperti preservare i documenti e chiedere salvataggio/chiusura secondo AGENTS.md.
La CLI continua a usare import_path senza informazioni sul PC: include i font del produttore ma conserva i nomi installati e i binding quando non ha dati per decidere.
