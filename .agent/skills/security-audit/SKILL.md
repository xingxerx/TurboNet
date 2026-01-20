---
name: security-audit
description: Includes sub-skills for OWASP Top 10 checks and ethical hacking heuristics.
---

# Security Audit Skill

This skill allows the agent to perform basic security auditing and vulnerability scanning on the codebase.

## OWASP Top 10 Checklist
When auditing, check for:
1.  **Injection**: SQLi, NoSQLi, Command Injection. (Look for concatenated strings in queries).
2.  **Broken Auth**: Weak passwords, missing tokens, exposed session IDs.
3.  **Sensitive Data Exposure**: Keys in code, PII logging, weak crypto.
4.  **XXE**: XML External Entities.
5.  **Broken Access Control**: IDOR, missing role checks.

## Heuristics
*   "Never trust user input."
*   "Sanitize early, escape late."
*   "Least Privilege principle."

## Action
*   If you find a vulnerability, flag it with `[SECURITY CRITICAL]`.
*   Suggest a remediation (e.g., "Use parameterized queries").
