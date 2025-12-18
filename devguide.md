## Mastermind Development and Setup

### Quickstart:

- Install Rust/Cargo and Node/NPM.
- Install Yarn: `npm i --global yarn`.
- Run `yarn`.
- Run `yarn build:wasm`.
- Run `yarn build:grammar`.
- Run `yarn dev`, then follow the link to http://localhost:5173.

Pushes to _dev_ and _main_ are published to https://staging.mastermind.lostpixels.org and https://mastermind.lostpixels.org respectively.

### Overview:

This repository contains two main components: the compiler and the web IDE. There are GitHub Actions workflows which build, test, and deploy the web IDE (with bundled compiler) to Firebase Web Hosting.

#### Compiler

The `./compiler` subdirectory contains a Cargo (Rust) package, ensure Rust is installed.

The compiler codebase has two main entrypoints: `main.rs` and `lib.rs`, for the command-line and WASM compilation targets respectively. All other Rust source files are common between compilation targets.

Key files to look at:

- `tokeniser.rs`: tokenises the raw text files into Mastermind syntax tokens.
- `parser.rs`: parses strings of tokens into higher-level Mastermind clauses.
- `compiler.rs`: compiles the high-level clauses into a list of basic instructions akin to an intermediate representation (IR).
- `builder.rs`: takes the basic instructions from the compiler and builds the final Brainfuck program.

Some key commands:

(from within the `./compiler` subdirectory)

- `cargo run -- -h`: runs the command-line compiler module and displays command help information
- `cargo test`: runs the automated test suite
- `cargo build`: builds the command-line module
- `wasm-pack build`: builds the WASM module

#### Web IDE

The project root directory `package.json`/`yarn.lock` defines a Node package managed with Yarn. Most important commands or behaviours are defined as `npm run` or `yarn` scripts within `package.json`.

Ensure Node is installed, then ensure Yarn is installed with `npm i --global yarn`.

The web IDE is a SolidJS app using TypeScript/TSX, and Vite as a bundler. The text editing portions of the UI are provided by the _codemirror_ plugin, and syntax highlighting is defined in the included _lezer_ grammar: `./src/lexer/mastermind.grammar`.

Some key commands:

- `yarn`: installs npm packages
- `yarn build:wasm`: builds the compiler WASM module
- `yarn build:grammar`: compiles the lezer grammar to JS for use in codemirror
- `yarn dev`: runs the SolidJS app in a local Vite dev server
- `yarn build`: builds the SolidJS app
