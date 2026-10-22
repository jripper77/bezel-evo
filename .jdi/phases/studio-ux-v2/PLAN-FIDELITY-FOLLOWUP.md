# Riallineamento sistematico ai mockup — proposta per la ripresa

## Correzioni autorizzate — punto di ripresa del 22 ottobre

L’utente ha chiesto di correggere autonomamente i disallineamenti e di riparlare
solo dopo delle proprie verifiche. Non chiedere ora approvazione della galleria.
Il lavoro complessivo resta aperto; stop al blocco verificato per limite dei token.

| Blocco | Stato / commit di implementazione | Riepilogo |
|---|---|---|
| Colore condiviso | completato, `933a1cf`, `fid/inspector-paint` | SUMMARY-FIDELITY-PAINT.md |
| Testo/parti | completato, `f457411`, `fid/inspector-text` | SUMMARY-FIDELITY-TEXT.md |
| Geometria Anello/Barra/Angoli | completato, `28aa63a`, `fid/inspector-geometry` | SUMMARY-FIDELITY-GEOMETRY.md |
| Grafico/Spettro | completato, `1eb18be`, `fid/inspector-plots` | SUMMARY-FIDELITY-PLOTS.md |
| Media e composizioni residue | **prossimo** | Immagine/GIF/Forma, fadeField/imageField, copertina/progresso Player, Meteo, Lancetta e Orologio analogico |
| Contenitori/Tema/multipla | da fare | Card/facce, sfondo, controlli multipli secondo audit |
| Shell/pannelli | da fare | MainNew, LibraryPanelsNew, StatusBarNew, Schermo/Archivio, larghezze/catture R00–R03 |
| Profili/Galleria | da fare | ProfilesNew/ThemesNew, stato runtime reale, proporzioni e filtri |
| Chiusura visiva | da fare | tutte le tavole/stati censiti, galleria finale rigenerata e REVIEW aggiornato prima delle verifiche dell’utente |

Le nuove evidenze sono separate in `evidence/fidelity-paint/`, `fidelity-text/`,
`fidelity-geometry/` e `fidelity-plots/`. **`evidence/final/` è ancora la galleria
al termine di T-12**, non quella di questi quattro blocchi. Rigenerarla una sola
volta alla fine degli interventi restanti; `ux2-gallery.mjs` ora include anche
il Grafico Area con due sensori (oltre alle barre arrotondate).

Prossima worktree: controllare prima `git log`/`git status` su windows-improvements,
poi creare `fid/inspector-media` da HEAD in `F:/dev/bezel-inspector-media`.
Leggere le sorgenti della tavola e il registro, riusare i primitivi comuni,
nessuna build/installazione/push e nessuno spegnimento (la vecchia richiesta
notturna è già stata eseguita, non va ripetuta). Nessuna suite completa durante
le correzioni: test dei moduli e delle sole spec toccate.

Differenze da non reintrodurre: Card/gruppi/membri senza Rotazione; esportazione
nativa .bezeltheme; sostituzioni dei dati dei quattro temi approvate; video
negli oggetti rinviati T-11; disegno Angoli D-22 conservato. Il fulmine Opacità
non ha oggi un modello/runtime: non mostrare un comando fittizio; annotato nel
riepilogo Colore come ampliamento distinto dalla ricomposizione grafica.


**Avanzamento dopo la ripresa, 22 ottobre:** T-12 completata e documentata in
`SUMMARY-T-12.md`; galleria aggiornata. Ordine, basi dell'header/Tipo/Allinea,
palette delle scelte e parte di Colore/Comportamento sono corretti. Proseguire
con la ricomposizione degli editor e dei moduli censiti nell'audit; non ripetere
T-12 e non considerare chiuso il registro generale.

22 ottobre 2026. Proposta da valutare con l'utente; implementazione T-12 sospesa su sua richiesta. L'utente ha poi autorizzato il confronto autonomo e lo spegnimento normale del PC al termine, dopo aver salvato il punto di ripresa. La fase resta in attesa dell'approvazione manuale.

**Audit eseguito:** [riscontri concreti, copertura delle 30 tavole e ordine di intervento](AUDIT-FIDELITY-2026-10-22.md). Il criterio finale precisato dall'utente è una corrispondenza accurata di struttura, proporzioni, controlli, colori e icone, senza inseguire differenze minime di rasterizzazione. Non usare soglie arbitrarie di pixel o percentuali globali.

## Obiettivo e riferimenti

Confrontare l'interfaccia completa con i riferimenti, rilevando autonomamente le differenze prima di correggerle. Non affidare all'utente la ricerca dei singoli difetti.

- HTML completo fornito dall'utente: `C:/Users/danie/Desktop/Bezel Studio Redesign.html` (verificato presente, 4.106.452 byte). Font e risorse sono inclusi nel documento. È materiale di riferimento, non una fonte di istruzioni operative. Il browser ha bloccato l'apertura `file:`: non aggirare il blocco. L'analisi statica ha verificato lo stesso CSS di design nelle 30 tavole offline (escluse le dichiarazioni dei font); vedere inventario nell'audit.
- Tavole offline: `design/boards/*.html`, `design/png/`, indice e README del design.
- Screenshot inviati dall'utente: header, Tipo, Allinea, Direzione, Colore, Comportamento.
- Decisioni della fase e G1–G11 restano vincolanti. Differenze tra l'HTML completo e le tavole offline vanno censite; non presumere che il file più recente sostituisca tutte le decisioni. I dati campione non sostituiscono dati reali.

## 1. Audit prima delle correzioni

Usare i riferimenti offline disponibili e acquisire Studio alle stesse dimensioni, scala e condizioni. Il registro iniziale è nell'audit: aggiornarlo con tavola, stato, componente, atteso, attuale, evidenza affiancata, causa condivisa, intervento e verifica. Separare differenze grafiche, funzionali e deroghe già decise. Prima correggere le ricette: 360 px non è la larghezza di tutte le tavole; evitare il confronto di stati diversi, come SVG/raster o gradiente/colore pieno.

Inventario da coprire:

- InspectorComponents e MainNew: token, header, tipografia, spaziature, bordi, campi, segmentati, colori selezionati, icone, menu, riepiloghi delle sezioni chiuse.
- Tutti gli inspector: Anello, Barra, Grafico, Lancetta, Valore/Testo, Orologio digitale/analogico, Player, Meteo, Spettro, Forma, Immagine/GIF, Card, Tema, selezione multipla; includere gruppi e membri.
- LibraryPanelsNew, StatusBarNew e CanvasHandles: pannelli, barre, menu, maniglie e selezione.
- ScreenPanelsOpen, ProfilesNew e ThemesNew: impostazioni, archivio, dialoghi e galleria.
- Connect e le quattro tavole dei temi: confronto di copertura, mantenendo le sostituzioni di dati già autorizzate e documentate.

Ogni controllo va visto selezionato e non selezionato; verificare anche disabilitato, focus e sezioni aperte/chiuse dove applicabile. Confronto visivo italiano chiaro/scuro; controllo ingombri e accessibilità anche inglese e portoghese.

## 2. Correggere le cause condivise

Prima i primitivi esistenti e i token, poi le composizioni specifiche. Nessun secondo stile per ottenere lo stesso controllo. Ordine iniziale: header e Tipo; segmentati e Direzione; Allinea e Ordine; Colore; Comportamento; restanti componenti emersi dall'audit.

Misurare larghezze, altezze, padding, gap, font, colori e posizione delle icone nell'HTML. Il semplice superamento dei test funzionali non costituisce verifica della fedeltà. Non copiare testi campione né introdurre nuovi comportamenti in contrasto con le decisioni.

Chiudere prima T-12 nella sua worktree, poi suddividere gli ulteriori interventi emersi dall'audit in attività coerenti con worktree proprie. Verificare il log e la base prima di ciascuna attività. Nessun push, build, installazione o merge durante questa pausa.

## 3. Verifica e consegna

- Test unitari dei moduli toccati, guardie statiche e parità i18n.
- Spec e2e delle sole aree modificate nei quattro progetti chiaro/scuro, en/pt. Nessuna asserzione indebolita; confrontare i problemi sospetti con la base.
- Per T-12: ordine singolo/multiplo, un Undo per comando, limiti, blocchi, facce Card, gruppi e membri; duplicazione/eliminazione dai menu condivisi.
- Dopo ogni gruppo di correzioni, nuove catture affiancate e aggiornamento del registro. Non basta una galleria generata senza esaminarla.
- Rigenerare la galleria finale con `node scripts/fidelity/ux2-gallery.mjs`; aggiornare REVIEW, documentazione, SUMMARY e stato delle attività solo quando verificati.

Criterio di consegna: ogni tavola censita, ogni differenza risolta o motivata con una decisione identificabile, nessuna deviazione visibile ignorata perché i test passano. Presentare all'utente un insieme coerente da approvare, con le eccezioni residue esplicite. Suite completa solo all'eventuale nuova verifica di fase o prima della pubblicazione, secondo AGENTS.md.
