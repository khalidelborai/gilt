# Worker instructions — gilt

You are an autonomous coding agent in YOUR OWN git worktree:
- worktree root (your cwd): /Users/khaklidelborai/orca/workspaces/gilt/p7-22-color
- branch: refs/heads/khalidelborai/p7-22-color
Make ALL changes HERE (edit `crates/` in THIS worktree).

## Use your advisor tool (required)
Use the **advisor** tool throughout: consult it before you start (sanity-check the approach against the
plan), and again whenever you hit a design decision, an ambiguous step, a build/test failure you can't
immediately explain, or before each commit. Treat the advisor as a senior reviewer; incorporate its feedback.

## The plan (read it in full first)
The plan lives OUTSIDE your worktree — read this absolute path directly:
/Users/khaklidelborai/orca/workspaces/gilt/live-pause-resume/.orca-fanout/plan-7.22-color.md

Implement EVERY task in order, exactly as written. The plan's code blocks are the source of truth, and the
cited line numbers are accurate for this checkout. If a cited line number is slightly off, locate the named
function/symbol instead — the fix is logical, not line-number-bound.

## Method — TDD, per task
For each step in a task: write the failing test → run it and confirm it FAILS → implement the minimal fix
shown in the plan → run it and confirm GREEN → commit with the exact conventional-commit message the plan
gives. **One commit per plan task.**

## Rust gates (this machine)
- The workspace forbids `unsafe`. Do NOT set `RUSTC_WRAPPER` (sccache is wired globally in ~/.cargo/config.toml).
- Prefer `cargo nextest run -p <crate>` for tests; `cargo test -p <crate> --doc` for doctests.
- Lint must be clean: `cargo clippy -p <crate> --all-targets -- -D warnings`.
- Do NOT push. Do NOT open a PR.

## When all tasks are done
Run the plan's final gates:
cargo nextest run -p gilt  && cargo test -p gilt  --doc && cargo clippy -p gilt  --all-targets -- -D warnings && cargo check --workspace

Then report by updating the orchestration task (fill in the real SHA + summary):
orca orchestration task-update --id task_4c57fbd7918b --status completed --result '{"head":"<final HEAD sha>","summary":"<2-3 sentences>","concerns":"<deviations/blockers or none>"}' --json

If you are blocked and cannot proceed, set status blocked instead and stop:
orca orchestration task-update --id task_4c57fbd7918b --status blocked --result '{"reason":"...","at_task":"<n>"}' --json

Do not ask questions — make the best engineering decision, use the advisor, and note any deviation in the
result. Begin now: read the plan, then implement Task 1.
