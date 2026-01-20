---
name: docker-expert
description: Automates Dockerfile optimization, multi-stage build configurations, and container security hardening.
---

# Docker Expert Skill

This skill optimizes Dockerfiles for size, speed, and security.

## Best Practices
1.  **Multi-Stage Builds**: Separate build tools from runtime artifacts.
    ```dockerfile
    FROM golang:1.21 as builder
    ...
    FROM alpine:latest
    COPY --from=builder /app/bin /bin
    ```
2.  **Layer Caching**: Order instructions from least changed to most changed. Copy `go.mod`/`package.json` before source code.
3.  **Security**:
    *   Do not run as root (`USER nonroot`).
    *   Use trusted base images (e.g., `distroless`).
    *   Scan for vulnerabilities (if `trivy` is installed).
4.  **Optimization**:
    *   Combine `RUN` commands to reduce layers: `RUN apt-get update && apt-get install -y ... && rm -rf /var/lib/apt/lists/*`

## Usage
When asked to "fix Dockerfile" or "dockerize this":
*   Read existing Dockerfile.
*   Apply the above rules.
*   Check `.dockerignore` to ensure `.git`, `node_modules`, etc., are excluded.
