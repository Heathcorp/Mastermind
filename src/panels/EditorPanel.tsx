import { Component, createSignal, For, createEffect, on } from "solid-js";

import "./editor.css";

import { EditorView } from "@codemirror/view";
import { AiOutlinePlus } from "solid-icons/ai";
import {
  DragDropProvider,
  DragDropSensors,
  createDroppable,
  useDragDropContext,
} from "@thisbeyond/solid-dnd";

import { useAppContext } from "../App";
import Tab from "../components/Tab";

const EditorPanel: Component = () => {
  const app = useAppContext()!;

  const [editingFile, setEditingFile] = createSignal<string>();

  createEffect(
    on([app.fileStates, editingFile], () => {
      // default behaviours for when files are deleted
      if (!app.fileStates().length) return;
      if (
        !editingFile() ||
        !app.fileStates().find((f) => f.id === editingFile())
      ) {
        setEditingFile(app.fileStates()[0]?.id);
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
        {/* tab rearranging logic for filestates in global file array */}
        <DragDropProvider
          onDragEnd={({ draggable, droppable }) =>
            droppable &&
            app.reorderFiles(
              draggable.id as string,
              droppable.id === TAB_END_ID ? null : (droppable.id as string)
            )
          }
        >
          <DragDropSensors>
            <For each={app.fileStates()}>
              {(fileState) => (
                <Tab
                  fileId={fileState.id}
                  fileLabel={fileState.label}
                  fileActive={fileState.id === editingFile()}
                  onSelect={() => setEditingFile(fileState.id)}
                />
              )}
            </For>
            <TabFiller
              onAdd={() => {
                const newId = app.createFile();
                setEditingFile(newId);
                // setEditingLabel(newFile.id);
              }}
            />
          </DragDropSensors>
        </DragDropProvider>
      </div>
      <div class="code-panel" ref={editorRef} />
    </div>
  );
};

export default EditorPanel;

const TAB_END_ID = "end";
const TabFiller: Component<{ onAdd: () => void }> = (props) => {
  // for dragging a file to the end of the list
  // had to make this its own component because of dragDrop context issues
  const droppableRef = createDroppable(TAB_END_ID);
  const [isUnderDrag, setIsUnderDrag] = createSignal(false);
  const [, { onDragOver, onDragEnd }] = useDragDropContext()!;

  onDragOver(({ droppable }) => setIsUnderDrag(droppable?.id === TAB_END_ID));
  onDragEnd(() => setIsUnderDrag(false));

  return (
    <div
      ref={droppableRef}
      classList={{ "tab-filler": true, "tab-insert-marker": isUnderDrag() }}
    >
      <AiOutlinePlus class="text-button" onClick={props.onAdd} />
    </div>
  );
};
