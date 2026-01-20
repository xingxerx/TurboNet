---
name: license-header-adder
description: Scans files and inserts the appropriate legal/license headers.
---

# License Header Adder

This skill automates the process of adding license headers to source files.

## Usage

When the user asks to "add license headers" or "check licenses":

1.  **Identify License**: Check `LICENSE` file or `package.json`/`Cargo.toml` to determine the project's license (e.g., MIT, Apache 2.0).
2.  **Scan Files**: Look for source files (e.g., `.rs`, `.py`, `.js`, `.ts`, `.c`, `.cpp`, `.go`) that correspond to the project's main languages.
3.  **Check Headers**: Read the first few lines of each file to see if a license header exists.
4.  **Add Header**: If missing, prepend the appropriate comment block.

### Logic
*   **MIT**:
    ```text
    /*
     * Copyright (c) [Year] [Owner]
     *
     * Permission is hereby granted, free of charge, to any person obtaining a copy...
     * [Standard MIT Text]
     */
    ```
*   **Apache 2.0**:
    ```text
    /*
     * Copyright [Year] [Owner]
     *
     * Licensed under the Apache License, Version 2.0 (the "License");
     * you may not use this file except in compliance with the License.
     * ...
     */
    ```

## Configuration

*   Prefer using existing comment styles for the language (e.g., `//` for Rust/JS, `#` for Python).
*   Be careful not to disrupt shebangs (`#!`) or encoding declarations.
