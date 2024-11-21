# 🍷 cargo-sample

A Cargo tool for bootstrapping new projects from repository examples.
Always sample before you try.

<p align="center">
  <img alt="instructional video" src="https://github.com/user-attachments/assets/9ec68cbc-2c4b-4e3f-8989-50a79ec839b3">

</p>


## 🧑‍🔧 Installation

```bash
cargo install cargo-sample
```

## 💡 Usage

Create a new project based on an example from any cargo package that has a repo with examples

e.g.
```bash
mkdir demo-folder && cd demo-folder
cargo sample axum

----- or ------------

cargo sample axum demo-folder
```

This will:
1. Figure out the latest syable release of the crate
2. Clone the repository to a temporary folder, checkout the proper branch
3. Find the examples in the `examples/` directory
4. Prompt you to select an example
3. Create a new project with the example's content on the demo folder

## 📈 TODO

- [ ] search for examples everywhere in the repo
- [X] replace local dependencies in toml of examples for crates io dependencies
- [ ] allow sampling on already existing dir, kind of importing an example to my local project
