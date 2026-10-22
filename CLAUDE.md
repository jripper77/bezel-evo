# JDI — Instrucoes Claude Code

## Ripresa operativa — 22 ottobre 2026 (notte)

Leggere AGENTS.md, .jdi/STATE.md e [il passaggio](.jdi/HANDOFF-CLAUDE.md).
Versioni: pubblicata e installata **0.2.0** (Latest su GitHub, tag `v0.2.0`, fase
studio-ux-v2 chiusa). Solo locale: correzione distanza copertina Player (Unreleased).
I tag di upstream stanno in `refs/tags/upstream/*` (`remote.upstream.tagOpt --no-tags`):
i `v0.x` appartengono a Bezel Evo. Push e release solo su richiesta.
Design: canvas https://claude.ai/artifact/7muP7mTQAwBH8diy2AWZ6p.
Aperti: avvio dell'8,8 dalla flash interna, nomi tagliati nelle righe Sensori e facce
Card, anteprima immagini nel demo, stato «attivo ora» non esposto dal runtime.
ChatGPT può lavorare in parallelo nello stesso repository: controllare `git log` prima
di formattare o fare commit ampi. Ripresa in `F:\Devezel`.

Este projeto usa o JDI (Just Do It) como workflow de desenvolvimento. JDI eh um workflow enxuto: 6 comandos no loop principal, 5 agents core + 2 per-project specialists.

## Loop canonico

```
/jdi-new "<descricao>"   -> research + PROJECT.md + ROADMAP.md
/jdi-bootstrap           -> cria specialists per-project (doer + reviewer)
/jdi-discuss <N>         -> captura decisoes locked da phase
/jdi-plan <N>            -> decompoe em tasks com waves
/jdi-do <N>              -> executa via doer specialist
/jdi-verify <N>          -> gates de qualidade via reviewer specialist
/jdi-ship <N>            -> finaliza phase + avanca pra proxima

# Roadmap mutation (qualquer hora)
/jdi-add-phase "<name>" [--goal "<t>"] [--at <pos>]   -> adiciona phase
/jdi-remove-phase <N> [--force]                        -> remove future/pending phase

# Continuidade / snapshot
/jdi-status                                            -> resumo: phase atual + ultima acao + proximo comando
```

`/jdi-create [desc]` gera novos agents/skills genericos no `core/` (rodado dentro do repo JDI, nao de projetos consumindo).

## Agents core (em `.claude/agents/`)

Genericos, shipped pelo JDI:

| Agent | Modelo | Funcao |
|---|---|---|
| `jdi-researcher` | Opus | Research pre-roadmap. Le ideia, pergunta, gera PROJECT.md + ROADMAP.md |
| `jdi-bootstrap` | Sonnet | Dispara architect modo specialist pra gerar per-project doer + reviewer |
| `jdi-asker` | Sonnet | Loop adaptativo de perguntas pra capturar decisoes locked (CONTEXT.md) |
| `jdi-planner` | Opus | Decompoe phase em tasks, agrupa em waves de paralelismo (PLAN.md) |
| `jdi-architect` | Opus | Meta-agent. 2 modos: cria agents/skills genericos no core/ OU cria specialists per-project |

## Specialists per-project (em `.jdi/agents/`)

Gerados pelo `/jdi-bootstrap` baseado em PROJECT.md:

| Agent | Funcao |
|---|---|
| `jdi-doer-{slug}` | Executor que ja conhece stack/code-design/conventions do projeto. Sem descoberta, ja sabe |
| `jdi-reviewer-{slug}` | Roda gates de qualidade definidos pra stack: build, tests, coverage, lint, security |

Routing em `.jdi/specialists.md` e `.jdi/reviewers.md`.

## Memoria — files em `.jdi/`

```
.jdi/
  PROJECT.md          <- visao + stack + code-design locked
  ROADMAP.md          <- phases + status
  DECISIONS.md        <- D-XX append-only (decisoes locked)
  STATE.md            <- current_phase + next_step
  specialists.md      <- routing pro doer
  reviewers.md        <- routing pro reviewer
  registry.md         <- audit trail dos specialists criados
  agents/             <- per-project specialists
    jdi-doer-{slug}.md
    jdi-reviewer-{slug}.md
  phases/{NN-slug}/
    CONTEXT.md        <- output do asker
    PLAN.md           <- output do planner
    SUMMARY.md        <- output do doer
    REVIEW.md         <- output do reviewer
```

## Convencoes

- Conventional Commits — scope = phase slug, ex: `feat(01-setup-api): ...`
- Atomic commits — 1 task = 1 commit
- 80% cobertura minima (overridable via PROJECT.md)
- Code design locked uma vez no `/jdi-new`, nunca muda
- D-XX referenciado em commit message quando aplicavel

## Idioma

- Codice, commit, PR: inglese
- Discussione con l'utente e documenti in `.jdi/` (decisioni, piani, riepiloghi, report): italiano
- i18n nel frontend: mai stringhe fisse nell'interfaccia; testi in en, pt-BR e it con parità di chiavi

## Regole del progetto

- Piattaforma: solo Windows è supportato (D-2026-10-16-studio-redesign-fidelity-27). Il codice Linux ereditato non si compila, non si testa e non si mantiene: se intralcia lo sviluppo si può togliere. Nessuna verifica automatica su GitHub (ci.yml rimosso); l'installer Windows si costruisce a mano (windows-evo-package.yml).
- Test: si testa solo ciò che si tocca (crate, moduli UI e spec e2e dell'area cambiata); suite completa e copertura una sola volta, prima di pubblicare (push) o alla verifica della phase. Un test instabile si riesegue da solo.
- Commit: ogni commit termina con il trailer dell'agente che l'ha scritto: `Co-Authored-By: ChatGPT <noreply@openai.com>` per il lavoro di ChatGPT, `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>` per quello di Claude.
- Regole operative condivise con gli altri agenti (versioni, `dist/`, build Windows): `AGENTS.md`.

## Prioridade quando conflita

1. Seguranca
2. Performance
3. Boas praticas

## Hooks (opcional)

`.githooks/pre-commit` e `post-commit` shipped. Pra ativar:

```bash
git config core.hooksPath .githooks
```

Windows: hooks rodam via Git Bash (vem com Git for Windows). Sem ele, hooks sao silenciosamente ignorados.
