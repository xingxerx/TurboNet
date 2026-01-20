---
name: git-commit-formatter
description: Automatically analyzes staged changes and generates a Conventional Commit message.
---

# Git Commit Formatter

This skill helps you generate semantic commit messages based on the [Conventional Commits](https://www.conventionalcommits.org/) specification.

## Usage

When the user asks to "commit changes" or "write a commit message":

1.  **Check Status**: Run `git status` to see what is staged.
2.  **Analyze Diffs**: Run `git diff --cached` to see the actual changes.
3.  **Generate Message**: Formulate a commit message with the following structure:
    ```
    <type>(<scope>): <description>

    [optional body]

    [optional footer(s)]
    ```
    *   **Types**: `feat`, `fix`, `docs`, `style`, `refactor`, `perf`, `test`, `build`, `ci`, `chore`, `revert`.
    *   **Scope**: A noun describing the section of the codebase (e.g., `api`, `auth`, `parser`).
4.  **Propose**: Propose the command `git commit -m "..."` to the user.

## Rules

*   Keep the summary line under 72 characters.
*   Use the imperative mood in the subject line.
*   Mention closing issues in the footer (e.g., `Closes #123`).
