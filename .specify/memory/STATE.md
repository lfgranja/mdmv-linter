# STATE — mdmv-linter

> Memória de sessão (≤1 página). Leia no início de tarefa não-trivial; atualize ao encerrar.
> Atualizado: 2026-09-23 (implementação 001: Wave 0 + Wave 1 concluídas)

## Posição
- Fase: **F0/F1 do ROADMAP** (fundação + artefato determinístico) — Wave 0 (T001–T010) e
  Wave 1/Foundational (T011–T043) **concluídas e verdes** (`fmt`, `clippy -D warnings`,
  `nextest` 79/79, `cargo deny check`).
- Próxima: **Wave 2 / US1 MVP** (T044–T057, `init`).
- Branches: `feat/001-harness-setup` (PR #2) → `feat/001-harness-foundational` (stacked).
- Pesquisa: ~100%. Produto instalável: ~25% (esqueleto + infra + dominio + assets; comandos stub).
- Este repo é o dogfood (presets + CI + fixture + CLI).

## Decisões ativas
- Governança determinística pré-código; paridade agent→git→CI via `verify.sh`.
- Presets por tier (lib/app/embedded); `allow_attributes = "deny"` (anti-cheat).
- Toolchain pinada; fixture canário que compila a config e planta 1 violação por lint.
- Skill fina ≤250 linhas, publicada só após trigger test medido.
- OpenCode: Spec-kit **por projeto** (não global); sem plugins pesados; Graphify por projeto; `impeccable` só para design; GSD descartado (ideias emprestadas).
- Subagentes MDMV: `mdmv-verifier` (gate) e `mdmv-researcher` (contexto).
- Supply-chain: `cargo-deny` + `cargo-audit` + `osv-scanner` no gate (ver Fase E).

## Bloqueios / riscos
- **Defeito de contrato (descoberto na Wave 0)**: `[workspace.lints]` (base comum) + membro com
  `[lints] workspace = true` **e** adições por tier é **inválido no Cargo** (herança é tudo-ou-nada:
  `cannot override workspace.lints in lints`). O `init` de US1 precisa materializar o union
  completo no manifesto de cada membro. Corrigir a redação de `contracts/cli-and-ci.md` §1 /
  T001/T002 / data-model antes de US1.
- **Ambiguidade da fixture (T038)**: "1 canário por lint folha deny/forbid" é inviável para os
  grupos (`all`/`nursery`/`cargo`/`pedantic`/`rustdoc::all`). Interpretação adotada: 1 canário por
  lint **nomeado explicitamente** (24 canários); grupos cobertos pelos nomes. Documentar em US3.
- **CI dogfood**: `.github/workflows/harness-verify.yml` aplicado, mas `mdmv-linter check --fixture`
  só existe em US2/US3 → CI ficará vermelho até lá (esperado neste estágio).
- casv-rust: 12/26 arquivos órfãos + `LocalClient` stubado — pré-requisito da integração.
- Churn de lints a cada ~6 semanas → fixture + `doctor` obrigatórios.

## Próximo passo
- Wave 2 (US1): testes T044–T050 (TDD, devem falhar), depois `init` T051–T057+T054a.
- Antes de US1: decidir a materialização do union de lints por membro (ver defeito acima).
- Fases A–F do plano OpenCode concluídas (A coerência, B subagentes, C memória, D headroom, E supply-chain, F Graphify+impeccable).
- Seguir: G (documento `documentos/opencode-configuracao-ideal-2026-09-23.md`).
- Graphify instalado por projeto aqui (`.opencode/plugins/graphify.js` + skill + seção no AGENTS.md).
