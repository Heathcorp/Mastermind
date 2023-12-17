import {
  Component,
  createSignal,
  createContext,
  useContext,
  Accessor,
} from "solid-js";

import { EditorState } from "@codemirror/state";
import { EditorView } from "@codemirror/view";
import { v4 as uuidv4 } from "uuid";

import "./App.css";
import Divider from "./components/Divider";
import EditorPanel from "./panels/EditorPanel";
import InputPanel from "./panels/InputPanel";

import initWasm, { wasm_compile } from "../compiler/pkg";
import OutputPanel from "./panels/OutputPanel";
import SettingsPanel from "./panels/SettingsPanel";
import { defaultExtensions } from "./misc";
import { makePersisted } from "@solid-primitives/storage";

const AppContext = createContext<AppContextProps>();

const App: Component = () => {
  // global signals and functions and things
  // to the program this is just a solidjs signal, all of this extra stuff is just for persistence
  const [fileStates, setFileStates] = makePersisted(
    createSignal<FileState[]>([]),
    {
      name: "mastermind_editor_files",
      serialize: (fileStates: FileState[]) =>
        JSON.stringify(
          fileStates.map((fileState) => ({
            id: fileState.id,
            label: fileState.label,
            raw_text: fileState.editorState.doc.toString(),
          }))
        ),
      deserialize: (data: string): FileState[] =>
        (
          JSON.parse(data) as unknown as {
            id: string;
            label: string;
            raw_text: string;
          }[]
        ).map((storedState) => ({
          id: storedState.id,
          label: storedState.label,
          editorState: EditorState.create({
            doc: storedState.raw_text,
            extensions: [
              ...defaultExtensions,
              EditorView.updateListener.of((e) => {
                // this basically saves the editor every time it updates, this may be inefficient
                saveFileState(storedState.id, e.state);
              }),
            ],
          }),
        })),
    }
  );

  const createFile = () => {
    const newId = uuidv4();
    setFileStates((prev) => [
      ...prev,
      {
        id: newId,
        label: "untitled",
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
      const fileStateIndex = prev.findIndex((f) => f.id === id);
      if (fileStateIndex === -1) return prev;
      const fileState = prev.splice(fileStateIndex, 1)[0];
      return [...prev, { ...fileState, editorState: state }];
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

  const compile = (entryFileId: string) => {
    const fileMap = Object.fromEntries(
      fileStates().map((file) => {
        return [file.id, file.editorState.doc.toString()];
      })
    );
    const result = wasm_compile(fileMap, entryFileId);
    return result;
  };
  initWasm();

  return (
    <AppContext.Provider
      value={{
        fileStates,
        createFile,
        deleteFile,
        saveFileState,
        setFileLabel,
        compile,
      }}
    >
      <div id="window">
        <EditorPanel />
        <Divider />
        <div class="panel">
          <SettingsPanel />
          <Divider />
          <OutputPanel outputText={"output text lines"} />
          <Divider />
          <InputPanel />
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
  createFile: () => string;
  deleteFile: (id: string) => void;
  saveFileState: (id: string, state: EditorState) => void;
  setFileLabel: (id: string, label: string) => void;
  compile: (entryFileId: string) => string;
}

interface FileState {
  id: string;
  label: string;
  editorState: EditorState;
}
