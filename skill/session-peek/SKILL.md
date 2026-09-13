---
name: session-peek
description: Use when starting a dev server/backend/db process for a user, or when asked to check logs, errors, or recent output from an already-running process. sessionPeek runs a tagged process and lets you query its structured, filtered log instead of dumping raw terminal output into your context.
---

# sessionPeek

sessionPeek is a small, stack-agnostic CLI that wraps any long-running command (dev server, backend, database, etc.) so a human's terminal and an agent's terminal never fight over the same process, and so the agent never has to ingest a full raw output stream to answer a question about it.

The dev starts a process once, tagged with a short name. You (the agent) query that tag's log later, with filters, instead of asking the dev to paste output or running the command yourself and eating the whole stream in your context.

## Before using it

Check the binary is available: `sessionPeek --version` (or `which sessionPeek`). If it's missing, tell the user rather than silently building it yourself — it's a Rust binary built from its own repo via `cargo build --release`, producing `target/release/sessionPeek`. Don't install or add it to PATH without telling the user first.

## Command reference (current — v1)

Three commands exist right now: `run`, `log`, `delete`. Do not invoke or suggest `-d`/`--detach` on `run`, `status`, or `stop` — they are planned but **not implemented yet**; using them will just fail.

### `sessionPeek run -t <tag> -- <command...>`

Spawns `<command...>`, streaming its output live to whoever's terminal invoked it, while also writing every line to a structured log file at `<project-root>/.cache/sessionpeek/<tag>.log` in the form:

```
[2026-09-13T11:52:31Z][stdout] hello
[2026-09-13T11:52:31Z][stderr] err-line
```

This call **blocks** in the foreground until the process exits — there is no background/detach flag yet. That means:
- When the **dev** runs it, it behaves exactly like running the command directly (e.g. `npm run dev`) — nothing changes for them except a log file also gets written.
- When **you** (the agent) need to start something yourself without blocking your own turn, invoke it through your own tool's background-execution feature (e.g. Bash's `run_in_background`) rather than waiting on it — `sessionPeek run` itself has no such flag today.
- Never start two `run` invocations with the same tag concurrently — both will append to the same log file with no coordination between them, which gets confusing. If a tagged process needs restarting, stop the existing one first (Ctrl-C for a foreground one; there's no `stop` command yet for anything else).

Add `--fresh` to clear the tag's existing log and start writing from empty, instead of appending to whatever's already there (e.g. `sessionPeek run -t backend --fresh -- <command...>`). Useful when old output from a previous run is no longer relevant and would just add noise to later `log` queries.

Tag naming convention: short, lowercase, descriptive — `frontend`, `backend`, `db`, matching how the dev already thinks about their services.

### `sessionPeek log <tag> [flags]`

Reads and filters `<tag>`'s log file, printing only what matches. This is your primary tool for answering "what's happening with X" or "is there an error in Y" without asking for a paste or re-running anything.

| Flag | Effect |
|---|---|
| `--tail N` | last N matching lines only |
| `--stdout-only` | only stdout lines (mutually exclusive with `--stderr-only`) |
| `--stderr-only` | only stderr lines |
| `--grep <pattern>` | regex match against line content |
| `--since <duration>` | only lines newer than `<duration>` ago, e.g. `30s`, `10m`, `2h` |

Flags compose. Useful recipes:
- Checking for problems after a change: `sessionPeek log backend --stderr-only --since 5m`
- Confirming a server came up cleanly: `sessionPeek log frontend --tail 20`
- Hunting a specific error: `sessionPeek log backend --grep "Error|Exception"`

If `log` errors with "no log found for tag", the tag either was never started, or was started from a different project root (root discovery walks up from cwd to the nearest marker — `.git`, `Cargo.toml`, `package.json`, `.sessionpeek.json`, etc. — so a mismatch usually means the dev ran it from a different directory than you're querying from).

### `sessionPeek delete <tag>`

Deletes `<tag>`'s log file outright. Use this for cleanup (e.g. the dev is done with a tag and wants its history gone), not as a way to "reset" a tag before restarting it — use `run --fresh` for that instead, since it's one step and doesn't race with a process that might still be writing.

Caveat: if a `run` for that tag happens to still be actively writing when you delete its log, deleting doesn't stop that process — it keeps writing, just to a now-unlinked file that's no longer reachable at that path until the process exits. Prefer deleting a tag only once you know nothing is actively running for it.

## Suggesting this to a dev

When it's natural to suggest sessionPeek (e.g. the dev is about to start a dev server you'll need to monitor, or keeps pasting terminal output for you to read), suggest the ad hoc form first: run the *existing* command through sessionPeek without changing anything else —

```
sessionPeek run -t frontend -- npm run dev
```

This works regardless of the project's state and touches nothing in the repo.

## Embedding it into project scripts (package.json, Makefile, Procfile, etc.)

Only do this — and always ask the user before editing a shared script file — if the project is **fresh and effectively single-owner**. Do not wire sessionPeek into a pre-established project's canonical scripts; that forces the tool on every other contributor who didn't ask for it.

**Signals a project is pre-established (do not embed):**
- `git log` shows more than one distinct author, or a commit history clearly predating this session
- CI config exists (`.github/workflows`, etc.)
- A CONTRIBUTING guide, or a README describing a team workflow, exists
- Lockfiles/dependency setup were clearly done before you got involved

**If pre-established:** don't touch the shared script. Either use the ad hoc form above every time, or — only if the user explicitly asks for convenience — add a clearly separate, additive alias next to the original (e.g. `"dev:peek": "sessionPeek run -t dev -- npm run dev"` alongside the untouched `"dev": "npm run dev"`), never replacing or renaming what's already there.

**If genuinely fresh** (first commit(s), single author, no CI/team signals — e.g. a brand-new scaffold the current user is setting up): it's reasonable to wire sessionPeek directly into the primary script, e.g.

```json
"scripts": {
  "dev": "sessionPeek run -t frontend -- vite"
}
```

Still confirm with the user before making the edit — this is a judgment call about project state, not a certainty, and it's their script to own.
