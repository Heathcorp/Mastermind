import {
  Component,
  For,
  createEffect,
  createSignal,
  JSX,
  Show,
} from "solid-js";
import Divider from "../components/Divider";
import { useAppContext } from "../App";
import { makePersisted } from "@solid-primitives/storage";
import { AiFillGithub, AiOutlineStop } from "solid-icons/ai";
import { FiCopy, FiSave } from "solid-icons/fi";
import { IoHelpCircle } from "solid-icons/io";

import "./settings.css";
import { Portal } from "solid-js/web";
import { SolidMarkdown } from "solid-markdown";
import readmeContent from "../../README.md?raw";
import { IoClose } from "solid-icons/io";
import remarkGfm from "remark-gfm";
import JSZip from "jszip";
import downloadBlob from "../utils/downloadBlob";

const MemoryAllocationOptions : string[] = ['1D Mastermind' , '2D Mastermind - Spiral' , '2D Mastermind - Tiles' , '2D Mastermind - Nearest']
const SettingsPanel: Component<{ style?: JSX.CSSProperties }> = (props) => {
  const app = useAppContext()!;

  const [enabledOptimisations, setEnabledOptimisations] = makePersisted(
    createSignal<MastermindOptimisations>({
      optimise_cell_clearing: false,
      optimise_constants: false,
      optimise_empty_blocks: false,
      optimise_generated_code: false,
      optimise_memory_allocation: false,
      optimise_unreachable_loops: false,
      optimise_variable_usage: false,
    }),
    { name: "mastermind_compiler_optimisations" }
  );

  const [settings, setSettings] = makePersisted(
      createSignal<MastermindSettings>({
        memory_allocation_method: 0,
        enable_2d_grid: false,
      }),
      { name: "mastermind_compiler_settings" }
  );

  createEffect(() => {
    const fileStates = app.fileStates;
    const entryFile = app.entryFile();
    if (app.fileStates.length && !entryFile) {
      app.setEntryFile(fileStates[0]?.id);
    }
  });

  const onRun = async () => {
    // TODO: error handling here? is it needed?
    const output = app.output();
    if (output?.type !== "BF") return;
    await app.run(output.content, settings().enable_2d_grid);
  };

  const onCompile = async () => {
    const entryFileId = app.entryFile();
    if (!entryFileId) return;

    await app.compile(entryFileId, {
        ...enabledOptimisations(),
        ...settings()
    });
  };

  createEffect(() => {
    console.log(app.fileStates);
  });

  const zipAndSave = async () => {
    const zip = new JSZip();
    app.fileStates.forEach((fileState) => {
      const blob = new Blob([fileState.editorState.doc.toString()], {
        type: "text/plain",
      });
      zip.file(fileState.label, blob);
    });
    await zip.generateAsync({ type: "blob" }).then((x) => {
      downloadBlob(x);
    });
  };

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
              classList={{
                disabled: !app.output(),
              }}
              style={{cursor: "copy", "align-items": "center"}}
              onClick={() => {
                const output = app.output();
                if (!output) return;
                window.navigator.clipboard
                    .writeText(output.content)
                    .then(() => window.alert("Output copied to clipboard!"));
              }}
          >
            <FiCopy/>
            {
              {
                ["BF"]: "compiled code",
                ["ERROR"]: "error output",
                ["OUTPUT"]: "code output",
                ["LIVE_OUTPUT"]: "live output",
              }[app.output()?.type ?? "OUTPUT"]
            }

            {/* TODO: convert this to be more correct, <Show/> or something? */}
            {app.output() && ` (${app.output()?.content.length} bytes)`}
          </div>
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
          <div
              class="row button"
              style={{gap: 0, "text-decoration": "none", color: "inherit"}}
              onClick={async () => await zipAndSave()}
          >
            <FiSave style={{"margin-right": "8px"}}/>
            Zip All & Save
          </div>
        </div>
        <Divider/>
        <div class="row settings-container">
          <span>
            <span class="settings-heading">Optimisations:</span>
            <span
                class="text-button"
                style={{"white-space": "nowrap"}}
                onClick={() =>
                    setEnabledOptimisations((prev) => {
                      const entries = Object.entries(prev);
                      const b = entries.some(([, v]) => !v);
                      return Object.fromEntries(
                          entries.map(([k]) => [k, b])
                      ) as unknown as MastermindOptimisations;
                      // trust me on this one typescript
                    })
                }
            >
              (toggle all)
            </span>
          </span>
          <form
              onChange={(e) => {
                const target = e.target as HTMLInputElement;
                setEnabledOptimisations((prev) => ({
                  ...prev,
                  [target.name]: !!target.checked,
                }));
              }}
          >
            <For each={Object.entries(enabledOptimisations())}>
              {([key, enabled]: [string, boolean]) => (
                  <label class="row">
                    <input
                        type="checkbox"
                        checked={enabled}
                        name={key}
                        id={key}
                    />
                    {optimisationLabels[key as keyof MastermindOptimisations]}
                  </label>
              )}
            </For>
          </form>
        </div><Divider/>
        <div class="row settings-container">
          <span>
            <span class="settings-heading">Settings:</span>
          </span>
            <form>
                <label class="row">
                    <input
                        type="checkbox"
                        name='Enable 2D Brainfuck'
                        id='enable_2d_grid'
                        checked={settings().enable_2d_grid}
                        onChange={(event) => {
                            const isChecked = event.target.checked;
                            setSettings({
                                ...settings(),
                                enable_2d_grid: isChecked,
                                memory_allocation_method: !isChecked ? 0 : settings().memory_allocation_method,
                            });
                        }}
                    />
                    Enable 2D Brainfuck
                </label>
                <label class="row">
                    Memory Allocation
                </label>
                <select
                    value={settings().memory_allocation_method}
                    disabled={!settings().enable_2d_grid}
                    onChange={(event) => {
                        const value = parseInt((event.target as HTMLSelectElement).value, 10);
                        setSettings({...settings(), memory_allocation_method: value});
                    }}
                >
                    {MemoryAllocationOptions.map((option, index) => (
                        <option value={index}>
                            {option}
                        </option>
                    ))}
                </select>
            </form>
        </div>
      </div>
        {/* <Divider /> */}
        {/* social media links, currently only github */}
        <div class="socials">
            <a
                class="socials-icon text-button"
                href="https://github.com/Heathcorp/Mastermind"
                target="_blank"
            >
            <AiFillGithub title="Git repository"/>
        </a>
        <a
            class="socials-icon text-button"
            style={{"font-size": "2.25rem"}}
            target="_blank"
            onClick={() => app.setHelpOpen(true)}
        >
          <IoHelpCircle title="help"/>

          <Show when={app.helpOpen()}>
            {/* The weirdest solid js feature, puts the component into the top level html body */}
            <Portal>
              <div
                  class="readme-modal-container"
                  onClick={() => app.setHelpOpen(false)}
              >
                <div class="readme-modal" onClick={(e) => e.stopPropagation()}>
                  <div class="markdown-container">
                    <SolidMarkdown remarkPlugins={[remarkGfm]}>
                      {readmeContent}
                    </SolidMarkdown>
                  </div>
                  <IoClose
                      title="close help"
                      class="text-button"
                      style={{
                        "font-size": "1.5rem",
                        position: "absolute",
                        right: "1rem",
                        top: "1rem",
                      }}
                      onClick={() => app.setHelpOpen(false)}
                  />
                </div>
              </div>
            </Portal>
          </Show>
        </a>
      </div>
    </div>
  );
};

export default SettingsPanel;

interface MastermindOptimisations {
  optimise_cell_clearing: boolean;
  optimise_constants: boolean;
  optimise_empty_blocks: boolean;
  optimise_generated_code: boolean;
  optimise_memory_allocation: boolean;
  optimise_unreachable_loops: boolean;
  optimise_variable_usage: boolean;
}

interface MastermindSettings {
  enable_2d_grid: boolean;
  memory_allocation_method: number;
}

export interface MastermindConfig extends MastermindOptimisations, MastermindSettings {}

const optimisationLabels: Record<keyof MastermindOptimisations, string> = {
  optimise_cell_clearing: "cell clearing",
  optimise_constants: "constants",
  optimise_empty_blocks: "empty blocks",
  optimise_generated_code: "generated code",
  optimise_memory_allocation: "memory allocations",
  optimise_unreachable_loops: "unreachable loops",
  optimise_variable_usage: "variable usage",
};