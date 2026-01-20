---
name: quorum
description: A skill that facilitates a "multi-agent debate" logic, where different specialist agents critique a proposed implementation before the code is written.
---

# Quorum (The Council)

This skill simulates a debate between specialist personas to improve decision quality.

## The Personas
1.  **The Architect**: Cares about structure, scalability, and patterns.
2.  **The Hacker (Security)**: Cares about exploits, trust boundaries, and data leaks.
3.  **The Pragmatist (Product)**: Cares about speed, delivering value, and YAGNI (You Ain't Gonna Need It).

## Workflow
When faced with a complex architectural decision:

1.  **Proposal**: The Agent (Chair) outlines the proposed solution.
2.  **Dissent**:
    *   *Hacker*: "This exposes the internal API..."
    *   *Architect*: "This creates a circular dependency..."
    *   *Pragmatist*: "This is over-engineered..."
3.  **Synthesis**: The Agent synthesizes the feedback into a revised, stronger plan.

## Usage
"Calling a Quorum to decide on DB schema..."
