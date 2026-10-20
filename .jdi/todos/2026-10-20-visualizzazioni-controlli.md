# Controlli delle visualizzazioni e aggiornamento a Kay

## Messaggio esterno autorizzato

L'utente chiede di scrivere a DarkKye e indica la PR #1. Letti descrizione e
commenti: autore DarkK3y-v2 (Kay), in attesa della release per riadattare
PresentMon. Pubblicato aggiornamento con release .59, base windows-improvements,
ricerca eseguibile senza PATH e sessione PresentMon dedicata come già concordato.
Commento: https://github.com/jripper77/bezel-evo/pull/1#issuecomment-6280099949.

Su due correzioni esplicite dell'utente, finale aggiornato a:
«Thanks again for contributing! I'm still learning too.» Nessuna proposta di
sviluppare Bezel insieme o di proprietà/conduzione condivisa del progetto.

## Proseguimento delle visualizzazioni

- Preset con piccole anteprime SVG coerenti con gli altri selettori dell'app;
  pulsanti selezionabili, stato aria-pressed, etichette complete su più righe.
- Adatta alle celle: azione esplicita per matrice, mantiene X/Y/larghezza e
  limita l'altezza allo spazio disponibile. Numero editato confermato prima
  della misura; gestione focus evita di perdere il click durante il ridisegno.
  Undo ripristina la geometria precedente.
- Numero di pixel distinto da tacche del quadrante; nomi accessibili univoci
  per minimo/massimo della serie inferiore. Parità i18n en/pt-BR/it.
- Nessuna modifica al formato temi, al motore nativo o ai dati degli utenti.

Verifiche: 53 unitari/statici/i18n mirati e 16 e2e dell'area verdi nei quattro
progetti; screenshot chiaro/scuro ispezionati. Output in
target/logs/visualizations-next e target/scratch/visualizations-next.

Consegna locale .60 completata mediante build-evo.ps1 -Version 0.1.60
(2 minuti); CLI e metadati Studio verificati. Avviato direttamente
target/release/bezel-studio.exe: PID 16348 rispondente, titolo Bezel Evo 0.1.60;
Light PID 55056. Nessun processo Studio/Light presente prima della build.
SHA256 Studio 8D47322EE838A6A063DD7AD67A77DC509E7A0EEB021A826DDD8B0C55592B8095;
CLI AACBACBEC47E07D82D661A106D89812031275BDAD28DAE4B9080B71E4ADF5495.

Avvio osservato nei log: timeout IPC e pipe occupata fino alle 00:14:18 UTC;
screen2 segnala un errore I/O e 4427 eventi scene fino alle 00:15:02, poi
riprende Present regolari (1393 entro 00:15:52). Screen1 ha 3996 Present fino
alle 00:14:55. Non dichiarare avvio senza errori o entrambi gli schermi
verificati a vista: la causa della fase transitoria non è diagnosticata.
Nessun arresto forzato o modifica ai documenti per questa verifica.

La release pubblica resta .59 e l'installata .45. Nessun nuovo setup o push.
Da confermare a vista i nuovi controlli e il comportamento di riaggancio
all'avvio, soprattutto sull'8,8 pollici.
