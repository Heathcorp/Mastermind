# Running Mastermind

### 1. Install Rust
Install rust through the following website - https://www.rust-lang.org/tools/install

This will also install Cargo which is needed to build the project

### 2. Install Yarn
If not currently installed please install Yarn if unsure follow this guide - https://classic.yarnpkg.com/lang/en/docs/install

### 3. Install wasm-pack
Install using Yarn, Cargo or the following guide https://rustwasm.github.io/wasm-pack/installer/

### 4. Run Yarn Install
Install the Javascript dendencies by running
```bash
  yarn install
```

### 5. Build the grammar
Build the grammar using the following yarn command
```bash
    yarn build:grammar
```

### 6. Build Web Assembly Pack
Build Web Assembly Pack using the following yarn command
```bash
    yarn build:wasm
```

### 7. Run Dev Mode
Run Dev mode using the following command
```bash
    yarn dev
```