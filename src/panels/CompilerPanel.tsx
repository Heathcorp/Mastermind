import { Component, For, createEffect, JSX } from "solid-js";
import { useAppContext } from "../App";
import { AiOutlineStop } from "solid-icons/ai";
import { FaSolidPlay } from "solid-icons/fa";
// import { FiSave } from "solid-icons/fi";
// import downloadBlob from "../utils/downloadBlob";
// import JSZip from "jszip";

import "./settings.css";

const CompilerPanel: Component<{ style?: JSX.CSSProperties }> = (props) => {
  const app = useAppContext()!;

  createEffect(() => {
    const fileStates = app.fileStates;
    const entryFile = app.entryFile();
    if (app.fileStates.length && !entryFile) {
      app.setEntryFile(fileStates[0]?.id);
    }
  });

  const onRun = async () => {
    // TODO: error handling here? is it needed?
    const code = app.brainfuck();
    if (!code.text) return;
    app.setOutput({ type: "OUTPUT", content: "" });
    await app.run(code.text, app.config().enable_2d_grid);
  };

  const onCompile = async () => {
    const entryFileId = app.entryFile();
    if (!entryFileId) return;

    await app.compile(entryFileId, app.config());
  };

  createEffect(() => {
    console.log(app.fileStates);
  });

  return (
    <div class="panel" style={{ "flex-direction": "row", ...props.style }}>
      <div class="panel settings-panel">
        <div class="row settings-container">
          {/* entry file selection */}
          <label class="top-label" style={{ "padding-top": "0.5rem" }}>
            File:
            <select
              value={app.entryFile()}
              onChange={(e) => app.setEntryFile(e.target.value)}
            >
              {/* TODO: fix an issue with file renaming not updating this list */}
              <For each={app.fileStates}>
                {(file) => <option value={file.id}>{file.label}</option>}
              </For>
            </select>
          </label>
        </div>
        <div>
          <div
            classList={{ button: true, disabled: app.busy() }}
            style={{ padding: 0 }}
          >
            <div
              classList={{
                "text-button": true,
                "run-button": true,
                "text-button-disabled": app.busy(),
              }}
              style={{
                padding: "0.5rem",
                "max-width": "6rem",
                "min-width": "5.5rem",
              }}
              onClick={!app.busy() ? onCompile : undefined}
            >
              Compile File
            </div>
          </div>
        </div>
        <div>
          {app.busy() ? (
            <div
              classList={{ "run-button": true, button: true }}
              onClick={() => {
                app.restartWorker();
              }}
              title="kill brainfuck process"
            >
              <div class="stop-button">
                <AiOutlineStop style={{ "margin-right": "8px" }} />
              </div>
              Stop Code
            </div>
          ) : (
            <div
              classList={{ "run-button": true, button: true }}
              style={{}}
              onClick={onRun}
            >
              <div class="start-button">
                <FaSolidPlay
                  style={{ "margin-right": "8px", "padding-top": "0.2rem" }}
                />
              </div>
              Run Code
            </div>
          )}
        </div>
        <div>
          {/* misc options and markers */}
          <div
            class="row button"
            style={{
              gap: 0,
              "font-size": "0.9rem",
              "min-height": "1.5rem",
              "min-width": "11rem",
              "text-align": "center",
            }}
            classList={{ "button-selected": app.enableBlockingInput() }}
            onClick={() => app.setEnableBlockingInput((prev) => !prev)}
          >
            Blocking Input [
            {app.enableBlockingInput() ? (
              <div class="positive-text">enabled</div>
            ) : (
              <div class="negative-text">disabled</div>
            )}
            ]
          </div>
          {/*<div*/}
          {/*    class="row button"*/}
          {/*    style={{gap: 0, "text-decoration": "none", color: "inherit"}}*/}
          {/*    onClick={async () => await zipAndSave()}*/}
          {/*>*/}
          {/*  <FiSave style={{"margin-right": "8px"}}/>*/}
          {/*  Zip All & Save*/}
          {/*</div>*/}
        </div>
      </div>
    </div>
  );
};

export default CompilerPanel;
