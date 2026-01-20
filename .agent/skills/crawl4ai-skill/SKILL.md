---
name: crawl4ai-skill
description: Integrates with web crawling libraries to ingest live documentation.
---

# Crawl4AI Skill

This skill allows the agent to ingest and summarize documentation from external websites using its `read_url_content` capabilities, effectively simulating a "crawling" behavior for knowledge acquisition.

## Usage

When the user gives a URL and asks to "learn from this" or "read the docs":

1.  **Fetch Content**: Use `read_url_content` to get the markdown representation of the page.
2.  **Summarize/Ingest**:
    *   If the content is a single page, analyze it and summarize key API points, configuration options, or concepts relevant to the user's task.
    *   If the content is a landing page, identify the most relevant sub-links (e.g., "Getting Started", "API Reference") and offer to read those next.
3.  **Documentation Generation**: optionally save the summarized knowledge into a local markdown file (e.g., `docs/external/lib_name.md`) for persistent reference.

## Prompting Strategy

*   "Reading [URL] to extract API signatures..."
*   "Ingesting documentation for [Library Name]..."
