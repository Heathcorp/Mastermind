import { createSignal } from "solid-js";
import "./App.css";

import init, { greet } from "../compiler/pkg/mastermind";

function App() {
  init().then(() => {
    greet("WASM!!!");
  });

  return (
    <>
      <h1>Mastermind</h1>
      <h3>Language and compiler for brainfuck programs</h3>
      <h4>Coming soon</h4>
    </>
  );
}

export default App;
