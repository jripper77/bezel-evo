# Importazione video, storage e indicatori - 24 ottobre 2026

## Richiesta e riscontro reale

L'utente segnala video presente ma dichiarato assente, bordi arancioni incoerenti nei controlli inquadratura e lampeggio della pillola Bezel (tempo/memoria). Chiede che l'importazione controlli lo storage e proponga la copia solo se manca il video: sorgente locale automatica, altrimenti selezione del file.

Archivio reale COM6 letto senza scritture: flash interna contiene
`0_Abstract_Background_1920x1080.mp415055558.mp4`, 5,8 MB.
La sorgente nella cartella vendor ha 5806640 byte, H.264/yuv420p, 480x1920.
Il nome normalizzato da Bezel non coincideva con quello originale (maiuscole/punteggiatura).

## Intervento

- Ricerca del nome canonico prima in flash poi su scheda, quindi del nome vendor originale. Quando la sorgente locale esiste, alias solo per video compatibile e conversione identica, verificando i byte. Senza metadati locali, riuso del MP4 nativo con inquadratura predefinita compatibile con il pannello; inquadrature personalizzate richiedono la propria copia.
- Preflight sola lettura separato dall'avvio del video; nuovo comando Studio consentito esplicitamente nelle capability Tauri. Funziona anche con Live spento.
- Importazione: riuso della copia presente, conferma normale di conversione/invio se manca, selezione del file se manca anche la sorgente locale. Annullare conserva il tema importato e non carica file. Live rimane spento.
- Probe temporaneo distinto dalla sorgente di un upload in attesa. Riconfermare lo stesso asset non cancella il file usato dalla nuova operazione.
- Controlli inquadratura con bordi e selezione neutri del sistema condiviso. Pillola Bezel aggiornata al massimo ogni secondo con tempo medio; dettagli costruiti quando il popover viene aperto e campioni grezzi conservati.

## Verifiche

- UI: 131 unitari mirati verdi (memoria, inquadratura, video, demo, i18n, messaggi backend).
- E2E: 16 importazione/copia, 8 bordi/indicatore, 28 memoria/inquadratura e 32 archivio, tutti verdi (84 casi distinti).
- Controllo visivo catture chiare/scure dei nuovi bordi: coerenti; evidenze in target/logs/import-video-polish/framing-*.png.
- Rust finale: 328 test core e 2 doctest verdi; 236 test Studio verificati e 1 doctest verde. Il noto confronto f64 di engine::tests::remote_canvas_is_binary_scaled_and_unchanged_until_document_edit ha fallito sotto carico (1.1762 contro 1.1762000000000001) ed è passato isolato; nessuna suite completa ripetuta per il flaky. Clippy di core/Studio con tutti i target e -D warnings, fmt mirato e diff --check verdi.
- Core: regressioni su nome vendor in flash/scheda, dimensione errata, inquadratura modificata e sorgente locale assente. Studio: preflight offline senza scritture, selezione sorgente e preparazione/upload.
- Prova hardware limitata alla lettura dell'archivio. Nessun upload, cancellazione o avvio video automatico eseguito sul dispositivo durante le prove.

## Consegna

Build locale 0.2.5 tramite build-evo.ps1. Prima esecuzione bloccata dalla CLI ancora aperta; utente ha poi confermato salvataggio e chiusura di Studio/Light, assenza processi verificata prima del retry -Version 0.2.5. Nessun push o pubblicazione richiesti.


## Consegna completata

Build-evo.ps1 -Version 0.2.5 completata in 1m32s dopo la chiusura confermata.
Commit: core 193ab99d, controlli/indicatore c7ddc756, importazione 3918d186,
versione/changelog 6326624.
La cartella canonica dist/bezel era assente: ripristinata senza creare una cartella per versione, con i due eseguibili, risorse sensori, temi stock e launcher. Nessuna cartella precedente o dato utente eliminato; installazione ufficiale LocalAppData non modificata.
CLI --version e metadati Studio confermano 0.2.5. Hash delle copie confrontati con target/release:
- CLI: 8BBEE9C289D7DD6F8D34451CCE21BA71327ABC3585232F397D9E304BB8215B1A.
- Studio: 3D90CC682F6F3F81E8D13A642CFE3EC3040B48769119926C776F99C3C145C5DD.
Verbale target/logs/import-video-polish/build-verification-0.2.5.json.
Studio riaperto da dist/bezel, PID 12672, rispondente, titolo Bezel Evo 0.2.5;
Light PID 22044 dalla stessa cartella. Log del lancio nel gruppo della prova.
Da confermare con l'utente sul display reale: reimportare JTR2 Black, riuso del video originale e successivo Live. Nessuna prova di riproduzione fisica dichiarata completata. Nessun setup, push o pubblicazione.
