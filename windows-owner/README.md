# Private native Windows owner proof

Experimental native Rust window and notification-area icon, no installer or autostart. Choose text file captures at most 4096 UTF-8 bytes locally; explicit confirmation grants only that immutable snapshot or status, once, for up to 60 seconds. No file write/delete, desktop, browser, administrator, TCP, remote or ChatGPT access is granted. No bearer token or file content is printed, placed on clipboard or persisted. The UI intentionally has no external-client handoff yet; internal test clients use the in-memory pairing API. This is not a usable direct dot connection.

Disconnect signals the owner stop event and waits up to two seconds for the serving thread. Failure fences new grants. Exit uses process termination if cleanup cannot be confirmed, which ends this process's threads/handles, not unrelated browsers. No child process is launched. Tray left/right click restores the window, whose menu has explicit Disconnect and Exit; closing the window exits, never hides a background service. Shell icon appearance and actual interactive dialogs are not yet Windows-tested. The static title is not authoritative live session status after peer completion or expiry; a completion-driven status update remains necessary before delivery.

Snapshot selection opens a local fixed-drive file through retained parent directory handles, refuses reparse/single-link violations and uses read-only sharing during capture. This is still trusted/quiescent local filesystem scope, not a hostile ancestor/DOS-device/attribute-reparse/ACL security boundary. Selection reads synchronously and can block the UI; there is no guaranteed preemption of arbitrary filesystem I/O. The initial file dialog is an OS shell dialog and can browse locations even though the snapshot admission rejects nonlocal namespaces; no claim that opening the dialog itself cannot perform shell/network activity. Capture before explicit sharing is a local user-directed read, not a remote grant.

Tests authored: internal owner consent-grant/snapshot pipe roundtrip, revocation/restart and invalid-input no-session. Compile-check succeeds against Windows MSVC target. These new tests and native UI have NOT yet run on Windows. Existing prior Windows transport/workspace acceptance applies only to the earlier source version, not this delta.

Official API references:
- https://learn.microsoft.com/en-us/windows/win32/api/commdlg/nf-commdlg-getopenfilenamew
- https://learn.microsoft.com/en-us/windows/win32/api/commdlg/ns-commdlg-openfilenamew
- https://learn.microsoft.com/en-us/windows/win32/api/shellapi/ns-shellapi-notifyicondataw
- https://learn.microsoft.com/en-us/windows/win32/shell/notification-area

Release holds: reviewed pairing/client identity and user-visible permissions, full real UI/Exit process acceptance, file-identity/ACL race policy, other-user/same-user threat tests, external dot admission, browser/app control and installer signing/distribution. No release clearance implied by a future CI pass.
