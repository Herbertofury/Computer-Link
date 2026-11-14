# Independent architecture and delivery gates

The local process is a tool executor, not an additional AI agent. Clients provide reasoning. The provider-neutral registry has no knowledge of ChatGPT, dot, model selection or billing.

1. Core registry: available/effect metadata, duplicate detection, explicit failure, read-only call mode. Implemented and fixture-tested.
2. Diagnostics adapter: OS/architecture/runtime and optional selected-volume capacity. Implemented; no hostnames, account names, full paths, environment or file content returned.
3. Transport: local JSON-lines implemented. MCP protocol integration remains pending; use an official SDK rather than claiming JSON-lines equals MCP. Remote authentication, pairing and consent design must precede any network listener.
4. Workspace adapter: candidate reuse, bounded read/write, backups/conflict readback, separate permission. Not implemented.
5. Process adapter: candidate reuse, argv contracts, output streaming, cancellation and job termination scope. Not implemented.
6. Desktop adapter: Windows UIA first, visual input explicit fallback. Not implemented.
7. Browser adapter: supported browser tools first, isolated from desktop. Not implemented.
8. Client adapters: ordinary chat, dot and generic MCP clients each get independent acceptance matrices. Not implemented.

Disable/remove any optional adapter without breaking diagnostics, CLI or other adapters. No Constellation, Hover, Agent-Foundry, or game project should require this service for unrelated features. Shared governance is a pinned documentation snapshot, not a runtime dependency.

## Security boundaries

This foundation is not a hardened service or permission sandbox. Python code loaded into the process is trusted; the registry effect label is not protection against malicious adapters. No HTTP listener or persistent grants are created. Future external adapters require provenance/license review and OS-enforced boundaries appropriate to their privileges. Never expose shell, filesystem, clipboard, screen or UI control through unauthenticated public endpoints. Do not implement automatic approval clicking, credential harvesting, quota bypass, or hidden monitoring.

## Acceptance checklist

- [x] Independent registry/adapter removal tested.
- [x] Unsupported operations and malformed inputs fail without execution.
- [x] Local subprocess JSON-line recovery and disk capacity readback tested on Linux.
- [ ] Windows clean install/uninstall and native diagnostics verified.
- [ ] Comparative upstream benchmark and license pinning.
- [ ] Official MCP transport conformance; cancellation and output budgets.
- [ ] Authenticated pairing, revoke/disconnect, visible activity and per-capability configuration.
- [ ] File edit / process / UI workflows with backups and effect verification.
- [ ] Ordinary chat and dot end-to-end actual supported connection tested separately.
- [ ] No-loss compatibility, latency and memory evidence on real Windows workflows.
