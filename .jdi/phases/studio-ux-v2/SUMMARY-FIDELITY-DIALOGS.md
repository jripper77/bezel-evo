# Fedeltà — Profili e Galleria

22 ottobre 2026. Branch `fid/studio-dialogs`, base `357d77bc`.
Codice `f5c7653`, `7e1ea7c`.

## Realizzato

Profili: miniature contenute in riquadri 72 px e strisce nella lista, assegnazioni con miniatura e select nativo nello stesso riquadro, priorità prefissata compatta, frasi con parti evidenziate, icone e pallini delle regole. Aggiunta su sei tessere comuni (App/Musica/Orario, Sensore/Inattività/Blocco), modalità Manuale con icona. Transizioni nei pulsanti comuni con durata accanto, rientro compatto. Indicatori attivi/veri sempre dai dati runtime, senza ricavarli dalla selezione.

Galleria: intestazione e filtri compatti a sinistra, dettaglio a destra a partire dall’alto; anteprime reali contenute in strisce da 60 px, nuovo tema compatto, orientamento a icone. Dettaglio con miniatura 72 px, prova/menu comuni, distintivo sullo schermo dai dati reali, sensori con pallini e sezioni richiudibili. Duplica/Esporta/Elimina nella stessa riga del primitivo condiviso; esportazione nativa `.bezeltheme` invariata. Corrette anche tre didascalie portoghesi e alcuni accenti delle note del blocco Shell scritti dalla pipeline Windows.

`previewChoice` ora può rappresentare azioni e pulsanti disabilitati, senza un secondo stile; gli altri chiamanti mantengono le scelte a due stati. Riutilizzati accordionSection, createSectionMemory, iconChoice, prefixedField, fieldGrid, numberField, pill, iconButton, screenThumb, menu esistente e Filtri esistente.

## Verifiche

- 80/80 unitari e guardie: profiles-model, profiles-save, gallery-model, theme-previews, inspector-sections, i18n, static-guards.
- E2E profiles/themes/library/dialogs-fidelity: **124/124**, chiaro/scuro, en/pt-BR. Ulteriori **8/8** dopo le icone delle regole.
- Primo giro: 112/116; la spec del menu trovava due pulsanti omonimi ora presenti (tessera e dettaglio). Limitata al menu della tessera che il test stava già passando col mouse, conservando tutte le verifiche. La spec Transizione usa ora il gruppo di pulsanti al posto del select; persistenza, durata e direzione continuano a essere verificate.
- Nuove asserzioni: griglia 2×3 con icone e nomi accessibili, azioni prive di aria-pressed, priorità modificabile e salvata, strisce 60 px, tre azioni sulla stessa riga, eliminazione disabilitata, accessibilità e assenza di overflow.
- 8 catture italiane chiare/scure in `evidence/fidelity-dialogs/`: profilo attivo con regole vere e due schermi, pronto con regole false, modalità manuale e Galleria. La cattura espande il contenuto del dialogo per mostrare anche la parte bassa; il prodotto mantiene lo scorrimento nel viewport.
- Log in `target/logs/fidelity-dialogs/` della worktree.

## Differenze dalle tavole

- Restano tutti e cinque gli effetti del runtime (Nessuna/Sipario/Tendina/Persiane/Mosaico), non le tre etichette disegnate nella tavola: nessuna nuova semantica di transizione inventata. Conservati direzione e comando di prova già funzionanti.
- Le assegnazioni e il tipo di regola restano select nativi operabili da tastiera, con composizione compatta. I nomi degli schermi, le proporzioni delle immagini, il numero dei profili e le letture sono reali del bridge/demo, non i dati di esempio delle tavole.
- Nuovo tema mantiene la scelta verticale/orizzontale già funzionante; ordinamento dal più recente invariato. Nessun comando fittizio di ordinamento aggiunto.
- «In pausa» non è inventato per un profilo che il runtime descrive come pronto: le catture distinguono stato attivo, regole false e scelta manuale.

Nessun Rust/runtime modificato, nessuna build/installazione/push. Prossimo: galleria finale unica e registro conclusivo delle tavole, poi verifica visiva dell’utente. Le prove hardware restano fuori da queste catture.
