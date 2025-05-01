import {
  Component,
  For,
  createEffect,
  JSX,
  Show,
} from "solid-js";
import Divider from "../components/Divider";
import { useAppContext } from "../App";
import { AiOutlineStop } from "solid-icons/ai";
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

  // const zipAndSave = async () => {
  //   const zip = new JSZip();
  //   app.fileStates.forEach((fileState) => {
  //     const blob = new Blob([fileState.editorState.doc.toString()], {
  //       type: "text/plain",
  //     });
  //     zip.file(fileState.label, blob);
  //   });
  //   await zip.generateAsync({ type: "blob" }).then((x) => {
  //     downloadBlob(x);
  //   });
  // };

  return (
    <div class="panel" style={{ "flex-direction": "row", ...props.style }}>
      <div class="panel settings-panel">
        <div class="row settings-container">
          {/* entry file selection */}
          <label class="row">
            entry file:
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
          {/* button with 3 options (compile, run, or both) */}
          <div style={{position: "relative"}}>
            <div
                classList={{button: true, disabled: app.busy()}}
                style={{padding: 0}}
            >
              <div class="row" style={{gap: 0, "align-items": "stretch"}}>
                <div
                    classList={{
                      "text-button": true,
                      "text-button-disabled": app.busy(),
                    }}
                    style={{padding: "0.5rem"}}
                    onClick={!app.busy() ? onCompile : undefined}
                >
                  compile program
                </div>
                <Divider/>
                <div
                    classList={{
                      "text-button": true,
                      "text-button-disabled":
                      // TODO: make a specific compiled code signal like we used to, basically store the last successful compilation
                          app.busy() || app.output()?.type !== "BF",
                    }}
                    style={{padding: "0.5rem"}}
                    onClick={
                      !app.busy() && app.output()?.type === "BF"
                          ? onRun
                          : undefined
                    }
                >
                  run code
                </div>
              </div>
              <Divider/>
              <div
                  style={{"text-align": "center", padding: "0.5rem"}}
                  onClick={
                    !app.busy()
                        ? async () => {
                          await onCompile();
                          // technically this second await is pointless
                          await onRun();
                        }
                        : undefined
                  }
              >
                compile & run
              </div>
            </div>
            {/* status overlay on the button */}
            {app.status() !== "IDLE" && (
                <div class="button-status-overlay">
                  <div class="button-status-text">
                    {
                      {
                        ["COMPILING"]: "compiling program",
                        ["RUNNING"]: "running code",
                        ["INPUT_BLOCKED"]: "waiting for input",
                        ["IDLE"]: null,
                      }[app.status()]
                    }
                    <Show when={app.status() !== "IDLE"}>
                      <div
                          onClick={() => app.restartWorker()}
                          title="kill brainfuck process"
                          class="stop-button"
                      >
                        <AiOutlineStop/>
                      </div>
                    </Show>
                  </div>
                </div>
            )}
          </div>
          {/* misc options and markers */}
          <div
              class="row button"
              style={{gap: 0}}
              classList={{"button-selected": app.enableBlockingInput()}}
              onClick={() => app.setEnableBlockingInput((prev) => !prev)}
          >
            blocking input [
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
        <Divider/>
      </div>
    </div>
  );
};

export default CompilerPanel;