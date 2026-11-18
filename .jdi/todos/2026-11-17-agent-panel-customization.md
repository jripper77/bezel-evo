# Pannelli Codex/Claude Code: dettagli e personalizzazione

Richiesta: più dettagli e possibilità di personalizzazione dei pannelli agenti.
Scelte dell'utente: tutte e quattro le aree proposte.

Implementazione (non ancora in una build, versione invariata 0.2.35):
- Colori facoltativi per widget: Al lavoro, Tocca a te/errore, Finito
  (separabile; senza scelta segue il colore di attesa), Stato sconosciuto,
  barra quote, fondo barra, avviso quote. Campi `workingColor`, `attentionColor`,
  `completedColor`, `unknownColor`, `quotaColor`, `quotaTrackColor`,
  `warningColor`, scritti solo se impostati: i temi esistenti mantengono la
  tavolozza integrata. Pulsante «Ripristina colori predefiniti».
- Soglia avviso quote `warningPercent` 0-50 (default 20, 0 disattiva).
- Intestazione: titolo personalizzato `title` (max 64 caratteri, vuoto = nome
  dell'agente, vale anche in modalità sessione singola), `showHeader`, `showPage`.
- Righe progetto: `showStateText` (solo pallino se spento), `showAge` (tempo
  dall'ultimo aggiornamento), `showRowActivity` (strumento in uso solo durante
  il lavoro), `showDividers`. I nuovi widget Codex/Claude mostrano l'età.
- Età compatta: «45s», «12m», poi «2h 5m» (prima «0h 12m»).
- Inspector: sezioni Intestazione e Righe progetto nel gruppo Forma, colori nel
  gruppo Colore. Traduzioni en/it/pt-BR; titolo riusa `insp.partTitle` (D-14).
- Anteprima demo: titolo, colore fondo barre e pallino di stato.

Verifiche: test bezel-core/bezel-themes/bezel-render (nuovi test di dominio,
DTO con compatibilità e validazione, prova pixel nativa su colori, soglia e
intestazione nascosta), clippy dei tre crate con warning negati, fmt; 77 test
UI mirati, 16 prove browser dell'area agenti. Nessuna build, installazione,
commit o push.

## Consegna 0.2.36

Build con package-evo.ps1 (log in target/logs/agent-panel-customization/package.log):
setup, ZIP e SHA256 in dist/, hash del setup uguale a SHA256SUMS. Studio e Light
erano già chiusi: setup silenzioso con codice 0, eseguibili in LOCALAPPDATA e
dist/bezel uguali alla build. Studio riaperto dall'installazione. Tema personale
non modificato. Nessun commit, push o pubblicazione.

## Ridisegno dei Dettagli (dopo prova utente 0.2.36)

Feedback: pannello disordinato, residui non controllabili, colori tutti in fila.
- Nuovo campo `quotaValue` (remaining/used/both, facoltativo): senza scelta resta
  il comportamento storico (una riga = residuo, due righe = entrambi). Il valore
  scelto guida anche il riempimento della barra.
- Dati: sorgente (minuti e durata pagina affiancati), Progetti mostrati, Quote
  (aggiornamento Claude e nota).
- Forma: Pannello (titolo, intestazione, pagina; in sessione singola solo stato,
  messaggio, attività, stato, righe), Righe progetto, Quote (5h, settimana,
  ripristino, Valore mostrato, Disposizione una/due righe, Barra continua/Pixel,
  dimensione e soglia affiancate).
- Colore: griglia 2x2 Colori di stato e griglia 3 colonne Quote, ripristino
  solo quando c'è un colore personalizzato.
- Comportamento: Cambi di stato con l'evidenziazione.
Etichette riusate dove già canoniche (`widget.bar`, `insp.patternContinuous`).
Catture prima/dopo in target/logs/agent-details-redesign/.
Verifiche: test e clippy dei tre crate, fmt, 57 test UI mirati, 16 prove browser.
Consegna 0.2.37: pacchetto e SHA256 verificati; Light chiuso con bezel-light-quit
(nessuna terminazione forzata), setup silenzioso codice 0, eseguibili installati e
dist/bezel uguali alla build; Light e Studio riavviati dall'installazione.
Tema personale non modificato. Nessun commit, push o pubblicazione.

## Parti selezionabili (18 novembre, 0.2.38)

Design sul canvas (pagina Attuale): tavole «Dettagli ridisegnati», «alternativa a
gruppo» e «clic sulle parti». Scelta dell'utente: un solo oggetto con parti
selezionabili, senza quote come sensori.
- Clic sul widget: Dati, gruppo nuovo «Parti» (dopo Dati, aperto di serie) con
  occhio e freccia per parte, Testo di base, Comportamento (evidenzia cambi).
- Doppio clic su una parte del display o freccia: l'inspector mostra solo quella
  parte, con ‹ e briciola per tornare; Esc torna al widget, il secondo Esc
  deseleziona. Riquadro arancione con etichetta sul canvas. Stato solo nello
  Studio (editor/agent-part.js), mai nel tema; si azzera cambiando selezione.
- Parti: Titolo, Righe progetto, Quota 5 ore, Quota settimana (le due quote
  aprono lo stesso dettaglio condiviso); vista a sessione singola: Contenuto e quote.
- Stili per parte con il sistema esistente TextParts: agentTitle, agentName,
  agentState, agentQuota (carattere, peso, colore, dimensione). Il colore di
  avviso delle quote prevale sul colore della parte.
- Nuovi campi: showDot, showPath, stateBelow, rowHeight (8-400), barHeight (1-24),
  segmentGap (0-12); default identici al disegno precedente.
- Geometria delle parti in JS speculare al renderer (agentPartRects).
Verifiche: test e clippy dei tre crate (nuovo test pixel sulle parti), 9 test
UI agenti + 63 mirati, 16 prove browser agenti e 140 di inspector/canvas/libreria.
Catture in target/logs/agent-details-redesign/parts-*.png.
Consegna 0.2.38: pacchetto e SHA256 verificati, Light chiuso con bezel-light-quit, setup silenzioso codice 0, eseguibili installati e dist/bezel uguali alla build, Light e Studio riavviati. Tema personale non modificato. Nessun commit, push o pubblicazione.

## Parti spostabili e grafica delle tavole (18 novembre, 0.2.39)

- Ogni parte (titolo, righe, quota 5 ore, quota settimana, contenuto) può avere
  un posto proprio, in frazioni del widget (campo `layout`, scritto solo se usato);
  le altre restano nella disposizione automatica, che non conta le quote spostate.
- Canvas: con una parte aperta le maniglie del widget lasciano posto a quelle
  della parte; trascinare dentro la parte la sposta, le maniglie la ridimensionano,
  sempre confinata nel widget e in un solo passo di annulla; clic su un'altra
  parte dello stesso widget la apre. Inspector: «Posizione nel widget» (X, Y, L, A
  in px) con «Torna al posto automatico»; nel widget «Disposizione automatica per
  tutte le parti».
- Inspector rifatto sui componenti delle tavole: vista a icone, progetti come
  chip con pallino di stato, campi compatti con prefisso, righe delle parti con
  icona/riepilogo/freccia/occhio, «In ogni riga» a riquadri, scelte a icone per
  stato, righe e barra, campioni colore con nome, intestazione della parte con percorso.
- Corretti gli accenti persi in sei stringhe pt-BR degli agenti (sessão, até, indisponível).
Verifiche: test e clippy dei tre crate (nuovo test pixel sul posto a mano),
84 test UI mirati, 20 prove browser agenti, 88 di inspector e canvas.
Consegna 0.2.39: pacchetto e SHA256 verificati, Studio e Light già chiusi, setup silenzioso codice 0, eseguibili installati e dist/bezel uguali alla build, Light e Studio avviati. Tema personale non modificato. Nessun commit, push o pubblicazione.
