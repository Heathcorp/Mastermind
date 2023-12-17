import { Component, createSignal, For, createEffect, on, Show } from "solid-js";

import "./editor.css";

import { EditorView } from "@codemirror/view";
import { defaultExtensions } from "../misc";

import { AiOutlineDelete, AiOutlineEdit, AiOutlinePlus } from "solid-icons/ai";
import { useAppContext } from "../App";

const EditorPanel: Component = () => {
  const app = useAppContext()!;

  const [editingLabel, setEditingLabel] = createSignal<string | null>(null);
  const [editingFile, setEditingFile] = createSignal<string>();

  createEffect(
    on([app.fileStates, editingFile], () => {
      // default behaviours for when files are deleted
      if (app.fileStates().length) {
        if (
          !editingFile() ||
          !app.fileStates().find((f) => f.id === editingFile())
        )
          setEditingFile(app.fileStates()[0].id);
      } else {
        // if for some reason we don't have a document (the site just started), create one
        const newId = app.createFile();
        setEditingFile(newId);
      }
    })
  );

  let editorRef: HTMLDivElement | undefined;

  let editorView: EditorView | undefined;
  let previousFileId: string | undefined;
  createEffect(
    on([editingFile, () => editorView, () => editorRef], () => {
      if (!editorRef) return;
      if (!editingFile()) return;

      if (!editorView) {
        // the element exists but view hasn't been constructed yet
        // construct it
        const fileState = app.fileStates().find((f) => f.id === editingFile());
        editorView = new EditorView({
          state: fileState?.editorState,
          parent: editorRef,
        });
      } else if (previousFileId && previousFileId !== editingFile()) {
        // file has changed
        // save old file to filestate
        const oldState = editorView.state;
        app.saveFileState(previousFileId, oldState);

        const newFileState = app
          .fileStates()
          .find((f) => f.id === editingFile());
        if (!newFileState) return;
        editorView.setState(newFileState.editorState);
      }
      previousFileId = editingFile();
    })
  );

  return (
    <div class="panel">
      <div class="tab-bar">
        <For each={app.fileStates()}>
          {(tab) => (
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
                    e.preventDefault();
                    // TODO: refactor this, maybe find a form library? At least make this a reusable component
                    app.setFileLabel(
                      tab.id,
                      (
                        e.target.children as HTMLCollection & {
                          label: HTMLInputElement;
                        }
                      ).label.value
                    );

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
                  <Show when={app.fileStates().length > 1}>
                    <AiOutlineDelete
                      class="text-button"
                      style={{ "margin-left": "0.5rem" }}
                      onClick={() =>
                        window.confirm(
                          "Are you sure you want to delete this file? This cannot be undone."
                        ) && app.deleteFile(tab.id)
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
              const newId = app.createFile();
              setEditingFile(newId);
              // setEditingLabel(newFile.id);
            }}
          />
        </div>
      </div>
      <div class="code-panel" ref={editorRef} />
    </div>
  );
};

export default EditorPanel;
