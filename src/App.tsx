import {
  Component,
  createSignal,
  createContext,
  useContext,
  Accessor,
} from "solid-js";

import { EditorState } from "@codemirror/state";
import { v4 as uuidv4 } from "uuid";

import "./App.css";
import Divider from "./components/Divider";
import EditorPanel from "./panels/EditorPanel";
import InputPanel from "./panels/InputPanel";

import init, { InitOutput as CompilerObj } from "../compiler/pkg";
import OutputPanel from "./panels/OutputPanel";
import SettingsPanel from "./panels/SettingsPanel";
import { initialState } from "./misc";

const CompilerContext = createContext<CompilerObj>();
const AppContext = createContext<AppContextProps>();

const App: Component = () => {
  // global wasm compiler object
  const [compiler, setCompiler] = createSignal<CompilerObj>();
  init().then(setCompiler);

  // global signals and functions and things
  // TODO: turn this into a resource for reading from local storage I think
  const [fileStates, setFileStates] = createSignal<FileState[]>([]);
  const createFile = () => {
    const newId = uuidv4();
    setFileStates((prev) => [
      ...prev,
      { id: newId, label: "untitled", editorState: initialState },
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

  return (
    <AppContext.Provider
      value={{
        fileStates,
        createFile,
        deleteFile,
        saveFileState,
        setFileLabel,
      }}
    >
      <CompilerContext.Provider value={compiler()}>
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
      </CompilerContext.Provider>
    </AppContext.Provider>
  );
};

export default App;

export function useCompiler() {
  return useContext<CompilerObj | undefined>(CompilerContext);
}

export function useAppContext() {
  return useContext<AppContextProps | undefined>(AppContext);
}

interface AppContextProps {
  fileStates: Accessor<FileState[]>;
  createFile: () => string;
  deleteFile: (id: string) => void;
  saveFileState: (id: string, state: EditorState) => void;
  setFileLabel: (id: string, label: string) => void;
}

interface FileState {
  id: string;
  label: string;
  editorState: EditorState;
}
