import { Component, createSignal, For, createEffect, on, Show } from "solid-js";

import "./editor.css";

import { EditorView } from "@codemirror/view";
import { EditorState } from "@codemirror/state";
import { initialState } from "../misc";

import { AiOutlineDelete, AiOutlineEdit, AiOutlinePlus } from "solid-icons/ai";
import { v4 as uuidv4 } from "uuid";

const EditorPanel: Component = () => {
  const [editingLabel, setEditingLabel] = createSignal<string | null>(null);
  const [editingFile, setEditingFile] = createSignal<string>();
  // TODO: turn this into a resource for reading from local storage I think
  const [fileStates, setFileStates] = createSignal<
    { id: string; label: string; editorState: EditorState }[]
  >([]);

  createEffect(
    on([fileStates, editingFile], () => {
      // default behaviours for when files are deleted
      if (fileStates().length) {
        if (!editingFile() || !fileStates().find((f) => f.id === editingFile()))
          setEditingFile(fileStates()[0].id);
      } else {
        // if for some reason we don't have a document (the site just started), create one
        const newId = uuidv4();
        setFileStates((prev) => [
          ...prev,
          { id: newId, label: "untitled", editorState: initialState },
        ]);
        setEditingFile(newId);
      }
    })
  );

  let editorView: EditorView | undefined;
  let previousFileId: string | undefined;
  createEffect(
    on([editingFile, () => editorView], () => {
      if (!editorView) return;

      if (previousFileId !== editingFile()) {
        // file has changed
        // save old file to filestate
        const oldState = editorView.state;
        setFileStates((prev) => {
          const fileState = prev.find((f) => f.id === previousFileId);
          if (!fileState) return prev;
          fileState.editorState = oldState;
          return prev;
        });

        const newFileState = fileStates().find((f) => f.id === editingFile());
        if (!newFileState) return;
        editorView.setState(newFileState.editorState);
        previousFileId = editingFile();
      }
    })
  );

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
                  <Show when={fileStates().length > 1}>
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
                  </Show>
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
                editorState: initialState,
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
          editorView = new EditorView({
            state: initialState,
            parent: e,
          });
        }}
      />
    </div>
  );
};

export default EditorPanel;
