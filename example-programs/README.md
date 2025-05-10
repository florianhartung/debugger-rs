# Example programs for testing the debugger
- multiple_prints: This program prints the letters ABCC in four separate calls.

# Structure
Every program directory contains:
- C source file
- A precompiled binary


# How to build & run
- `<PROGRAM_NAME>` is the name of the program to run, e.g. `multiple_prints`
- With Nix:
  ```sh
  nix-build -A <PROGRAM_NAME>
  cargo run -- ./result/bin/<PROGRAM_NAME>
  ```
- Without Nix using the precompiled binaries
  ```sh
  cargo run -- ./<PROGRAM_NAME>/<PROGRAM_NAME>
  ```
