---
name: react-patterns
description: A frontend-specific skill tuned for modern React + Tailwind CSS v4.
---

# Modern React Patterns

This skill ensures the agent uses modern, performant, and clean patterns for React development.

## Core Rules

1.  **Functional Components Only**: No `class` components.
2.  **Hooks**: Use `useState`, `useEffect`, `useMemo`, `useCallback` appropriately. Custom hooks for reusable logic.
3.  **Tailwind CSS v4**:
    *   Use utility classes for styling.
    *   Avoid `@apply` unless effectively creating a reusable component abstraction that cannot be a component itself.
    *   Use arbitrary values `w-[10px]` sparingly; prefer theme tokens.
4.  **State Management**:
    *   Local state > Context > External Store (Zustand/Redux).
    *   Don't lift state up unless necessary.
5.  **Performance**:
    *   Code splitting with `React.lazy`.
    *   Avoid prop drilling (use Composition or Context).

## Anti-Patterns to Spot
*   `useEffect` dependencies missing.
*   Mutating state directly.
*   Deeply nested ternary operators in JSX.
