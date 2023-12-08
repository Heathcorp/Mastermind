import { createSignal } from "solid-js";
import "./App.css";

import init, { greet } from "../compiler/pkg/mastermind";

function App() {
  const [count, setCount] = createSignal(0);

  init().then(() => {
    greet("WASM!!!");
  });

  return (
    <>
      <h1>Vite + Solid</h1>
      <div class="card">
        <button onClick={() => setCount((count) => count + count + 1)}>
          count is {count()}
        </button>
        <p>
          Edit <code>src/App.tsx</code> and save to test HMR
        </p>
      </div>
      <p class="read-the-docs">
        Click on the Vite and Solid logos to learn more
      </p>
    </>
  );
}

export default App;
