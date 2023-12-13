import "./App.css";
import Divider from "./components/Divider";
import EditorPanel from "./panels/EditorPanel";
import InputPanel from "./panels/InputPanel";

// import init, { greet } from "../compiler/pkg";

function App() {
  // init().then(() => {
  //   greet("WASM!!!");
  // });

  return (
    <div id="window">
      <EditorPanel />
      <Divider />
      <div class="panel">
        <div class="panel">settings</div>
        <Divider />
        <div class="panel output-panel">output</div>
        <Divider />
        <InputPanel />
      </div>
    </div>
  );
}

export default App;
