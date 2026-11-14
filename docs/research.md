# Computer tools research, 2026-11-14

## Discovery and reuse decision

Accessible Herbertofury repository inventory and current READMEs identify Project Constellation as browser chat organization/recovery, Hover as native previews with app-specific control, Noxviola as a project index, Agent-Foundry as governance. ProjectDump contents and conversation searches for bridge/MCP/regular chats did not establish an existing standalone general computer connector. This is bounded negative evidence, not proof that unpublished local work does not exist. Preserve all these projects. Computer Link is the new standalone destination, not a forced dependency for them.

## Candidates from official repositories

| Candidate | Evidence and role | License/reuse | Evaluation needed |
|---|---|---|---|
| [Desktop Commander](https://github.com/wonderwhy-er/DesktopCommanderMCP) | File/search/edit and terminal MCP, ordinary chat computer access product | MIT per upstream repository; first existing foundation to evaluate | Pin version; inspect command/file boundaries, cancellation and result readback; compare local operation cost |
| [Windows MCP](https://github.com/sbroenne/mcp-windows) | Windows UI Automation semantic actions plus CLI | MIT per upstream | Actual Windows app targets, stale-element behavior, focus/DPI/multiple windows; not proven by Linux tests |
| [Chat on Steroids](https://github.com/totec448-spec/chat-on-steroids) | Upstream of qDeizer/chatGPT-Computer-MCP; Windows scoped chat bridge | [MIT](https://github.com/totec448-spec/chat-on-steroids/blob/main/LICENSE) | Prefer upstream evaluation over obscure forks; verify transport, scope, consent and installer before reuse |
| [ChatCmd](https://github.com/int04/ChatCmd) | Local-first chat bridge; repository describes unofficial DOM bridge | [MIT](https://github.com/int04/ChatCmd/blob/main/LICENSE) | Browser UI changes, explicit sending semantics, provider terms and connection support. DOM automation does not establish supported dot access |
| [PC Bridge](https://github.com/Martin123132/PC-Bridge) | Similar ordinary chat to Windows workspace product | [PolyForm Noncommercial or Small Business](https://github.com/Martin123132/PC-Bridge/blob/main/LICENSE), source-available, not OSI open source | Reference only pending use-specific license review; do not blindly copy into MIT project |
| [pywinauto](https://github.com/pywinauto/pywinauto) | Mature Windows text/property GUI automation foundation | Verify exact pinned package license before vendoring | Compare against .NET UIA on real apps; do not build a duplicate UIA engine without evidence |
| [Microsoft Hybrid Computer Use](https://microsoft.github.io/Hybrid-Computer-Use/assets/Hybrid-Computer-Use.pdf) | Research combining programmatic, semantic and visual interfaces | Paper, not a redistribution grant for arbitrary implementation | Good comparison design: choose strongest supported interface per task; no performance claim from paper alone |

Other leads: [Windows GUI MCP](https://github.com/dcl632/windows-gui-mcp), [Windows Commander MCP](https://github.com/exalsch/windows-commander-mcp), [Easy Local MCP](https://github.com/Ryanma-YX/easy-local-mcp). Discovery only; licenses, security, maintenance and real target behavior not yet qualified. No upstream executable was run and no upstream source was copied in this checkpoint.

## Recommendation, not a proven winner

Evaluate Desktop Commander for file/terminal reuse and Windows MCP for semantic desktop actions first; keep ordinary chat transport and desktop mechanics separate. Build only the orchestration, explicit capability control and evidence that existing candidates do not supply well enough. A bridge cannot force an unsupported client to gain tools or waive its billing. Dot acceptance requires the actual dot tool surface, not just a working local MCP client.

## Comparative evidence protocol

For each pinned candidate: record commit/version/license, install size, cold startup, idle CPU/RSS, round-trip latency (p50/p95), cancellation, reconnect and complete results on identical temporary workspaces. Include non-ASCII names, permission failures, large output, stale targets, repeated calls and disconnected clients. Save raw results, failures and exact harness identity. Benchmark actual workflows, not only synthetic dispatch. Do not call a tool best/fastest until target comparisons prove it without reduced capabilities.
