---
description: Review changed code for security vulnerabilities (OWASP top 10, injection, data exposure)
---

# Security Review

Review all changed files for security vulnerabilities. Focus on the diff against the base branch.

## Steps

1. **Get the diff:**
   Run `git diff origin/main -- '*.rs' '*.ts' '*.svelte' '*.json'` to see what changed.

2. **Check for OWASP top 10 issues:**
   - **Injection:** SQL injection, command injection, XSS, template injection. Look for unsanitized user input passed to queries, shell commands, or HTML rendering.
   - **Broken auth:** Hardcoded credentials, missing auth checks on new endpoints, tokens in logs.
   - **Sensitive data exposure:** Secrets in code/config, PII in logs, missing encryption for sensitive fields.
   - **Insecure deserialization:** Untrusted data deserialized without validation (e.g., `serde_json::from_str` on user input without schema validation).
   - **SSRF:** User-controlled URLs passed to fetch/request functions without allowlist.

3. **Check Rust-specific concerns:**
   - `unsafe` blocks added or modified
   - `.unwrap()` on user-facing code paths (should use proper error handling)
   - Unchecked `.parse()` on external input (NaN, inf bypass)
   - Path traversal via user-supplied file paths
   - Denial of service: unbounded allocations from user input, missing timeouts

4. **Check frontend-specific concerns:**
   - `{@html}` with unsanitized content (XSS)
   - `eval()` or `new Function()` usage
   - Sensitive data stored in localStorage/sessionStorage
   - CORS misconfigurations in fetch calls

5. **Check config/infra concerns:**
   - New environment variables containing secrets without `.env` gitignore coverage
   - Permissive CORS origins added
   - Debug/verbose logging enabled in production paths
   - New dependencies with known vulnerabilities

6. **Present findings:**
   For each issue found, report:
   - **Severity:** Critical / High / Medium / Low
   - **File:line** — exact location
   - **Issue** — what's wrong
   - **Fix** — how to remediate

   If no issues found, report: "Security review: PASS — no vulnerabilities found in changed code."
