# Computer-Link local text export candidate

Backlog scope: owner-selected local write/edit capability; preserve originals and
require exact per-operation consent. This candidate implements **export as a new
file**, not replacing an original. It is not wired to the tray UI, MCP, a listener,
the installer, or the owner's PC. No executable or public release is produced.

## Implemented boundary

* At most 4096 UTF-8 bytes, one ordinary ASCII leaf, immutable escaped preview.
* `LocalExport::selected_local_directory` requires an explicitly owner-selected
  existing ordinary fixed-drive NTFS directory. Ancestor and directory handles
  remain held, rejecting reparse objects and normal rename/delete sharing.
* Preview binds the retained directory, filename and bytes. Exact confirmation is
  consumed once, including failures; Deny has no disk effects.
* Revoke and owner drop invalidate old previews. Revoke serializes with an already
  admitted operation; it is **not** cancellation of blocking filesystem I/O.
* Create `<name>.pending` by `NtCreateFile` relative to the retained directory,
  FILE_CREATE, no sharing, FILE_OPEN_REPARSE_POINT, non-directory. Existing stage
  objects are never opened. Write and sync the exact retained file handle.
* Publish by `SetFileInformationByHandle(FileRenameInfo)` relative to the same
  directory, ReplaceIfExists FALSE. No second path open, no destination overwrite.
* Any failed stage is retained for explicit owner review. No automatic cleanup,
  retry, deleting, replacing, executing, remote transfer, or persistent grant.
* This is namespace publication after complete write, **not** crash/power-loss
  durability. Staging filenames are visible. ACLs inherit from the destination.

## Holds and remaining gates

This is not a general secure file server or an arbitrary editing API. The native
backend has cross-compiled but **has not run on Windows**. Exact Windows NTFS
rename and sharing behavior must pass native tests before owner UI wiring. The
inherited initial directory admission uses sequential Win32 paths, not complete
handle-relative traversal; hostile DOS-device remapping, kernel/admin interference,
and same-user attribute/ACL/reparse mutation are not proven safe. Only trusted,
quiescent local roots are in scope. Do not expose the API to untrusted callers.
An exact immutable preview API is not itself a real user-consent UI: caller
integration must show the selected root and complete escaped content and request
fresh local consent. No claim that boolean `true` authenticates an owner.

Backlog still open: independent security review, native synthetic tests (including
reparse/ancestor attacks and partial-I/O/power-loss evidence), accessible native
preview/consent UI, off-UI operation and cancellation semantics, dot admission.
Existing `windows-workspace` joined-path write is not reused or promoted.

## Evidence

Cloud `cargo test`: 6 state/validation tests passed. MSVC `cargo check --all-targets`:
passed. Authored Windows cases: publish exact bytes, no-overwrite/retained stages,
Deny/revoke no effects, root rename denial, drop revocation, destination hardlink,
destination appearing at publication, preexisting staging hardlink; unexecuted.
Root namespace test inherited from existing source is also unexecuted here.
Cleanup in tests applies only to unique synthetic temp fixtures, never product data.

Official API references consulted 2026-11-20:
- https://learn.microsoft.com/en-us/windows/win32/api/winternl/nf-winternl-ntcreatefile
- https://learn.microsoft.com/en-us/windows/win32/api/winbase/ns-winbase-file_rename_info
- https://learn.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-setfileinformationbyhandle

Run: `cargo test`; on disposable Windows NTFS runner `cargo test -- --test-threads=1`.
No network, dependencies, credentials, services or artifacts required.

## Expanded native acceptance candidate

Additional Windows tests exercise stage/destination directory junction refusal,
selected reparse root/ancestor refusal, pinned ancestor rename and incompatible
writer refusal, injected partial-write/sync failures, one/80-character names and
final exclusive-handle release. Injected failures are test seams, not an actual
hardware fault or power-loss test. Fixture junction creation uses Windows mklink
only within unique generated disposable directories, refuses shell metacharacters,
and requires success (no skipped reparse cases). No native result yet.
