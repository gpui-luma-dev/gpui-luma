# Rust Coding Guidelines

**Goal:** Keep code maintainable, readable, and AI-friendly even as the project grows.

### 1. File Size & Modularity

- **Maximum recommended file size:** ≤ 300 lines of code (LOC) per `.rs` file.
- Break files aggressively at natural boundaries.
- Preferred structure:
  - One primary public type/trait per file when possible
  - Related helper types, errors, and private logic can live in the same file **only** if total < 250 LOC
- Use Rust’s module system liberally:
  - Create subdirectories and `mod.rs` (or inline `mod` blocks) early
  - Use private `mod` blocks inside files for local organization before extracting to new files

**Rule of thumb:** If you need to scroll more than one screen to understand the main purpose of the file, split it.

### 2. Readability Principles

- Prioritize **clarity over cleverness**
- Use **newtypes** and domain-specific types generously
- Prefer explicit, self-documenting code over heavy comments
- Keep functions short (ideally < 50 LOC)
- Limit lifetime annotations to where truly necessary; hide them behind owned types or `Arc` when reasonable
- Use clear error types (`thiserror`) and proper error propagation
- Embrace `#[derive(...)]` (Debug, Clone, PartialEq, Eq, etc.) and builder patterns

### 3. AI-Assisted Development Rules

- Write **intent-first** code and comments. AI performs best when it understands your goal.
- Structure code so it is easy for AI to:
  - Understand context quickly
  - Refactor across modules
  - Generate correct boilerplate (async, locking, error handling)
- When asking AI for changes, provide:
  - Clear requirements
  - Relevant module boundaries
  - Performance/safety constraints
- Always review AI-generated code for:
  - Unnecessary lifetime complexity
  - Overly generic trait bounds
  - Rust idioms

### 4. Tooling & Formatting (Non-Negotiable)

- Run `cargo fmt` on every save
- Run `cargo clippy --fix --all-targets` regularly
- Enable strict warnings in `Cargo.toml`:
  ```toml
  [lints.rust]
  unsafe_code = "forbid"