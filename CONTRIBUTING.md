# Contributing to TurboNet

Thank you for your interest in contributing to TurboNet! We welcome contributions from the security research community.

## ⚠️ Legal & Ethics
By contributing to this project, you agree that your code:
1.  Is for educational and authorized testing purposes only.
2.  Does not contain malicious payloads (e.g., actual malware, ransomware).
3.  Adheres to the project's license and code of conduct.

## 🛠️ Development Workflow

1.  **Fork** the repository.
2.  **Clone** your fork locally.
3.  **Create a branch** for your feature or fix (`git checkout -b feature/amazing-feature`).
4.  **Test** your changes:
    ```bash
    cargo test --workspace
    ```
5.  **Commit** your changes with descriptive messages.
6.  **Push** to your branch.
7.  **Open a Pull Request**.

## 🧪 Testing Guidelines
-   Ensure all unit tests pass.
-   If you modify the crypto module, please verify against NIST test vectors if possible.
-   For GPU code (`.cu`), ensure it compiles with standard NVCC.

## 🔒 Reporting Vulnerabilities
If you find a security vulnerability in TurboNet itself, please do NOT open a public issue. Email the maintainers directly or use GitHub's private vulnerability reporting feature.
