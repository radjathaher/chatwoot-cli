---
title: fix path placeholder + method case
status: closed
priority: 1
issue-type: task
created-at: "\"\\\"2026-01-28T16:53:11.168069+07:00\\\"\""
closed-at: "2026-01-28T16:53:23.126414+07:00"
close-reason: "Fixed placeholder mapping + method uppercasing. Validation: get-account-details 200 with CHATWOOT_BASE_URL/CHATWOOT_API_TOKEN."
---

Fix API 400s by mapping path placeholders and uppercasing HTTP method. Files: src/main.rs, src/http.rs. Acceptance: get-account-details returns 200.
