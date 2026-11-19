# Experimental Windows disposable workspace proof

Source-only CI candidate, not an installer or computer connector. Windows stdio supports bounded list/read/create-new and process-local revoke in a newly selected directory. No overwrite/delete/admin, socket server or remote pairing.

Use only a known local disposable temp directory. CLI selection is not authenticated human consent. Absolute UNC/mapped roots are currently accepted, so no socket API does not guarantee no remote filesystem I/O. Trusted quiescent ancestors/root are required: joined paths are not handle-relative protection against replacement/junction races. ACLs inherit parent; blocking operations are not preemptively revoked. Interrupted effects are uncertain and not transaction-journal bound. Do not expose remotely.

CI compiles and runs disposable tests only; no owner files, deployment or artifact delivery. Passing CI does not establish production safety, desktop control or connection to an assistant.
