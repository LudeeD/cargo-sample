# cargo-sample

A Cargo tool for bootstrapping new projects from repository examples.

## Installation

```bash
cargo install cargo-sample
```

## Usage

Create a new project based on an example from any GitHub repository:

```bash
cargo sample <GIT PROJECT>
```

This will:
1. Clone the repository
2. Find the examples in the `examples/` directory
3. Prompt you to select an example
3. Create a new project with the example's content
