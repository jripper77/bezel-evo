# Sovrimpressione diagnostica assente sul pannello

## Causa

L'utente attiva 'Mostra le letture sullo schermo' e sceglie un angolo, ma il
display non cambia. La preferenza e il painter erano collegati al vecchio
Studio::prepare/presented. Il percorso normale EngineBridge/Light usa invece
bezel-runtime e non riceveva né disegnava quella preferenza.

## Correzione 09805cf1

- Performance/Corner e painter trasferiti nel runtime condiviso, con reexport
  per mantenere coerente il fallback legacy di Studio.
- Opzione wire debug_overlay opzionale e default per vecchi messaggi; Studio
  la sincronizza al cambio preferenze e al ripristino, Light legge il valore
  salvato anche all'avvio. Cambi alla sola diagnostica non riavviano i sensori.
- Sovrimpressione solo sulla copia del frame destinata al presenter fisico:
  canvas, anteprime e documento salvato restano senza marcature.
- Cambio angolo e disattivazione invalidano l'invio precedente. Temi statici
  con overlay attivo aggiornano ogni 250 ms, aggiungendo traffico come indicato
  nelle preferenze. Live spento continua a non inviare.
- Coda completamenti I/O limitata a 20, timestamp reali e statistiche di
  trasferimento opzionali. Tempo campionamento misurato dal sampler. Non sono
  FPS del gioco né refresh del pannello: sono consegne e tempi del PC.

## Verifiche

Test regressione sui pixel inviati dal FakeConnector: quattro angoli, canvas
pulito, tema invariato, refresh statico e pulizia dopo stop. Disabilitando
temporaneamente il paint del frame uscente il test fallisce; ripristinandolo
passa. Runtime finale: 77 test passati, 1 hardware ignorato; include i test del
painter trasferiti. Test di CLI e Studio passati; clippy dei tre crate con
all-targets e -D warnings, fmt e diff check verdi. Nessun modulo JS cambiato.
Quattro casi e2e debug.spec.mjs verdi. Log overlay-* nella worktree
target/logs/fps-integration. Nuovi test usano la fixture TempDir già esistente.

Guide en/pt-BR e changelog aggiornati; note cumulative draft 0.2.9 nel dist del
repository principale. Build Windows 0.2.9 riuscita in 1m52s con build-evo.ps1;
CLI, metadati Studio e sensori simulati verificati, commit versione 5861d77.
Hash in build-hashes-0.2.9.json. Chiesta chiusura di Studio 38836 e Light 10848
prima della copia; conferma pendente, dist/bezel resta 0.2.8. Prova sul pannello
reale ancora necessaria; nessun push, commento all'autore o pubblicazione.
