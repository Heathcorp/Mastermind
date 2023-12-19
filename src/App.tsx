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

import { EditorState } from "@codemirror/state";
import { EditorView } from "@codemirror/view";
import { v4 as uuidv4 } from "uuid";

import divisorsExample from "./assets/divisors_example.mmi?raw";
import printExample from "./assets/print.mmi?raw";

import "./App.css";
import Divider from "./components/Divider";
import EditorPanel from "./panels/EditorPanel";
// import InputPanel from "./panels/InputPanel";

import initWasm, { wasm_compile, wasm_run_bf } from "../compiler/pkg";
import OutputPanel from "./panels/OutputPanel";
import SettingsPanel, { MastermindConfig } from "./panels/SettingsPanel";
import { defaultExtensions } from "./misc";
import { makePersisted } from "@solid-primitives/storage";

const AppContext = createContext<AppContextProps>();

const App: Component = () => {
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
        const newId = uuidv4();
        setFileStates(
          [
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
          }))
        );
        setEntryFile(newId);
      }
    })
  );

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
      const fileState = prev.splice(fileStateIndex, 1)[0];
      return [...prev, { ...fileState, label }];
    });
  };

  const [compiledCode, setCompiledCode] = createSignal<string>();
  const compile = (entryFileId: string, optimisations: MastermindConfig) => {
    let entryFileName: string = fileStates()[0].label;
    const fileMap = Object.fromEntries(
      fileStates().map((file) => {
        if (file.id === entryFileId) entryFileName = file.label;
        return [file.label, file.editorState.doc.toString()];
      })
    );
    const result = wasm_compile(fileMap, entryFileName, optimisations);
    setCompiledCode(result);
    return result;
  };
  // TODO: add input stuff
  const runCode = (code: string) => {
    const result = wasm_run_bf(code);
    return result;
  };
  initWasm();

  const [output, setOutput] = createSignal<string>();

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
        compile,
        setOutput,
        runCode,
        compiledCode,
      }}
    >
      <div id="window">
        <EditorPanel />
        <Divider />
        <div class="panel">
          <SettingsPanel />
          <Divider />
          <OutputPanel outputText={output() ?? ""} />
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
  compile: (entryFileId: string, optimisations: MastermindConfig) => string;
  setOutput: (output?: string) => void;
  runCode: (code: string) => string;
  compiledCode: Accessor<string | undefined>;
}

interface FileState {
  id: string;
  label: string;
  editorState: EditorState;
}
