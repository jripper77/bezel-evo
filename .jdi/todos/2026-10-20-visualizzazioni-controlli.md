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

Consegna locale prevista .60 mediante build-evo.ps1; la release pubblica resta
.59. Nessun nuovo setup o push richiesto in questo turno.
