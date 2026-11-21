# Bug Verification: pending

- **Result**: not verified
- **Steps**: reset diag, ACTION_VIEW Hamlet with FLAG_GRANT_READ_URI_PERMISSION, observe >220 s, assert `action=complete` + `session_end` and position reaches timeline end.
