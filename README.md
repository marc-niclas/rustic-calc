# 🦀 rustic-calc

<p align="center">
  <img src=".github/assets/logo.svg" alt="rustic-calc logo" width="300" />
</p>

[![CI](https://github.com/marc-niclas/rustic-calc/actions/workflows/ci.yml/badge.svg)](https://github.com/marc-niclas/rustic-calc/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A fast, terminal-based calculator written in Rust. `rustic-calc` provides a clean TUI (Terminal User Interface) for performing mathematical evaluations with a persistent history and real-time feedback.

## ✨ Features

- **Interactive TUI**: Built with `ratatui` for a smooth terminal experience.
- **Expression History**: Keep track of your previous calculations in a scrollable list.
- **Order of Operations**: Correctly handles operator precedence (PEMDAS/BODMAS).
- **Supported Operators**:
  - Addition (`+`)
  - Subtraction (`-`)
  - Multiplication (`*`)
  - Division (`/`)
  - Exponentiation (`^`)
- **Negative Numbers**: Supports unary minus for negative values.
- **Error Handling**: Clear error messages for malformed expressions.

## 🚀 Getting Started

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (2024 edition)

### Installation

Clone the repository and build the project using Cargo:

```bash
git clone https://github.com/BrightNight-Energy/rustic-calc.git
cd rustic-calc
cargo build --release
```

### Usage

Run the calculator:

```bash
cargo run
```

#### Controls

| Key | Action |
|-----|--------|
| `Enter` | Calculate the current expression |
| `Up Arrow` | Recall the last expression from history |
| `Left/Right` | Move the cursor within the input field |
| `Backspace` | Delete characters |
| `Ctrl+C` | Exit the application |

## 🧪 Testing

The project includes a comprehensive suite of unit tests for the core calculation engine.

```bash
cargo test
```

## 🛠️ Development

This project uses `pre-commit` to ensure code quality.

1. Install pre-commit hooks:
   ```bash
   prek install
   ```
2. The CI pipeline runs `clippy` and `rustfmt` on every push.

### Project Structure

- `src/lib.rs`: Core logic for tokenization and mathematical evaluation.
- `src/main.rs`: TUI implementation and event loop.
- `tests/`: Integration tests for the calculation engine.

## 📜 License

Distributed under the MIT License. See `LICENSE` for more information.

---
*Built with ❤️ using Rust and Ratatui.*
