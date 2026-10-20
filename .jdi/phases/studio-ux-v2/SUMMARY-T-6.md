# T-6 — Nuove regole dei profili e stato del motore

Completata il 21 ottobre 2026 nella worktree `F:\dev\bezel-ux2-t6`, branch `ux2/t6`.
Base `c57e7de`, dopo T-5. Ripreso il lavoro di Claude senza modifiche pendenti; attribuzione del suo commit conservata.

## Implementazione e commit

- `8c2c148` (Claude): Sensore sopra/sotto soglia per N secondi, Orario con giorni e fascia anche oltre mezzanotte,
  Inattività e Schermo bloccato. Combinazione `match: any | all`, con `any` predefinito e omesso.
  Selezione con priorità della regola, attesa prima del rientro, stato di ogni regola, profilo attivo e motivo.
  Snapshot esteso con presenza; Windows legge ultimo input e desktop di input, solo se richiesto.
  Light e Studio usano lo stesso servizio; nuova operazione `ProfileStatus`, inoltrata dal backend Studio.
- `ec0cfb7` (ChatGPT): validazione JS delle nuove regole, riepiloghi tradotti e icone per tipo;
  simulazione pura della selezione nel demo, con lo stesso stato del nativo. Unica generazione delle letture
  condivisa fra campioni e profili del demo; presenza esplicita nei dati demo.
  Il bridge conserva configurazione e stato senza ricavare il profilo attivo dalla selezione del dialogo.
- `63eb739` (ChatGPT): fixture condivisa JSON eseguita sia dal test Rust del selettore sia dal modello JS.

La worktree era pulita al recupero: nessun codice non committato perso e nessuna riscrittura dei commit di Claude.

## Contratto per T-9b/T-9c

`bridge.profiles()` restituisce l'Overview nativo con `config`, `displays` e, se il motore è disponibile, `status`:

```js
{
  automatic: true,
  active: { profile: 'profile-id', reason: { kind: 'rules', rules: [0] } },
  rules: { 'profile-id': [true, false] }
}
```

`status.rules[id][indice]` è lo stato «vera ora» della regola dopo l'eventuale attesa.
`reason.kind`: `rules`, `returning`, `manual`, `default`; `rules` contiene gli indici coinvolti quando pertinente.
`active` è omesso quando sono attivi i temi di base. L'intero `status` può mancare se non c'è il motore:
la UI non deve inventare uno stato attivo.

Le nuove sorgenti serializzate sono `sensor`, `time`, `idle`, `locked`, oltre alle tre esistenti.
I campi facoltativi nuovi sono omessi ai valori predefiniti; vecchi profili conservano il significato e il formato.
Per Orario, 0 = lunedì e 6 = domenica; fascia da inizio incluso a fine esclusa. La parte dopo mezzanotte
appartiene al giorno di inizio. Le soglie sono strettamente sopra/sotto, non inclusive.

## Verifiche eseguite

- `cargo test -p bezel-core -p bezel-runtime -p bezel-sensors -p bezel-studio --locked`: verde.
  Con l'ultima fixture aggiunta e il nuovo controllo runtime: **327 core, 65 runtime, 76 sensors,
  229 Studio**, inclusi integrazione/doc-test pertinenti, **697 test distinti verdi**.
  Cinque test sensors già ignorati rimangono ignorati; nessuna nuova esclusione.
- `cargo clippy -p bezel-core -p bezel-runtime -p bezel-sensors -p bezel-studio --all-targets --locked -- -D warnings`:
  verde. Dopo la nuova fixture, runtime test/clippy rilanciati e verdi.
- `cargo fmt --all -- --check`: verde; nuovo test formattato con `cargo fmt -p bezel-runtime`.
- **139/139 test unitari JS mirati verdi**: `profiles-model`, `profile-selection`, `demo-backend`, bridge,
  i18n e static-guards. I due moduli del modello profili: **100% righe/funzioni**, rami complessivi **97,25%**.
- `node scripts/e2e-slot.mjs tests/e2e/profiles.spec.mjs`: **16/16 verdi**, chiaro/scuro × en/pt-BR,
  accessibilità inclusa; nessuna modifica alla spec o indebolimento delle asserzioni.
- `git diff --check`: verde. Log nella worktree: `target/logs/studio-ux-v2-t6/`.

Le fixture Rust/JS confrontano l'intero stato: soglia con attesa, uguaglianza che non supera la soglia,
lettura mancante che conserva il profilo, rientro al predefinito, fascia domenica→lunedì, combinazione di
Orario/Inattività/Blocco e modalità manuale. Altri test verificano priorità, parità risolta dall'ordine salvato,
reset dell'attesa alla modifica, vecchie regole App/Musica e inoltro IPC senza perdita di campi.

## Tavole, riuso e attività successive

Letti CONTEXT, PLAN, D-12, README e HTML offline di ProfilesNew; vista la tavola PNG.
T-6 prepara motore, modello e dati: il nuovo dialogo, il salvataggio automatico e le indicazioni visive
«attivo ora»/«vera ora» della tavola restano in **T-9c**, la barra con profilo in **T-9b**, come da piano.
Non si dichiara già realizzata la nuova UI né il suo confronto grafico.

Riusati `CardTrigger::matches` nel nativo e `triggerMatches` nel JS per App/Musica, servizio dei profili comune
a Light/Studio e generatore di letture del demo. **Nessuna duplicazione** fra Studio e Light, né del matching App/Musica.
La simulazione JS è la controparte necessaria del runtime nativo, verificata sulla stessa fixture.
I nuovi riepiloghi sono in `i18n/areas/profile-rules.js`, en/pt-BR/it a parità; basi delle traduzioni invariate.

T-10: verifica trasversale, prova a vista delle condizioni di inattività/blocco e della UI collegata da T-9,
pulizia prevista dalla fase. Non resta lavoro funzionale di T-6 rinviato a T-10.

Nessuna suite completa, build prodotto, setup, installazione, modifica di VERSION, push o pubblicazione.
