---
title: "release workflow: tag trigger"
status: closed
priority: 1
issue-type: task
created-at: "\"\\\"2026-01-28T16:42:40.017898+07:00\\\"\""
closed-at: "2026-01-28T16:43:12.118385+07:00"
close-reason: Added tag push trigger; use ref_name for release tag.
---

Add tag push trigger to release workflow; use ref_name if workflow_dispatch input missing. Files: .github/workflows/release.yml. Acceptance: tag push builds + release.
