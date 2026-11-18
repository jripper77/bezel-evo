# Quote Pixel e segnalazione cambio stato

Richiesta: barre agenti coerenti con Pixel, rosse vicino a esaurimento;
lampeggio di due secondi sul riquadro che cambia stato.

Implementazione 0.2.33:
- Opzione Barre quote Pixel: segmenti quadrati arancioni, passo vicino a 10px,
  gap 3px, altezza 5px e traccia #222220. Numero segmenti adattivo alla larghezza.
- Soglia 20% residuo: barra e relativa etichetta rosse; dati scaduti o non
  finiti restano indisponibili. Soglia attiva anche sulle barre continue.
- Opzione Lampeggia ai cambi di stato: bordo arancione e sfondo tenue sul solo
  widget interessato, quattro impulsi in due secondi (250ms acceso/spento).
  Il testo rimane sempre visibile; nessun effetto al caricamento, comparsa di
  nuovi progetti o aggiornamenti di messaggi/quote. Movimento disattivato
  sopprime l'effetto. Cambio sorgente e riavvolgimento del clock azzerano il
  confronto. Rilevata anche conclusione con permanenza conclusi impostata a zero.
- Stato mantenuto per scena/widget e percorso progetto normalizzato, indipendente
  dalle altre istanze. Pagine e allarmi non riutilizzano superfici statiche nelle card.
- Campi serializzati pixelStyle/blinkOnChange, default falsi per compatibilita.
  Nuovi widget dedicati attivano il lampeggio; tema personale attiva entrambe le opzioni.
- Traduzioni en/it/pt-BR e controlli Dettagli. Il demo mostra le tracce segmentate;
  transizioni verificate con il renderer nativo.

Verifiche: test e clippy dei tre crate toccati, 56 test UI mirati e 12 browser;
prove pixel sui segmenti, soglia 20%, dati scaduti, isolamento dei due riquadri,
fasi e termine dell'effetto. Anteprime reali e simulazione esplicitamente nominata
in target/logs/agent-pixel-alerts/. Non si modifica il dato delle sessioni sul disco.
Build 0.2.33 completata. Setup, ZIP e SHA256 pronti e verificati: 175 file,
eseguibili del pacchetto uguali alla build. Installata dopo conferma di salvataggio/chiusura: setup silenzioso terminato
con codice 0, eseguibili in LOCALAPPDATA e dist/bezel uguali alla build. Tema
aggiornato rileggendo la copia appena salvata e modificando soltanto pixelStyle
e blinkOnChange dei due widget: disposizione, impostazioni e profilo preservati.
Studio riaperto, anteprima installata verificata con renderer nativo. Backup: F:\Dev\bezel\target\scratch\agent-pixel-alerts\backup-20261116-153746.
Manifest in target/logs/agent-pixel-alerts/installed-0.2.33.json. Nessun push o pubblicazione.
