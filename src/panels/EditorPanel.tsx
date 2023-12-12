import { Component, createSignal, For, Match, Show } from "solid-js";

import "./editor.css";

import { EditorView } from "@codemirror/view";
import { EditorState } from "@codemirror/state";
import { tokyoNight } from "@uiw/codemirror-themes-all";

import { AiOutlineDelete, AiOutlineEdit, AiOutlinePlus } from "solid-icons/ai";
import { v4 as uuidv4 } from "uuid";

import {
  lineNumbers,
  highlightActiveLineGutter,
  highlightSpecialChars,
  drawSelection,
  dropCursor,
  rectangularSelection,
  crosshairCursor,
  highlightActiveLine,
  keymap,
} from "@codemirror/view";
export { EditorView } from "@codemirror/view";
import {
  foldGutter,
  indentOnInput,
  syntaxHighlighting,
  defaultHighlightStyle,
  bracketMatching,
  foldKeymap,
} from "@codemirror/language";
import { history, defaultKeymap, historyKeymap } from "@codemirror/commands";
import { highlightSelectionMatches, searchKeymap } from "@codemirror/search";
import {
  closeBrackets,
  autocompletion,
  closeBracketsKeymap,
  completionKeymap,
} from "@codemirror/autocomplete";
import { lintKeymap } from "@codemirror/lint";
import { cpp } from "@codemirror/lang-cpp";

const EditorPanel: Component = () => {
  const [editingLabel, setEditingLabel] = createSignal<string | null>(null);
  const [editingFile, setEditingFile] = createSignal<string>("filename2.txt");
  const [fileStates, setFileStates] = createSignal<
    { id: string; label: string; editorState: EditorState }[]
  >([]);

  return (
    <div class="panel">
      <div class="tab-bar">
        <For each={fileStates()}>
          {(tab, i) => (
            <div
              classList={{
                ["tab"]: true,
                ["tab-selected"]: tab.id === editingFile(),
              }}
              onClick={() => setEditingFile(tab.id)}
            >
              {tab.id === editingLabel() ? (
                <form
                  onSubmit={(e) => {
                    console.log("hello1");
                    e.preventDefault();
                    setFileStates((prev) => {
                      console.log("hello2");
                      const file = prev.find((f) => f.id === tab.id);
                      if (!file) return prev;
                      // TODO: refactor this, maybe find a form library? At least make this a reusable component
                      file.label = (
                        e.target.children as HTMLCollection & {
                          label: HTMLInputElement;
                        }
                      ).label.value;
                      return prev;
                    });
                    setEditingLabel(null);
                  }}
                >
                  <input name="label" value={tab.label} />
                </form>
              ) : (
                <>
                  <AiOutlineEdit
                    class="text-button"
                    onClick={() => setEditingLabel(tab.id)}
                  />
                  {tab.label}
                  <AiOutlineDelete
                    class="text-button"
                    style={{ "margin-left": "0.5rem" }}
                    onClick={() =>
                      window.confirm(
                        "Are you sure you want to delete this file? This cannot be undone."
                      ) &&
                      setFileStates((prev) =>
                        prev.filter((f) => f.id !== tab.id)
                      )
                    }
                  />
                </>
              )}
            </div>
          )}
        </For>
        <div class="tab-filler">
          <AiOutlinePlus
            class="text-button"
            onClick={() => {
              const newFile = {
                id: uuidv4(),
                label: "untitled",
                editorState: EditorState.create({}),
              };
              setFileStates((prev) => [...prev, newFile]);
              // setEditingLabel(newFile.id);
            }}
          />
        </div>
      </div>
      <div
        class="code-panel"
        ref={(e) => {
          // TODO: figure this stuff out and make it nicer

          let startState = EditorState.create({
            doc: "+++---[][][]",

            extensions: [
              cpp(),
              tokyoNight,
              lineNumbers(),
              highlightActiveLineGutter(),
              highlightSpecialChars(),
              history(),
              foldGutter(),
              drawSelection(),
              dropCursor(),
              EditorState.allowMultipleSelections.of(true),
              indentOnInput(),
              syntaxHighlighting(defaultHighlightStyle, { fallback: true }),
              bracketMatching(),
              closeBrackets(),
              autocompletion(),
              rectangularSelection(),
              crosshairCursor(),
              highlightActiveLine(),
              highlightSelectionMatches(),
              keymap.of([
                ...closeBracketsKeymap,
                ...defaultKeymap,
                ...searchKeymap,
                ...historyKeymap,
                ...foldKeymap,
                ...completionKeymap,
                ...lintKeymap,
              ]),
            ],
          });

          let view = new EditorView({
            state: startState,
            parent: e,
          });
        }}
      />
    </div>
  );
};

export default EditorPanel;
