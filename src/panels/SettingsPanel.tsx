import { Component, For, createEffect, createSignal, on } from "solid-js";
import Divider from "../components/Divider";
import { useAppContext } from "../App";
import { makePersisted } from "@solid-primitives/storage";
import { AiFillGithub } from "solid-icons/ai";
import { FiCopy } from "solid-icons/fi";

import "./settings.css";
const SettingsPanel: Component = () => {
  const app = useAppContext()!;

  const [enabledOptimisations, setEnabledOptimisations] = makePersisted(
    createSignal<MastermindConfig>({
      optimise_cell_clearing: false,
      optimise_constants: false,
      optimise_empty_blocks: false,
      optimise_generated_code: false,
      optimise_memory_allocation: false,
      optimise_unreachable_loops: false,
      optimise_variable_usage: false,
    })
  );

  createEffect(
    on([app.fileStates, app.entryFile], () => {
      if (app.fileStates().length && !app.entryFile()) {
        app.setEntryFile(app.fileStates()[0]?.id);
      }
    })
  );

  const onRun = () => {
    const code = app.compiledCode();
    if (!code) return;
    const result = app.runCode(code);
    app.setOutput(result);
  };

  const onCompile = () => {
    const entryFileId = app.entryFile();
    const result =
      (!!entryFileId && app.compile(entryFileId, enabledOptimisations())) ||
      undefined;
    console.log(result);
    app.setOutput(result);
  };

  return (
    <div class="panel" style={{ "flex-direction": "row" }}>
      <div class="panel settings-panel">
        <div class="row settings-container">
          {/* entry file selection */}
          <div class="row">
            entry file:
            <select
              value={app.entryFile()}
              onChange={(e) => app.setEntryFile(e.target.value)}
            >
              <For each={app.fileStates()}>
                {(file) => {
                  return <option value={file.id}>{file.label}</option>;
                }}
              </For>
            </select>
          </div>
          {/* button with 3 options (compile, run, or both) */}
          <div class="button" style={{ padding: 0 }}>
            <div class="row" style={{ gap: 0, "align-items": "stretch" }}>
              <div
                class="text-button"
                style={{ padding: "0.5rem" }}
                onClick={onCompile}
              >
                compile program
              </div>
              <Divider />
              <div
                classList={{
                  "text-button": true,
                  "text-button-disabled": !app.compiledCode(),
                }}
                style={{ padding: "0.5rem" }}
                onClick={onRun}
              >
                run code
              </div>
            </div>
            <Divider />
            <div
              style={{ "text-align": "center", padding: "0.5rem" }}
              onClick={() => {
                onCompile();
                onRun();
              }}
            >
              compile & run
            </div>
          </div>
          {/* misc options and markers */}
          <div
            class="row button"
            classList={{
              row: true,
              button: true,
              disabled: !app.compiledCode(),
            }}
            style={{ cursor: "copy", "align-items": "center" }}
            onClick={() => {
              const code = app.compiledCode();
              if (!code) return;
              window.navigator.clipboard
                .writeText(code)
                .then(() =>
                  window.alert("Compiled Brainfuck copied to clipboard!")
                );
            }}
          >
            <FiCopy />
            compiled code
            {app.compiledCode() && ` (${app.compiledCode()?.length} bytes)`}
          </div>
        </div>
        <Divider />
        <div class="row settings-container">
          <span>
            <span class="settings-heading">Optimisations:</span>
            <span
              class="text-button"
              onClick={() =>
                setEnabledOptimisations((prev) => {
                  const entries = Object.entries(prev);
                  const b = entries.some(([, v]) => !v);
                  return Object.fromEntries(
                    entries.map(([k]) => [k, b])
                  ) as unknown as MastermindConfig;
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
                <label
                  for={key}
                  class="row"
                  style={{ "align-items": "flex-end" }}
                >
                  <input
                    type="checkbox"
                    checked={enabled}
                    name={key}
                    id={key}
                  />
                  {configLabels[key as keyof MastermindConfig]}
                </label>
              )}
            </For>
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
          <AiFillGithub />
        </a>
      </div>
    </div>
  );
};

export default SettingsPanel;

export interface MastermindConfig {
  optimise_cell_clearing: boolean;
  optimise_constants: boolean;
  optimise_empty_blocks: boolean;
  optimise_generated_code: boolean;
  optimise_memory_allocation: boolean;
  optimise_unreachable_loops: boolean;
  optimise_variable_usage: boolean;
}

const configLabels: Record<keyof MastermindConfig, string> = {
  optimise_cell_clearing: "cell clearing",
  optimise_constants: "constants",
  optimise_empty_blocks: "empty blocks",
  optimise_generated_code: "generated code",
  optimise_memory_allocation: "memory allocations",
  optimise_unreachable_loops: "unreachable loops",
  optimise_variable_usage: "variable usage",
};
