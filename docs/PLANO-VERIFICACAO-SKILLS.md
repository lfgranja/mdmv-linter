# Plano de Verificação de Skills — B1–B4

> **Status:** pendente (registro de trabalho futuro)
> **Origem:** curadoria do OpenCode da MDMV (2026-09-22/23) — relatório
> `MDMV/documentos/opencode-configuracao-ideal-2026-09-22.md` (§5.6 e §7).
> **Princípio:** [`OBJETIVO.md`](../OBJETIVO.md) — *"sem teste de trigger, não publica"* e
> *"gate determinístico > prosa; make the environment smarter, not the model"*.

---

## 1. Contexto

A curadoria do pool de skills do OpenCode da MDMV foi **aplicada**, mas **não medida**:

- Pool reduzido de **116 → 16** skills (todas em `~/.agents/skills/`).
- Metadados residentes: **~8.571 → ~1.131 tokens** (−87%).
- Zero plugins, zero declarações `globs`, zero agentes oh-my-openagent.
- Config global: `~/.config/opencode/opencode.json` (`plugin: []`, `instructions` de 2 arquivos,
  `tool_output`/`compaction` explícitos).

A curadoria seguiu um critério, mas **o critério não foi verificado**. B1–B4 são os quatro itens
que transformam a curadoria num **contrato verificável**, e devem ser executados **neste projeto
(`mdmv-linter`)**, onde nasce o `verify.sh` e o gate determinístico.

### Estado de partida (verificável)

| Item | Estado atual |
|---|---|
| Skills curadas | 16 em `~/.agents/skills/` (`opencode debug skill` → 17, incl. 1 built-in) |
| `globs` no pool | 0 |
| Gate de CI de skills | **não existe** (sem `.github/workflows`) |
| Harness de trigger-test | **não existe** |
| Modelo de referência | `opencode-go/deepseek-v4.1-flash` (V1, OpenCode 1.18.32) |

---

## 2. B1 — Harness de trigger-test

**Objetivo:** provar que cada skill curada é selecionada quando relevante e ignorada quando não é.

**Método (por skill):**
1. Montar **10 prompts rotulados**: 5 que **devem** acionar a skill e 5 que **não devem**.
2. Rodar headless: `opencode run --format json "<prompt>"` (verificar seleção da skill no log/saída).
3. Medir a **taxa de seleção correta** = (deve-aciona + não-deve-ignora) / 10.

**Entregáveis:**
- `scripts/trigger-test/` — cenários (pool de prompts) + runner + relatório.
- Registro do resultado por skill (taxa bruta, sem inventar gate).

**Critério pass/fail:** ≥ **80%** de seleção correta por skill. Abaixo disso, a skill não é
publicável como está (reescrever a `description` — é a única string vista na seleção — ou arquivar).

**Cuidados:**
- Medir no **modelo real** (ver B4); resultados de outros modelos não transferem.
- Não confundir "teste de trigger" com "teste de conteúdo": B1 mede **roteamento**, não obediência.

---

## 3. B2 — Gate de CI determinístico

**Objetivo:** impedir que uma skill "apodreça" ou infle sem ser detectada (churn de toolchain,
descrição vazia, referências quebradas).

**Escopo do job (obrigatório):** para cada `SKILL.md`:
- frontmatter presente; `name` == nome da pasta; `description` não vazia;
- corpo **≤ 250 linhas**; **≤ 5** referências, todas existentes;
- **executa B1** (trigger-test) e falha se faltar teste.

**Paridade agent→git→CI:** o **mesmo** script roda no `pre-commit` (lefthook/`.githooks`) e no CI.
O contrato real é o **required check** em branch protegido (hooks são contornáveis com `--no-verify`).

**Formatos:** GitHub Actions **e** Forgejo Actions (conforme `OBJETIVO.md` §7).

**Ratchet:** manter baseline de violações; **piora bloqueia** (não gamificar — ver `OBJETIVO.md` §4).

**Dependência:** consome B1 (harness) e B4 (medição de aceitação).

---

## 4. B3 — Experimentos R1/R2/R4/R6

Experimentos A/B baratos para fixar comportamentos que a documentação do OpenCode **não define**.
Nenhum exige código de produção; são medições de configuração.

| Exp. | Pergunta | Como medir | Pass / Fail | Status |
|---|---|---|---|---|
| **R1** | `permission.skill: deny` remove a skill da **superfície de roteamento** ou só bloqueia a invocação? | Negar algumas skills e inspecionar `opencode debug skill` / descrição da tool `skill` | **Pass** = some da superfície → deny serve para curar; **Fail** = continua listada → só remoção física funciona (aposta atual) | aberto (perdeu urgência: remoção já foi física) |
| **R2** | O symlink `~/.config/opencode/skills → ~/.agents/skills` fazia descobrir as **mesmas skills duas vezes**? | Comparar a contagem com e sem o symlink | **Pass** = contagem igual (deduplica); **Fail** = dobrada | **resolvido na intervenção** (symlink removido; contagem = 16) — registrar resultado |
| **R4** | Caminho de `instructions` inexistente é **ignorado em silêncio** ou derruba o startup? | Rodar o OpenCode num diretório sem `AGENTS.md`/`CONTRIBUTING.md` | **Pass** = ignora (config atual é segura); **Fail** = erro duro → condicionar as instruções | aberto (vivo: a config global referencia 2 arquivos que só existem em alguns repos) |
| **R6** | `globs:` injeta conteúdo automaticamente ao abrir arquivo que casa (`.rs`/`Cargo.toml`)? | Medir o conteúdo injetado com e sem `globs` | **Pass** = não injeta surpresa; **Fail** = injeta → confirma custo oculto | **resolvido na intervenção** (globs removidos) — registrar resultado |

**Entregável:** uma nota curta em `docs/research/` com o procedimento, a saída observada e o veredito
de cada um dos quatro.

---

## 5. B4 — Precisão de gatilho no modelo (R5)

**Objetivo:** medir, no **nosso** modelo, a precisão de seleção — porque os números da literatura
(precisão 42,3% → 5,9% de 5→100 skills; shadowing = 68% da degradação em ~200 skills) vêm de
**outros modelos** e não transferem.

**Método:** é a **medição de aceitação** que usa o harness de B1. Definir baseline, threshold (≥80%)
e re-medir sempre que o modelo/provider mudar.

**Entregável:** número bruto por skill + data + modelo/provider, registrado em `docs/research/`.

**Relação com B1:** B1 **constrói** o harness; B4 **usa** o harness para decidir a aceitação da curadoria.

---

## 6. Ordem de execução e dependências

```
B3 (barato; informa decisões)  ──┐
                                 ├──> B1 (harness) ──> B4 (medição) ──> B2 (gate de CI)
R2/R6 já resolvidos ─────────────┘
```

1. **B3** — fechar os experimentos de configuração (rápido; R2/R6 só registrar; R1/R4 medir).
2. **B1** — construir o harness de trigger-test.
3. **B4** — medir a precisão no modelo real e congelar a baseline.
4. **B2** — promover o harness a gate de CI (required check), com ratchet.

---

## 7. Definição de pronto

- [ ] `scripts/trigger-test/` existe e roda headless sem intervenção manual.
- [ ] Cada uma das 16 skills tem taxa de seleção registrada (≥80% ou arquivada/reescrita).
- [ ] Baseline de precisão medida no modelo de referência (B4) e versionada.
- [ ] CI com job obrigatório que linta `SKILL.md` **e** roda o trigger-test.
- [ ] Paridade: o mesmo script roda no pre-commit e no CI.
- [ ] Ratchet de violações ativo (piora bloqueia).
- [ ] Nota de `docs/research/` com os vereditos de R1/R2/R4/R6.

---

## 8. Referências

- `MDMV/documentos/opencode-configuracao-ideal-2026-09-22.md` — relatório de origem (§5.6, §7, §8).
- [`OBJETIVO.md`](../OBJETIVO.md) — visão, princípios e critérios de "cheguei".
- [`ROADMAP.md`](../ROADMAP.md) — plano de entrega do harness.
- [`specs/001-deterministic-rust-harness/spec.md`](../specs/001-deterministic-rust-harness/spec.md) — FR-013 (skill fina ≤250 linhas, ≤5 refs, só publica com trigger medido).
- OpenCode — `opencode debug skill` / `opencode debug config` (inspeção da superfície real).
