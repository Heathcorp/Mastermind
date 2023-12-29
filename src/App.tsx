import {
  Component,
  createSignal,
  createContext,
  useContext,
  Accessor,
  Setter,
  createEffect,
  on,
} from "solid-js";

// special vite syntax for web workers to be included in the bundle
import CompilerWorker from "./worker.ts?worker";

import { EditorState } from "@codemirror/state";
import { EditorView } from "@codemirror/view";
import { v4 as uuidv4 } from "uuid";

import divisorsExample from "./assets/divisors_example.mmi?raw";
import printExample from "./assets/print.mmi?raw";

import "./App.css";
import Divider from "./components/Divider";
import EditorPanel from "./panels/EditorPanel";
// import InputPanel from "./panels/InputPanel";

import OutputPanel from "./panels/OutputPanel";
import SettingsPanel, { MastermindConfig } from "./panels/SettingsPanel";
import { defaultExtensions } from "./misc";
import { makePersisted } from "@solid-primitives/storage";

const AppContext = createContext<AppContextProps>();

// update this when you want the user to see new syntax
const MIGRATION_VERSION = 1;

const App: Component = () => {
  const [version, setVersion] = makePersisted(createSignal<number>());
  createEffect(
    on([version], () => {
      const v = version();
      if (!v || v !== MIGRATION_VERSION) {
        window.alert(
          "Version has changed since last load, new example programs will be loaded.\nNote: your old programs may now have incorrect syntax"
        );
        loadExampleFiles();
        setVersion(MIGRATION_VERSION);
      }
    })
  );
  // global signals and functions and things
  // to the program this is just a solidjs signal, all of this extra stuff is just for persistence
  const [entryFile, setEntryFile] = makePersisted(createSignal<string>());
  const [fileStates, setFileStates] = makePersisted(
    createSignal<FileState[]>(
      (() => {
        return [];
      })()
    ),
    {
      name: "mastermind_editor_files",
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
    on([fileStates], () => {
      if (!fileStates().length) {
        // there are no files, initialise to the example files (divisors of 1 to 100)
        loadExampleFiles();
      }
    })
  );

  const loadExampleFiles = () => {
    const newId = uuidv4();
    setFileStates((prev) => [
      ...[
        {
          id: newId,
          label: "divisors_example.mmi",
          rawText: divisorsExample,
        },
        { id: uuidv4(), label: "print.mmi", rawText: printExample },
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
    setEntryFile(newId);
  };

  const createFile = () => {
    const newId = uuidv4();
    setFileStates((prev) => [
      ...prev,
      {
        id: newId,
        label: `untitled_${prev.length}`,
        editorState: EditorState.create({
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
    setFileStates((prev) => {
      const fileState = prev.find((f) => f.id === id);
      if (!fileState) return prev;
      fileState.editorState = state;
      return [...prev];
    });
  };
  const setFileLabel = (id: string, label: string) => {
    setFileStates((prev) => {
      const fileStateIndex = prev.findIndex((f) => f.id === id);
      if (fileStateIndex === -1) return prev;
      const fileState = prev.splice(fileStateIndex, 1)[0]!;
      return [...prev, { ...fileState, label }];
    });
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
    "COMPILING" | "RUNNING" | "INPUT_BLOCKED"
  >();

  const compilerWorker = new CompilerWorker();

  const compile = (entryFileId: string, optimisations: MastermindConfig) => {
    return new Promise<string>((resolve, reject) => {
      let entryFileName: string | undefined;
      const fileMap = Object.fromEntries(
        fileStates().map((file) => {
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
        if (transaction === e.data.transaction) {
          removeCallback();
          setBusy(false);
          if (e.data.success) {
            setOutput({ type: "BF", content: e.data.message });
            setStatus();
            resolve(e.data.message);
          } else {
            setOutput({ type: "ERROR", content: e.data.message });
            setStatus();
            reject(e.data.message);
          }
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
          optimisations,
        },
      });

      // TODO: maybe make a timeout to auto-reject
      // probably more important for the run code
    });
  };

  const run = (code: string) => {
    return new Promise<string>((resolve, reject) => {
      const transaction = uuidv4();
      const callback = (e: {
        data: { transaction: string; success: boolean; message: string };
      }) => {
        if (transaction === e.data.transaction) {
          removeCallback();
          setBusy(false);
          if (e.data.success) {
            setOutput({ type: "OUTPUT", content: e.data.message });
            setStatus();
            resolve(e.data.message);
          } else {
            setOutput({ type: "ERROR", content: e.data.message });
            setStatus();
            reject(e.data.message);
          }
        }
      };
      compilerWorker.addEventListener("message", callback);
      const removeCallback = () =>
        compilerWorker.removeEventListener("message", callback);

      setStatus("RUNNING");
      setBusy(true);
      // paranoid me is posting the message after setting up the listener
      compilerWorker.postMessage({
        command: "RUN",
        transaction,
        arguments: { code },
      });
    });
  };

  const [output, setOutput] = createSignal<{
    type: "BF" | "ERROR" | "OUTPUT";
    content: string;
  }>();

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
      }}
    >
      <div id="window">
        <EditorPanel />
        <Divider />
        <div class="panel">
          <SettingsPanel />
          <Divider />
          <OutputPanel />
          {/* <Divider />
          <InputPanel /> */}
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
  fileStates: Accessor<FileState[]>;
  entryFile: Accessor<string | undefined>;
  setEntryFile: Setter<string | undefined>;
  createFile: () => string;
  deleteFile: (id: string) => void;
  saveFileState: (id: string, state: EditorState) => void;
  setFileLabel: (id: string, label: string) => void;
  setOutput: Setter<
    | {
        type: "BF" | "ERROR" | "OUTPUT";
        content: string;
      }
    | undefined
  >;
  output: Accessor<
    | {
        type: "BF" | "ERROR" | "OUTPUT";
        content: string;
      }
    | undefined
  >;
  reorderFiles: (from: string, to: string | null) => void;

  compile: (
    entryFileId: string,
    optimisations: MastermindConfig
  ) => Promise<string>;
  run: (code: string) => Promise<string>;

  busy: Accessor<boolean>;
  status: Accessor<"COMPILING" | "RUNNING" | "INPUT_BLOCKED" | undefined>;
}

interface FileState {
  id: string;
  label: string;
  editorState: EditorState;
}
