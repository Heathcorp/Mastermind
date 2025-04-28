import {
  Component,
  createSignal,
  createContext,
  useContext,
  Accessor,
  Setter,
  createEffect,
  on,
  onMount,
} from "solid-js";

// special vite syntax for web workers to be included in the bundle
import CompilerWorker from "./worker.ts?worker";

import { EditorState } from "@codemirror/state";
import { EditorView } from "@codemirror/view";
import { v4 as uuidv4 } from "uuid";

import divisorsExample from "./assets/divisors_example.mmi?raw";
import printExample from "./assets/print.mmi?raw";
import primeExample from "./assets/prime_1_to_100.mmi?raw";
import christmasTreeExample from "./assets/christmas_trees.mmi?raw";
import brainfuckExample from "./assets/brainfuck.mmi?raw";

import "./App.css";
import Divider from "./components/Divider";
import EditorPanel from "./panels/EditorPanel";
import InputPanel from "./panels/InputPanel";
import SideBar from "./panels/SideBar.tsx";

import OutputPanel from "./panels/OutputPanel";
import SettingsPanel, {MastermindConfig} from "./panels/SettingsPanel";
import { defaultExtensions } from "./misc";
import { makePersisted } from "@solid-primitives/storage";
import { createStore } from "solid-js/store";

const AppContext = createContext<AppContextProps>();

// update this when you want the user to see new syntax
const MIGRATION_VERSION = 4;

const App: Component = () => {
  const [version, setVersion] = makePersisted(createSignal<number>(), {
    name: "mastermind_version",
  });
  const [helpOpen, setHelpOpen] = createSignal(false);
  const [settingsOpen, setSettingsOpen] = createSignal(false);
  createEffect(
    on([version], () => {
      const v = version();
      if (v !== MIGRATION_VERSION) {
        if (v) {
          window.alert(
            "Version has changed since last load, new example programs will be loaded.\nNote: your old programs may now have incorrect syntax"
          );
        }
        loadExampleFiles();
        setVersion(MIGRATION_VERSION);
        setHelpOpen(true);
      }
    })
  );
  // global signals and functions and things
  // to the program this is just a solidjs signal, all of this extra stuff is just for persistence
  const [entryFile, setEntryFile] = makePersisted(createSignal<string>(), {
    name: "mastermind_entry_file",
  });
  const [fileStates, setFileStates] = makePersisted(
    createStore<FileState[]>([]),
    {
      name: "mastermind_files",
      serialize: (fileStates: FileState[]) =>
        JSON.stringify(
          fileStates.map((fileState) => ({
            id: fileState.id,
            label: fileState.label,
            rawText: fileState.editorState.doc.toString(),
          }))
        ),
      deserialize: (data: string): FileState[] => {
        let rawParsed: {
          id: string;
          label: string;
          rawText: string;
        }[] = JSON.parse(data);

        return rawParsed.map((storedState) => ({
          id: storedState.id,
          label: storedState.label,
          editorState: EditorState.create({
            doc: storedState.rawText,
            extensions: [
              ...defaultExtensions,
              EditorView.updateListener.of((e) => {
                // this basically saves the editor every time it updates, this may be inefficient
                saveFileState(storedState.id, e.state);
              }),
            ],
          }),
        }));
      },
    }
  );

  createEffect(
    on([() => fileStates], () => {
      if (!fileStates.length) {
        // there are no files, initialise to the example files (divisors of 1 to 100)
        loadExampleFiles();
      }
    })
  );

  const loadExampleFiles = () => {
    const defaultFileId = uuidv4();
    setFileStates((prev) => [
      ...[
        {
          id: defaultFileId,
          label: "divisors_example.mmi",
          rawText: divisorsExample,
        },
        { id: uuidv4(), label: "print.mmi", rawText: printExample },
        { id: uuidv4(), label: "prime_1_to_100.mmi", rawText: primeExample },
        {
          id: uuidv4(),
          label: "christmas_trees.mmi",
          rawText: christmasTreeExample,
        },
        {
          id: uuidv4(),
          label: "brainfuck.mmi",
          rawText: brainfuckExample,
        },
      ].map((rawState) => ({
        // This could probably be common function, duplicate code of above deserialization and file creation functions (TODO: refactor)
        id: rawState.id,
        label: rawState.label,
        editorState: EditorState.create({
          doc: rawState.rawText,
          extensions: [
            ...defaultExtensions,
            EditorView.updateListener.of((e) => {
              // this basically saves the editor every time it updates, this may be inefficient
              saveFileState(rawState.id, e.state);
            }),
          ],
        }),
      })),
      ...prev,
    ]);
    setEntryFile(defaultFileId);
  };

  const createFile = async (file?: File) => {
    const newId = uuidv4();
    let rawText: string | undefined;
    if (file) rawText = await file.text();
    setFileStates((prev) => [
      ...prev,
      {
        id: newId,
        label: file?.name ?? `untitled_${prev.length}`,
        editorState: EditorState.create({
          doc: rawText,
          extensions: [
            ...defaultExtensions,
            EditorView.updateListener.of((e) => {
              // this basically saves the editor every time it updates, this may be inefficient
              saveFileState(newId, e.state);
            }),
          ],
        }),
      },
    ]);
    return newId;
  };
  const deleteFile = (id: string) => {
    setFileStates((prev) => prev.filter((f) => f.id !== id));
  };
  const saveFileState = (id: string, state: EditorState) => {
    setFileStates(
      (file) => file.id === id,
      "editorState",
      () => state
    );
  };
  const setFileLabel = (id: string, label: string) => {
    setFileStates(
      (file) => file.id === id,
      "label",
      () => label
    );
  };
  const reorderFiles = (from: string, to: string | null) => {
    if (from === to) return;
    setFileStates((prev) => {
      const newArray = [...prev];
      const fromIndex = newArray.findIndex((f) => f.id === from);
      if (fromIndex === -1) return prev;
      const removedFile = newArray.splice(fromIndex, 1)[0]!;
      // if to id is null then just push it to the end (bit of a hack but better than a magic string id for the tab filler div?)
      const toIndex =
        to === null ? newArray.length : newArray.findIndex((f) => f.id === to);
      if (toIndex === -1) return prev;
      // insert file into correct position
      newArray.splice(toIndex, 0, removedFile);

      return newArray;
    });
  };

  const [busy, setBusy] = createSignal<boolean>(false);
  const [status, setStatus] = createSignal<
    "COMPILING" | "RUNNING" | "INPUT_BLOCKED" | "IDLE"
  >("IDLE");

  let compilerWorker = new CompilerWorker();
  const restartWorker = () => {
    // kill the old web worker and replace it with a new one
    console.log("Terminating and restarting web worker");
    compilerWorker.terminate();
    compilerWorker = new CompilerWorker();
    setBusy(false);
    setStatus("IDLE");
    setInputCallback(undefined);
    setInput((prev) => ({ text: prev.text, amountRead: null }));
  };

  const compile = (entryFileId: string, config: MastermindConfig) => {
    return new Promise<string>((resolve, reject) => {
      let entryFileName: string | undefined;
      const fileMap = Object.fromEntries(
        fileStates.map((file) => {
          if (file.id === entryFileId) entryFileName = file.label;
          return [file.label, file.editorState.doc.toString()];
        })
      );
      if (!entryFileName) {
        reject();
        return;
      }

      // surely there is a library for this kind of thing, transactionify messages or something, maybe make one?
      const transaction = uuidv4();
      const callback = (e: {
        data: { transaction: string; success: boolean; message: string };
      }) => {
        if (transaction !== e.data.transaction) return;

        removeCallback();
        setBusy(false);
        if (e.data.success) {
          setOutput({ type: "BF", content: e.data.message });
          setStatus("IDLE");
          resolve(e.data.message);
        } else {
          setOutput({ type: "ERROR", content: e.data.message });
          setStatus("IDLE");
          reject(e.data.message);
        }
      };
      compilerWorker.addEventListener("message", callback);
      const removeCallback = () =>
        compilerWorker.removeEventListener("message", callback);

      setStatus("COMPILING");
      setBusy(true);
      // post the message after setting up the listener for paranoia
      compilerWorker.postMessage({
        command: "COMPILE",
        transaction,
        arguments: {
          fileMap,
          entryFileName,
          config,
        },
      });

      // TODO: maybe make a timeout to auto-reject
      // probably more important for the run code
    });
  };

  const run = (code: string, enable_2d_grid: boolean) => {
    return new Promise<string>((resolve, reject) => {
      const transaction = uuidv4();
      const callback = (e: {
        data: {
          transaction: string;
          success: boolean;
          message: string;
          command?: string;
          arguments?: { byte?: number; transaction?: string };
        };
      }) => {
        if (transaction !== e.data.transaction) return;

        if (e.data.command === "OUTPUT_BYTE") {
          // TODO: refactor so the output is a uint8array and we can do multi-byte chars, also so as to avoid ! here
          const char = String.fromCharCode(e.data.arguments!.byte!);
          // if this is the first byte back from the BVM, reset the output buffer and start adding on characters
          setOutput((prev) => ({
            type: "LIVE_OUTPUT",
            content: prev?.type === "LIVE_OUTPUT" ? prev.content + char : char,
          }));
          return;
        } else if (e.data.command === "REQUEST_INPUT") {
          const inputTransaction = e.data.arguments!.transaction!;
          // get/wait for input from the user, use new transaction id to send back in input byte to the worker

          const sendInputByte = (b: number) => {
            compilerWorker.postMessage({
              transaction: inputTransaction,
              command: "INPUT_BYTE",
              arguments: { byte: b },
            });
          };

          // TODO: make this a uint8 array instead of chars
          const c = popNextInputCharacter();
          if (c) {
            sendInputByte(c.charCodeAt(0));
          } else if (enableBlockingInput()) {
            // make a blocked waiting-for-input callback
            setStatus("INPUT_BLOCKED");
            setInputCallback(() => sendInputByte);
          } else {
            // if there is no input and input blocking is disabled, just send a null-byte
            sendInputByte(0);
          }
          return;
        }

        removeCallback();
        setBusy(false);
        if (e.data.success) {
          setOutput({ type: "OUTPUT", content: e.data.message });
          setInput((prev) => ({ ...prev, amountRead: null }));
          setStatus("IDLE");
          resolve(e.data.message);
        } else {
          setOutput({ type: "ERROR", content: e.data.message });
          setInput((prev) => ({ ...prev, amountRead: null }));
          setStatus("IDLE");
          reject(e.data.message);
        }
      };
      compilerWorker.addEventListener("message", callback);
      const removeCallback = () =>
        compilerWorker.removeEventListener("message", callback);

      setStatus("RUNNING");
      setBusy(true);

      compilerWorker.postMessage({
        command: "RUN",
        transaction,
        arguments: { code, enable_2d_grid },
      });
    });
  };

  const [output, setOutput] = makePersisted(
    createSignal<{
      type: "BF" | "ERROR" | "OUTPUT" | "LIVE_OUTPUT";
      content: string;
    }>(),
    { name: "mastermind_output" }
  );

  const [input, setInput] = makePersisted(
    createSignal<{ text: string; amountRead: number | null }>({
      text: "write input here...",
      amountRead: null,
    }),
    { name: "mastermind_input" }
  );
  // to fix a bug for when the program starts and it saved the amount read in the state:
  onMount(() => setInput((prev) => ({ ...prev, amountRead: null })));
  const popNextInputCharacter = (): string | undefined => {
    let c;
    setInput((prev) => {
      if ((prev.amountRead ?? 0) < prev.text.length) {
        c = prev.text.charAt(prev.amountRead ?? 0);
        return { text: prev.text, amountRead: (prev.amountRead ?? 0) + 1 };
      } else if (prev.amountRead === null) {
        return { text: prev.text, amountRead: 0 };
      } else {
        return prev;
      }
    });
    return c;
  };

  // this side effect is used to detect when input changes for when the BVM is waiting for user input
  const [inputCallback, setInputCallback] = createSignal<(b: number) => void>();
  const [enableBlockingInput, setEnableBlockingInput] = makePersisted(
    createSignal(false)
  );
  createEffect(
    on([input, inputCallback, enableBlockingInput], () => {
      const callback = inputCallback();
      const enableBlocking = enableBlockingInput();
      if (!callback) return;

      const c = popNextInputCharacter();
      if (c) {
        // if there is now enough characters in the input, call the callback and remove it so that it only happens once
        callback(c.charCodeAt(0));
        setStatus("RUNNING");
        setInputCallback(undefined);
      } else if (!enableBlocking) {
        // if there is no input and input blocking is disabled, just send a null-byte
        callback(0);
        setStatus("RUNNING");
        setInputCallback(undefined);
      }
    })
  );

  return (
    <AppContext.Provider
      value={{
        fileStates,
        entryFile,
        setEntryFile,
        createFile,
        deleteFile,
        saveFileState,
        setFileLabel,
        output,
        setOutput,
        reorderFiles,
        compile,
        run,
        busy,
        status,
        input,
        setInput,
        restartWorker,
        helpOpen,
        setHelpOpen,
        enableBlockingInput,
        setEnableBlockingInput,
        settingsOpen,
        setSettingsOpen,
      }}
    >
      <div id="window">
        <EditorPanel />
        <Divider />
        <div class="panel">
          <SettingsPanel style={{ flex: 3 }} />
          <Divider />
          <InputPanel style={{ flex: 1 }} />
          <Divider />
          <OutputPanel style={{ flex: 4 }} />
        </div>
        <Divider />
        <div>
          <SideBar/>
        </div>
      </div>
    </AppContext.Provider>
  );
};

export default App;

export function useAppContext() {
  return useContext<AppContextProps | undefined>(AppContext);
}

interface AppContextProps {
  fileStates: FileState[];
  entryFile: Accessor<string | undefined>;
  setEntryFile: Setter<string | undefined>;
  createFile: (file?: File) => Promise<string>;
  deleteFile: (id: string) => void;
  saveFileState: (id: string, state: EditorState) => void;
  setFileLabel: (id: string, label: string) => void;
  setOutput: Setter<
    | {
        type: "BF" | "ERROR" | "OUTPUT" | "LIVE_OUTPUT";
        content: string;
      }
    | undefined
  >;
  output: Accessor<
    | {
        type: "BF" | "ERROR" | "OUTPUT" | "LIVE_OUTPUT";
        content: string;
      }
    | undefined
  >;
  input: Accessor<{ text: string; amountRead: number | null }>;
  setInput: Setter<{ text: string; amountRead: number | null }>;

  enableBlockingInput: Accessor<boolean>;
  setEnableBlockingInput: Setter<boolean>;

  reorderFiles: (from: string, to: string | null) => void;

  compile: (
    entryFileId: string,
    settings: MastermindConfig
  ) => Promise<string>;
  run: (code: string, enable_2d_grid: boolean) => Promise<string>;

  busy: Accessor<boolean>;
  status: Accessor<"COMPILING" | "RUNNING" | "INPUT_BLOCKED" | "IDLE">;

  restartWorker: () => void;

  helpOpen: Accessor<boolean>;
  setHelpOpen: Setter<boolean>;

  settingsOpen: Accessor<boolean>;
  setSettingsOpen: Setter<boolean>;
}

interface FileState {
  id: string;
  label: string;
  editorState: EditorState;
}
