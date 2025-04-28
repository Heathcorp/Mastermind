import { Portal } from "solid-js/web";
import { IoClose } from "solid-icons/io";
import { Component, createSignal, JSX, Show, For } from "solid-js";
import { useAppContext } from "../App";
import { makePersisted } from "@solid-primitives/storage";

const SettingsModal: Component<{ style?: JSX.CSSProperties }> = () => {
  const MemoryAllocationOptions: string[] = [
    "1D Mastermind",
    "2D Mastermind - Spiral",
    "2D Mastermind - Tiles",
    "2D Mastermind - Nearest",
  ];

  const [enabledOptimisations, setEnabledOptimisations] = makePersisted(
    createSignal<OptimisationSettings>({
      optimise_cell_clearing: false,
      optimise_constants: false,
      optimise_empty_blocks: false,
      optimise_generated_code: false,
      optimise_memory_allocation: false,
      optimise_unreachable_loops: false,
      optimise_variable_usage: false,
    }),
    { name: "optimisation_settings" }
  );

  const [twoDimensionalSettings, setTwoDimensionalSettings] = makePersisted(
    createSignal<TwoDimensionalSettings>({
      memory_allocation_method: 0,
      enable_2d_grid: false,
    }),
    { name: "two_dimensional_settings" }
  );

  const app = useAppContext()!;
  return (
    <Show when={app.settingsOpen()}>
      {/* The weirdest solid js feature, puts the component into the top level html body */}
      <Portal>
        <div
          class="readme-modal-container"
          onClick={() => app.setSettingsOpen(false)}
        >
            <div class="settings-modal" onClick={(e) => e.stopPropagation()}>
                <h3>SETTINGS</h3>
                <div class="settings-container" style={{"min-width": "300px"}}>
                <span class="settings-heading">Optimisations:
                    <span
                        class="text-button"
                        style={{"white-space": "nowrap"}}
                        onClick={() =>
                            setEnabledOptimisations((prev) => {
                                const entries = Object.entries(prev);
                                const b = entries.some(([, v]) => !v);
                                return Object.fromEntries(
                                    entries.map(([k]) => [k, b])
                                ) as unknown as OptimisationSettings;
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
                                {optimisationLabels[key as keyof OptimisationSettings]}
                            </label>
                        )}
                    </For>
                </form>
                <span>
                    <br/>
                <span class="settings-heading">2D GENERATION:</span>
              </span>
                <form>
                    <label class="row">
                        <input
                            type="checkbox"
                            name="Enable 2D Brainfuck"
                            id="enable_2d_grid"
                            checked={twoDimensionalSettings().enable_2d_grid}
                            onChange={(event) => {
                                const isChecked = event.target.checked;
                                setTwoDimensionalSettings({
                                    ...twoDimensionalSettings(),
                                    enable_2d_grid: isChecked,
                                    memory_allocation_method: !isChecked
                                        ? 0
                                        : twoDimensionalSettings().memory_allocation_method,
                                });
                            }}
                        />
                        Enable 2D Brainfuck
                    </label>
                    <label class="row">Memory Allocation</label>
                    <select
                        value={twoDimensionalSettings().memory_allocation_method}
                        disabled={!twoDimensionalSettings().enable_2d_grid}
                        onChange={(event) => {
                            const value = parseInt(
                                (event.target as HTMLSelectElement).value,
                                10
                            );
                            setTwoDimensionalSettings({
                                ...twoDimensionalSettings(),
                                memory_allocation_method: value,
                            });
                        }}
                    >
                        {MemoryAllocationOptions.map((option, index) => (
                            <option value={index}>{option}</option>
                        ))}
                    </select>
                </form>
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
                    onClick={() => app.setSettingsOpen(false)}
                />
            </div>
        </div>
      </Portal>
    </Show>
  );
};

export default SettingsModal;

interface OptimisationSettings {
  optimise_cell_clearing: boolean;
  optimise_constants: boolean;
  optimise_empty_blocks: boolean;
  optimise_generated_code: boolean;
  optimise_memory_allocation: boolean;
  optimise_unreachable_loops: boolean;
  optimise_variable_usage: boolean;
}

interface TwoDimensionalSettings {
  enable_2d_grid: boolean;
  memory_allocation_method: number;
}

export interface MastermindConfig
  extends OptimisationSettings,
    TwoDimensionalSettings {}

const optimisationLabels: Record<keyof OptimisationSettings, string> = {
  optimise_cell_clearing: "cell clearing",
  optimise_constants: "constants",
  optimise_empty_blocks: "empty blocks",
  optimise_generated_code: "generated code",
  optimise_memory_allocation: "memory allocations",
  optimise_unreachable_loops: "unreachable loops",
  optimise_variable_usage: "variable usage",
};
