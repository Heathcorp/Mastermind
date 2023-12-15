import { Component, For } from "solid-js";
import Divider from "../components/Divider";
import { useAppContext } from "../App";

const SettingsPanel: Component = () => {
  const { fileStates } = useAppContext()!;

  return (
    <div class="panel settings-panel">
      <div class="row">
        <div class="row">
          entry file:
          <select>
            <For each={fileStates()}>
              {(file) => {
                console.log(file);
                return <option value={file.id}>{file.label}</option>;
              }}
            </For>
          </select>
        </div>
      </div>
      <div class="row">
        {/* button with 3 options (compile, run, or both) */}
        <div class="button" style={{ padding: 0 }}>
          <div class="row" style={{ gap: 0, "align-items": "stretch" }}>
            <div
              class="text-button"
              style={{ padding: "0.5rem" }}
              onClick={() => {
                console.log("compiling program");
              }}
            >
              compile program
            </div>
            <Divider />
            <div
              class="text-button"
              style={{ padding: "0.5rem" }}
              onClick={() => {
                console.log("running code");
              }}
            >
              run code
            </div>
          </div>
          <Divider />
          <div
            style={{ "text-align": "center", padding: "0.5rem" }}
            onClick={() => {
              console.log("compiling and running");
            }}
          >
            compile & run
          </div>
        </div>
      </div>
    </div>
  );
};

export default SettingsPanel;
