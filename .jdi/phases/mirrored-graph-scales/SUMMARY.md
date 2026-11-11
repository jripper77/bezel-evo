# mirrored-graph-scales — riepilogo

Fatto. Core: `Visualization::secondary_autoscale` e `shared_scale`, con `lower_autoscale(upper)`. DTO `bezel-themes`: `secondaryAutoscale` e `sharedScale` scritti solo se diversi dal predefinito. Renderer: picco comune come prima, altrimenti ogni serie automatica ha il suo picco dalla base zero; la serie fissa usa il suo min/max.

Studio: nella sezione «Sensore della serie inferiore» ci sono Automatica/Fissa, min/max (disattivati in automatico, con l'unità del sensore), l'interruttore «Stessa scala per le due serie» e il picco attuale della serie inferiore. Scegliere il sensore di una delle due serie imposta la scala comune solo se le unità coincidono. Il controllo del picco ora segue due sensori insieme. Il demo non cambia (disegna frazioni, senza scala).

Verifiche: test di render 152 (nuovo `mirrored_series_share_one_peak_or_scale_on_their_own`), themes e core completi, clippy `-D warnings` e fmt su core/render/themes, UI element-preview + graph-inspector 18, i18n 35, e2e visualizations (esteso) + inspector-linked/plots/preview/sections 80.

Da provare a mano: un grafico specchiato rete giù/su con scala propria sullo schermo vero.

## Dopo la prova dell'utente (0.2.27, 0.2.28)

- 0.2.27: pannello riorganizzato su richiesta («non si capisce»). Sorgente con «Serie superiore» e «Serie inferiore»; Scala verticale con un blocco per serie e l'interruttore (icona collegamento) sotto. Cambiare la scala di una serie non sposta più l'altra (la serie inferiore viene fissata al valore che aveva).
- 0.2.28: in automatico min/max in grigio mostrano le letture della finestra, della serie o di tutte e due se collegate; nello specchio il min resta 0 (le barre partono dalla linea centrale, come nel renderer). Il picco comune legge entrambi i sensori (`createPeakWatch().range`).
- Prima della pubblicazione: il test di confine `window-boundary` falliva per il pulsante «Apri la pagina di download» di update-check in `preferences.js`; aggiunto ai moduli che possono aprire un link (solo la pagina fissa `releases`).
