---
name: senior-engineering
description: A collection of skills for TDD (Test-Driven Development), Clean Architecture enforcement, and automated refactoring strategies.
---

# Senior Engineering Toolkit

This skill embodies the practices of disciplined software engineering.

## Sub-Skills

### 1. TDD Enfrocer
When writing new features:
1.  **Red**: Write the test FIRST. Ensure it fails for the right reason.
2.  **Green**: Write the *minimum* code to pass the test.
3.  **Refactor**: Clean up the code while keeping tests green.

### 2. Clean Architecture
Enforce the dependency rule: **Source Code Dependencies must point only inward, toward higher-level policies.**
*   Entities (Domain) should not know about Database or Web frameworks.
*   Use Interfaces/Ports to invert dependencies.

### 3. Automated Refactoring
Strategies for cleaning code:
*   **Extract Method**: Reduce function size.
*   **Replace Conditional with Polymorphism**: Remove switch statements.
*   **Introduce Parameter Object**: Simplify method signatures.

## Usage
Apply these heuristics whenever designing or implementing core business logic.
