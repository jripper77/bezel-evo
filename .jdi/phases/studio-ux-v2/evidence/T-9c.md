# Confronto T-9c — Profili e Temi

Confronto del 21 ottobre 2026. L'approvazione visiva finale resta in T-10. Il pulsante di esportazione usa `.bezeltheme`, come confermato dall'utente.

| Tavola | Riferimento offline | App scura | App chiara |
| --- | --- | --- | --- |
| ProfilesNew | [PNG](../design/png/ProfilesNew.png) · [HTML acquisito](t9c-profiles/dark/mock/ProfilesNew.png) | [dialogo](t9c-profiles/dark/app/library/ux2-profiles-detail.png) | [dialogo](t9c-profiles/light/app/library/ux2-profiles-detail.png) |
| ThemesNew | [PNG](../design/png/ThemesNew.png) · [HTML acquisito](t9c-themes/dark/mock/ThemesNew.png) | [dialogo](t9c-themes/dark/app/library/ux2-themes-detail.png) | [dialogo](t9c-themes/light/app/library/ux2-themes-detail.png) |

Ricette: `ux2-profiles` e `ux2-themes` nell'area `library`. Il mock usa `boards/index.json`, senza cambiare i file di design. Il mock resta scuro perché il suo HTML ha colori fissi; l'app usa i token in entrambi gli schemi. I dati sono quelli del demo e le anteprime PNG passano per `demo-render`; nessun dato d'esempio è stato inserito nel prodotto.

Rigenerare da `apps/bezel-studio`:

```powershell
node scripts/fidelity/shoot.mjs --area library --artboard ProfilesNew --state ux2-profiles --scheme both --out ../../.jdi/phases/studio-ux-v2/evidence/t9c-profiles
node scripts/fidelity/shoot.mjs --area library --artboard ThemesNew --state ux2-themes --scheme both --out ../../.jdi/phases/studio-ux-v2/evidence/t9c-themes
```

Le differenze e le note di verifica sono in [SUMMARY-T-9c](../SUMMARY-T-9c.md).
