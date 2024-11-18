# 🍷 cargo-sample

A Cargo tool for bootstrapping new projects from repository examples.
Always sample before you try.

<p align="center">
  <img alt="instructional video" src="https://github.com/user-attachments/assets/0b5559dd-c4d3-4773-990e-c4e556bfab43">
</p>


## 🧑‍🔧 Installation

```bash
cargo install cargo-sample
```

## 💡 Usage

Create a new project based on an example from any git repository that has a examples folder

e.g.
```bash
mkdir demo && cd demo
cargo sample https://github.com/tokio-rs/axum.git

----- or ------------

cargo sample https://github.com/tokio-rs/axum.git demo
```

This will:
1. Clone the repository to a temporary folder
2. Find the examples in the `examples/` directory
3. Prompt you to select an example
3. Create a new project with the example's content on the demo folder

## 📈 TODO

- [] search for examples everywhere in the repo
- [] replace local dependencies in toml of examples for crates io dependencies
- [] allow sampling on already existing dir, kind of importing an example to my local project
