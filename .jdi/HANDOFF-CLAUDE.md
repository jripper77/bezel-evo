# Passaggio a Claude — 19 ottobre 2026

## Design — aggiornamento 20 ottobre 2026 (sera)

Canvas «Bezel Studio Redesign»: https://claude.ai/artifact/7muP7mTQAwBH8diy2AWZ6p
(privato; si legge con lo strumento Artifact, action `read`, path
`project/<Tavola>.dc.html`, indice `project/canvas.json`).

- **Già nell'app (0.1.55+)**: shell, logo Pixel e `icon.svg`, pannelli Sensori/Livelli/
  Temi/Media/Schermo/Archivio, galleria temi, inspector con anteprima e schede, Card,
  Grafico, Media player, pallino stato sensori, sfondo «in uso + Cambia sfondo…»,
  dialogo Profili (fatto da Claude nella phase `profiles-dialog-ux`).
- **Difetti misurati** (larghezze fisse: libreria 271 px, inspector 320 px):
  nome sensore 51 px su 151 necessari (mini grafico 60 + valore 80 fissi); nome faccia
  Card 41 px contro 83 dell'etichetta «in vista». Proposta: righe su due livelli o
  icone compatte, più colonne ridimensionabili con larghezza ricordata.
- **Decisioni utente**: «Card» non si traduce; Profili con due modalità
  Automatico/Manuale (Manuale include i temi di base); la priorità è della regola e
  non va mostrata sul profilo (idea aperta: ordine dei profili trascinabile al posto
  del numero, cambia il formato salvato); scrollbar sempre quella condivisa di
  `styles/base.css`; schermata di accensione centrata, logo e titolo piccoli.
- **Temi predefiniti** (tavole ThemePixel/ThemeNebula/ThemePaper/ThemeNowPlaying,
  1920×480): seconda versione più fine; l'utente vuole dettagli e niente elementi
  enormi. Font OFL da includere (Doto, Space Grotesk, Fraunces, Instrument Sans,
  Bricolage Grotesque, JetBrains Mono). Richiedono lavoro: sensori per core, avanzamento
  a forma d'onda, testi calcolati («mancano … a domani», variazioni ▲▼). Non ancora
  realizzati come temi Bezel; da chiedere se servono anche le versioni 3,5″.

## Aggiornamento 20 ottobre 2026

- **Transizioni profili**: chiuse le richieste. Coperture opache al posto di
  Fade/Slide/Flip, funzionano anche sopra i video del dispositivo; 16 ms per
  fotogramma; ▶ prova verso la voce selezionata e torna dopo 3 s. Dialogo rifatto.
  Dettagli in `.jdi/phases/profiles-dialog-ux/SUMMARY.md`. Prova visiva aperta.
- **Avvio 8,8, indagine**: i primi fotogrammi dei sette video interni (copie
  locali identiche in `Desktop/TURZX-V3.1.0-ENG/video/4801920`) NON corrispondono
  alla foto rosa/azzurra TURZX: probabile schermata di serie del firmware.
  Il «video del template» dopo il riavvio era quasi certamente `earth.mp4` del
  tema Midnight rilanciato da Bezel, quindi la prova del 19 non dice cosa fa il
  firmware da solo con start mode 2 senza SD. In `TURZX.exe` (stringhe cifrate
  XOR fra i due byte di ogni carattere, flusso offuscato) esistono `SelectBootLogo`,
  `SelectBootVideo`, `StrPathBootVideo`, pulsanti `SwitchLogoBtn`/`BootVideoButton`:
  comando e famiglia di dispositivi ancora ignoti. Il catalogo alle 19:50 dice
  avvio = `bezel_loading_88.mp4`, start mode 2, dopo il ripristino delle 19:39:
  stato reale dello schermo non verificabile. Prossimo passo concordato con
  l'utente solo quando lo chiede: prova senza Bezel (Default → riavvio; video
  interno → riavvio; eventuale copia `00_bezel_loading.mp4` per l'ordine per nome).

Leggere prima AGENTS.md, questo documento, STATE.md e PROJECT.md.
Repository corrente `F:\Dev\bezel`, branch `windows-improvements`. Il vecchio
HANDOFF object-rotation riguarda un worktree storico, fase già integrata
(2e813e7): non riprendere da quelle istruzioni operative.

## Priorità: avvio personalizzato dell'8,8 dalla memoria interna

L'utente ricorda correttamente che il display ha flash interna, già utilizzata
per i video. NON è dimostrato che occorra una SD: la precedente risposta troppo
categorica è stata corretta. Upload e playback interno funzionano; manca la
selezione autonoma della nuova clip al boot.

Ultima foto utente: `C:\Users\danie\Desktop\20261019_194043.jpg`. Immagine
astratta rosa/azzurra, TURZX in alto a destra. Sequenza riferita: prima una
schermata, poi questa foto, poi il video del template. La foto non dimostra
origine, percorso o comando di selezione. Distinguere logo boot, attesa firmware
e video riattivato dal PC. Non promettere la sostituzione finché non verificata.

## Prove fisiche concluse

- 8,8 COM6: Rev C seriale 0525:a4a7, MCU COM4 1a86:ca88 CT88INCH;
  firmware osservato `chs_88inch.dev1_rom1.90`. Non è TURZX USB VID1CBE.
- Nessuna SD, confermata CLI/utente. Flash totale 69.132.288 B; sette video
  personali preesistenti. Nessuno cancellato, rinominato o sostituito.
- Upload `dist/bezel-start-screen/pixel/bezel_loading_88.mp4` in
  `internal/video/bezel_loading_88.mp4`, 48.589 B, dimensione verificata.
  Utente conferma riproduzione corretta e animata.
- `storage boot internal/video/bezel_loading_88.mp4 --screen COM6 --yes`, poi
  `restart --screen COM6`: nuova clip NON selezionata visivamente al riavvio.
  Il messaggio CLI «boots with» è troppo assertivo; impostare il modo video
  non prova la selezione persistente di quel file. Non inferire impossibilità
  generale dell'avvio interno dal fallimento di questo tentativo.
- Ripristinato `storage boot default --screen COM6 --yes`: startMode 0,
  standby off e timer 5 minuti. Clip conservata. Backup catalogo precedente:
  `dist/bezel-start-screen/pixel/before_apply_catalog.json`.
- 3,5 COM3: Rev A 1a86:5722 USB35INCHIPSV2, nessuno storage esposto.
  PNG verticale 320x480 centrato e loading animato dal PC confermati dall'utente.
  Tema temporaneo `dist/bezel-start-screen/pixel/probe-35`, FFmpeg host,
  194 frame/23,9 s (~8,1 invii/s, non refresh ottico). Terminato con Ctrl+C.
- Studio riaperto .55: COM3 e COM6 aperte, Present su entrambi, zero ScreenError
  nel nuovo avvio. PID osservati Studio 25532 e Light 55264, da ricontrollare.

## Grafica approvata e file locali

Preferita `dist/bezel-start-screen/pixel/`, ignorata da Git. Sfondo nero,
logo Pixel ufficiale sopra BEZEL EVO, EVO arancione, barra 24 quadratini,
sola scritta LOADING; loop 5 secondi senza percentuali inventate.
`reference.svg` è il riferimento dell'utente; `bezel_loading.svg` incorpora IBM
Plex Mono locale. PNG/GIF/MP4 orizzontali e MP4 nativo 480x1920 ruotato 90°
orario, orientamento fisico confermato. Generatori `render_reference.mjs` e
`build_pixel_loop.py`. Asset 3,5 `bezel_loading_35.png/.gif`; video in probe-35.
Variante precedente conservata nel padre, non confonderla con quella approvata.
Log delle prove `target/start-screen-*.log`.

## Indagine da continuare

1. Identificare come ROM1.90 seleziona la foto TURZX e gli eventuali media interni
   autonomi: comandi/percorso del protocollo seriale, SDK o cattura app vendor.
2. Libre Panel usa `usr/data/boot.jpg` e `usr/data/standby.h264`, comandi
   40/98/110/125, per usb-turing VID1CBE. Non inviare quei comandi alla diversa
   famiglia seriale assumendo compatibilità. Le prove SD storiche non escludono
   un percorso interno differente.
3. Snapshot Libre `target/research/libre-panel`; app vendor offuscata
   `C:\Users\danie\Desktop\TURZX-V3.1.0-ENG\TURZX.exe`, solo analisi statica
   finora. Firmware MIPS turzx_88inch_0015 NON è ROM1.90 seriale, non caricarlo.
4. Non forzare l'ordine dei video personali: diversi non hanno copie locali.
   Nessun flashing o comando firmware sconosciuto è stato eseguito.

Rapporti: `todos/2026-10-19-prove-loading-display.md`,
`todos/2026-10-19-avvio-88-revc.md`,
`todos/2026-10-19-schermata-attesa-bezel-evo.md`.
Protocollo `docs/reverse-engineering/protocol-turing-rev-c.md`, sezione 19.

## Richieste ancora NON implementate

- Schermo → Avvio e attesa per display, anteprima, scelta contenuto e Applica,
  capacità reali del modello; separare attesa host e boot residente.
- Transizioni profili moderne Copertura/Rivelazione/Tendina a scena ferma.
  Chiusura → attesa primo frame pronto → apertura, caricamento asincrono limitato.
  Service::tick oggi carica sincronicamente e Animation.started precede il
  caricamento/primo render: può consumare il tempo visibile dell'effetto.
  Oggi None/Fade/Slide/Flip; i video nativi saltano la transizione host.
- Audit verde 10 runtime profili, 6 renderer cards, 4 e2e profili; non prova
  visiva hardware. Automazioni sospese sul display in modifica: nascondere
  Studio per verificarle. Report `todos/2026-10-19-transizioni-attesa-review.md`.

## Stato prodotto e vincoli

- Sviluppo 0.1.55, installata 0.1.45. Nessuna nuova build/setup/push/tag/release
  per il loading. .55 mapping display/MCU persistente e profilo base Default;
  .54 selettore sensori condiviso/ricercabile e Inerzia; .53 rotazione da sensori.
- Checklist `todos/2026-10-19-ripresa-0.1.52.md`: standby completo/riaggancio,
  sensori reali, inerzia/rotazione manuale, DoD fase20 e /jdi-verify aperti.
  H264 fisico NON validato: nessun 1CBE presente; 50 invii/s sparsi su Rev C
  non equivalgono a streaming H264 o refresh ottico.
- Preservare documenti Studio, chiedere salvataggio/chiusura prima di accesso
  esclusivo. Mai terminazione forzata. La CLI storage apre direttamente la porta.
  Stop Light cooperativo: file `bezel-light-quit` sotto
  `%LOCALAPPDATA%\io.github.slipalison.bezel\runtime\2542278d3672e447`.
- Log in `%LOCALAPPDATA%\io.github.slipalison.bezel\logs`; catalogo storage in
  `%APPDATA%\bezel\storage\catalog.json`, impostazioni app separate sotto
  `%APPDATA%\io.github.slipalison.bezel`. Non sovrascrivere col vecchio backup.
- Build ufficiale/versione nuova per binari modificati, avvio target/release;
  setup solo se chiesto. Preservare dist. Test solo area toccata, suite completa
  prima del push/verify. Questo handoff modifica solo documentazione.
