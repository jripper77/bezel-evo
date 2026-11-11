# mirrored-graph-scales — riepilogo

Fatto. Core: `Visualization::secondary_autoscale` e `shared_scale`, con `lower_autoscale(upper)`. DTO `bezel-themes`: `secondaryAutoscale` e `sharedScale` scritti solo se diversi dal predefinito. Renderer: picco comune come prima, altrimenti ogni serie automatica ha il suo picco dalla base zero; la serie fissa usa il suo min/max.

Studio: nella sezione «Sensore della serie inferiore» ci sono Automatica/Fissa, min/max (disattivati in automatico, con l'unità del sensore), l'interruttore «Stessa scala per le due serie» e il picco attuale della serie inferiore. Scegliere il sensore di una delle due serie imposta la scala comune solo se le unità coincidono. Il controllo del picco ora segue due sensori insieme. Il demo non cambia (disegna frazioni, senza scala).

Verifiche: test di render 152 (nuovo `mirrored_series_share_one_peak_or_scale_on_their_own`), themes e core completi, clippy `-D warnings` e fmt su core/render/themes, UI element-preview + graph-inspector 18, i18n 35, e2e visualizations (esteso) + inspector-linked/plots/preview/sections 80.

Da provare a mano: un grafico specchiato rete giù/su con scala propria sullo schermo vero.
