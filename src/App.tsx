import "./App.css";
import Divider from "./components/Divider";
import Tab from "./components/Tab";

// import init, { greet } from "../compiler/pkg/mastermind";

import { EditorView } from "@codemirror/view";
import { EditorState } from "@codemirror/state";

function App() {
  // init().then(() => {
  //   greet("WASM!!!");
  // });

  return (
    <div id="window">
      <div class="panel">
        <div class="tab-bar">
          <Tab label="filename.txt" />
          <Tab label="filename.txt" selected />
          <Tab label="filename.txt" />
          <div class="tab-filler" />
        </div>
        <div class="code-panel">
          <textarea
            ref={(e) => {
              // TODO: figure this stuff out

              let startState = EditorState.create({
                doc: "Hello World",
              });

              let view = new EditorView({
                state: startState,
                parent: e,
              });
            }}
          ></textarea>
        </div>
      </div>
      <Divider />
      <div class="panel">2</div>
    </div>
  );
}

export default App;
