# STATE — mdmv-linter

> Memória de sessão (≤1 página). Leia no início de tarefa não-trivial; atualize ao encerrar.
> Atualizado: 2026-09-23 (implementação 001: Wave 0 + Wave 1 concluídas)

## Posição
- Fase: **F0/F1 do ROADMAP** — Wave 0 (T001–T010), Wave 1 (T011–T043) e
  **Wave 2 / US1 MVP** (T044–T057) **concluídas e verdes** (`fmt`, `clippy -D warnings`,
  `nextest` 96/96, `cargo deny check`, `.mdmv/verify.sh` exit 0).
- `init` funcional: deriva strict/ratchet do scan canônico, resolve tier por crate,
  aplica assets não-destrutivamente, grava baseline no ratchet, manifesto por último,
  override audit log idempotente, snapshot só quando muta. US1 validado (quickstart A/B/C).
- Próxima: **Wave 3 / US2** (T058–T065a, `check` + CI/parity/ratchet/supply-chain).
- Branches: `feat/001-harness-setup` (PR #2) → `feat/001-harness-foundational` (PR #3) →
  `feat/001-harness-us1-init`.
- Pesquisa: ~100%. Produto instalável: ~40% (init completo; check/doctor/metrics/upgrade/uninstall stub).
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
- Wave 3 (US2): `check` T062, depois CI T063–T065 + testes T058–T061/T065a (TDD primeiro).
- Pendente de contrato: reescrever `contracts/cli-and-ci.md` §1 / T001/T002 / `data-model.md`
  para o modelo de lints materializado por membro (o Cargo não faz "herdar + adicionar").
- `init` já materializa o union por membro em `crates/mdmv-linter/src/cli/init.rs`.
- Fases A–F do plano OpenCode concluídas (A coerência, B subagentes, C memória, D headroom, E supply-chain, F Graphify+impeccable).
- Seguir: G (documento `documentos/opencode-configuracao-ideal-2026-09-23.md`).
- Graphify instalado por projeto aqui (`.opencode/plugins/graphify.js` + skill + seção no AGENTS.md).
