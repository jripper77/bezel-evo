# Sensori e widget modulari (to do, 30 ottobre 2026)

Richiesta dell'utente, da aprire come due fasi (`/jdi-add-phase` + `/jdi-discuss`) quando lo decide.

## Sensori: dentro Light, identificabili uno per uno

Non plugin esterni: le fonti restano compilate in Light.

- Oggi `crates/bezel-sensors/src/system.rs` compone una lista anonima di `Box<dyn Provider>`.
- Obiettivo: ogni fonte (LHM, NVIDIA, RTSS/PresentMon, ping, meteo, audio, memoria, ...) è un modulo che si
  descrive: id stabile, nome, icona, cosa misura, stato (attiva / in attesa / non disponibile e perché) e opzioni
  proprie invece del `SensorOptions` comune.
- Un registro unico: un sensore nuovo = il suo file + una riga di registrazione.
- Studio: la pagina Sensori le mostra una per una con lo stato; eventuale attivazione singola.

## Widget: sezioni dell'inspector riutilizzabili

- Oggi le sezioni comuni stanno in `ui/inspector/common.js` e il raggruppamento in `ui/inspector/sections.js`,
  ma ogni widget (es. `ui/inspector/bar.js`) compone a mano le sue.
- Obiettivo: un catalogo di sezioni registrate una volta (`source`, `transition`, `fill`, `track`, `corners`,
  `textStyle`, ...); ogni widget dichiara l'elenco delle sezioni che usa; le sezioni nuove di un widget entrano
  nel catalogo e diventano disponibili agli altri.
- Lato Rust lo stesso principio: `ElementKind` (bezel-core `domain/theme.rs`) e `bezel-render/src/renderer.rs`
  (118 KB) come moduli per widget con un registro.

## Esempio di sensore nuovo: consumo degli LLM

- Niente login al sito (cookie di sessione = credenziali, fragile, probabile violazione dei termini).
- Senza login: Claude Code salva i token di ogni messaggio in `%USERPROFILE%\.claude\projects\*.jsonl` (finestra di 5 ore e settimana, come *ccusage*); dà i token usati, non il residuo esatto del piano. Codex CLI: verificare se i file locali riportano la percentuale di limite usata.
- API a pagamento: endpoint ufficiali di consumo/costi con chiave di amministrazione, custodita nel Gestore credenziali di Windows; lettura periodica in rete come il meteo.
