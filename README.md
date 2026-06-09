# Carcassonne Simulation Engine with MCTS/UCT

A Rust-based implementation of the Carcassonne board game logic and simulation engine, featuring automated bots (Random and Greedy) and WebAssembly (WASM) support to run simulation logic directly in web browsers.

---

## 📂 Project Structure

The repository is organized as follows:

- **[`src/game_logic/`](file:///c:/Users/pietr/Documents/studia/metody_sztucznej_inteligencji2/carcassonne-uct/src/game_logic)**: Core implementation of the Carcassonne rules, including board state, scoring for different structures (roads, cities, cloisters, fields), and follower management.
- **[`src/engines/`](file:///c:/Users/pietr/Documents/studia/metody_sztucznej_inteligencji2/carcassonne-uct/src/engines)**: AI game playing engines/bots:
  - **`RandomEngine`**: Makes random valid moves.
  - **`GreedyEngine`**: Chooses moves prioritizing immediate score gains.
  - **`CarcassonneEngine`**: The shared trait definition for bots.
- **[`tileset_format/`](file:///c:/Users/pietr/Documents/studia/metody_sztucznej_inteligencji2/carcassonne-uct/tileset_format)**: A utility crate used to parse custom board tiles from files and build a serialized tileset configuration.
- **[`tilesets/standard/`](file:///c:/Users/pietr/Documents/studia/metody_sztucznej_inteligencji2/carcassonne-uct/tilesets/standard)**: Configuration and definition files (`.tile`) representing individual Carcassonne board tiles in an ASCII/Unicode-art grid layout.
- **[`tests/`](file:///c:/Users/pietr/Documents/studia/metody_sztucznej_inteligencji2/carcassonne-uct/tests)**: Integration tests validating the correctness of the game scoring rules.
- **[`benches/`](file:///c:/Users/pietr/Documents/studia/metody_sztucznej_inteligencji2/carcassonne-uct/benches)**: Performance benchmarks using `Criterion` to evaluate game simulation throughput.
- **[`build.rs`](file:///c:/Users/pietr/Documents/studia/metody_sztucznej_inteligencji2/carcassonne-uct/build.rs)**: Build script that compiles the standard tileset from raw definition files at compile time, serializing it with `postcard` and embedding it directly into the binary.
- **[`index.html`](file:///c:/Users/pietr/Documents/studia/metody_sztucznej_inteligencji2/carcassonne-uct/index.html)**: A web entry point designed to load the compiled WebAssembly package and run the game simulation in browser environment logs.

---

## 🛠️ Prerequisites

To build and run this project, make sure you have the following tools installed:

1. **Rust Toolchain**: Install via [rustup](https://rustup.rs/):
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```
2. **wasm-pack** (for compiling to WebAssembly):
   ```bash
   cargo install wasm-pack
   ```
3. **Static File Server** (to serve the WASM frontend locally, choose one):
   - **Python** (built-in): `python -m http.server`
   - **Node.js**: `npx serve` or `npm install -g serve`
   - **VS Code Extension**: Live Server

---

## 🚀 How to Start

### 1. Run the Native Console Simulation
Run the game locally in your terminal. This spins up a game simulation between a `RandomEngine` and a `GreedyEngine`, printing the game state and board progression to standard output:
```bash
cargo run
```

### 2. Run the WebAssembly (WASM) Version in Browser
To compile the project to WebAssembly and serve it in a browser:

1. Build the WASM package:
   ```bash
   wasm-pack build --target web
   ```
   *This generates a compiled `./pkg` folder with JavaScript wrappers and the compiled WASM binary.*

2. Start a local HTTP server in the root of the workspace directory:
   ```bash
   # Using Python:
   python -m http.server 8000
   
   # Or using Node.js:
   npx serve -l 8000
   ```

3. Open your browser and navigate to `http://localhost:8000`.
4. Open the developer console (`F12` or `Ctrl+Shift+I` / `Cmd+Option+I`) to view the logs outputted by `wasm_test_main()`.

### 3. Run Automated Tests
Verify scoring logic (cloister, road, field, city) using Rust's testing framework:
```bash
cargo test
```

### 4. Run Benchmarks
Run performance benchmarks comparing different engine playthroughs:
```bash
cargo bench
```

---

## 🎨 Tile Format Definitions

Tiles are represented as `5x5` grids in `.tile` files using a custom pixel layout map:

| Symbol | Description |
| :---: | :--- |
| `⦻⦻` | Nothing / Empty |
| `██` | Blockade |
| `··` | Field / Meadow |
| `░░` | Road |
| `✝⌂` | Cloister |
| `▒▒` | City |
| `▓▓` | City with Pennant |

For example, a standard road-shield-city tile (`CRFR.tile`) is defined as:
```text
▓▓▓▓▓▓▓▓██
▓▓········
▓▓··░░░░░░
▓▓··░░····
██··░░····
```
At compile-time, the `build.rs` script reads all tile files listed in `tilesets/standard/tileset.toml`, compiles them into `tileset.bin` in the cargo output directory, which is then compiled into the final executable dynamically via `include_bytes!`.
