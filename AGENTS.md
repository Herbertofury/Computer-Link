# Repository instructions

Read `.agents/foundry/AGENTS-adapter.md` for the pinned shared engineering core.
Keep this file and nested instructions project-owned: setup, commands, constraints and acceptance belong here.

## Computer Link local scope

Keep core, diagnostics, files, process, desktop, browser and client adapters independently usable. This is a deterministic tool layer, not a new model runtime. No claims of zero usage, quota bypass, dot integration or Windows acceptance without direct evidence. Use `.agents/skills/verify-computer-link/SKILL.md` for verification. Commands: `python -m unittest discover -s tests -v` and `python -m computer_link`. Research and unresolved native acceptance are in `docs/`. Do not enable persistent access or public network listeners as a side effect of development. Do not publish private machine information. The current protocol is JSON-lines, not MCP.
