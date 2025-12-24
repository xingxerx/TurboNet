# TurboNet CUDA Build Instructions (Windows)

## Problem
Your build requires the CUDA compiler (`nvcc`) and the Microsoft Visual C++ compiler (`cl.exe`). If `cl.exe` is not in your PATH, CUDA compilation will fail.

## Solution

### 1. Install Visual Studio Build Tools
- Download: https://visualstudio.microsoft.com/visual-cpp-build-tools/
- During installation, select **Desktop development with C++**.

### 2. Open the Correct Command Prompt
- Open the **x64 Native Tools Command Prompt for VS 2022** (or your installed version).
- This sets up the environment so `cl.exe` is available.

### 3. Build TurboNet
- In that prompt, navigate to your project directory:
  ```
  cd /d D:\TurboNet
  cargo build
  ```

### 4. (Optional) Add cl.exe to PATH
- You can add the path to `cl.exe` to your system PATH, but using the special command prompt is preferred.

---

## Troubleshooting
- If you see `nvcc fatal   : Cannot find compiler 'cl.exe' in PATH`, repeat steps 1 and 2.
- If you use WSL, you must build from Windows, not from inside WSL.

---

## For Developers
- See `build.rs` for the CUDA build step.
- If you want to skip CUDA, comment out the `nvcc` command in `build.rs` (not recommended for production).
