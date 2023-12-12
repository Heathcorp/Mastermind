import { Component, createSignal, For } from "solid-js";

import "./editor.css";

import { EditorView } from "@codemirror/view";
import { EditorState } from "@codemirror/state";
import { tokyoNight } from "@uiw/codemirror-themes-all";

import { AiOutlinePlus } from "solid-icons/ai";

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
import Tab from "../components/Tab";

const EditorPanel: Component = () => {
  const [selectedTab, setSelectedTab] = createSignal<string>("filename2.txt");
  const [tabStates, setTabStates] = createSignal<
    { id: string; label: string }[]
  >([
    { id: "filename1.txt", label: "filename1!" },
    { id: "filename2.txt", label: "filename2." },
    { id: "filename3.txt", label: "filename3?" },
  ]);

  return (
    <div class="panel">
      <div class="tab-bar">
        <For each={tabStates()}>
          {(tab, i) => (
            <Tab
              label={tab.label}
              selected={tab.id === selectedTab()}
              onSelect={() => {
                setSelectedTab(tab.id);
              }}
            />
          )}
        </For>
        <div class="tab-filler">
          <AiOutlinePlus
            class="text-button"
            onClick={() =>
              setTabStates((prev) => [
                ...prev,
                { id: "hello", label: "added.g" },
              ])
            }
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
