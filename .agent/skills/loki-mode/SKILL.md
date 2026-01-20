---
name: loki-mode
description: A high-level orchestration skill designed for end-to-end workflows, moving a project from a PRD to full Production deployment.
---

# Loki Mode (Orchestrator)

Loki Mode is a meta-skill for autonomous end-to-end project execution.

## Phases

1.  **Phase 1: Inception**
    *   Input: PRD or User Idea.
    *   Action: Generate `implementation_plan.md` and `task.md`.
    *   Gate: User Approval.

2.  **Phase 2: Swarm (Parallel Execution)**
    *   Break down `task.md` into isolated components.
    *   Execute tasks (using tools).
    *   Maintain `task.md` status.

3.  **Phase 3: Convergence**
    *   Integration testing.
    *   Verify all components work together.

4.  **Phase 4: Delivery**
    *   Final code review (Ruthless Reviewer).
    *   Generate `walkthrough.md`.
    *   Notify User.

## Directive
"I am burdened with glorious purpose." -> Take ownership of the entire lifecycle.
