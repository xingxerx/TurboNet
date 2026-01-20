---
name: json-to-pydantic
description: Generates typed Pydantic models or TypeScript interfaces from raw JSON.
---

# JSON to Pydantic/Types

This skill converts raw JSON data into strictly typed code.

## Usage

When the user provides a JSON object or points to a JSON file and asks for "types" or "models":

1.  **Analyze JSON**: Read the JSON structure. Identify types (string, int, boolean, array, object).
2.  **Determine Target**: Ask or infer if the output should be Python (Pydantic) or TypeScript (Interface).
3.  **Generate Code**:

### Python (Pydantic)
*   Use `pydantic.BaseModel`.
*   Use `typing.List`, `typing.Optional`, etc.
*   Generate nested classes for nested objects.
*   Example:
    ```python
    from pydantic import BaseModel
    from typing import List, Optional

    class Address(BaseModel):
        street: str
        city: str

    class User(BaseModel):
        id: int
        name: str
        tags: List[str]
        address: Optional[Address] = None
    ```

### TypeScript
*   Use `interface`.
*   Example:
    ```typescript
    interface Address {
      street: string;
      city: string;
    }

    interface User {
      id: number;
      name: string;
      tags: string[];
      address?: Address;
    }
    ```
