import { Component, Show, createSignal } from "solid-js";

import {
  AiOutlineDelete,
  AiOutlineDownload,
  AiOutlineEdit,
} from "solid-icons/ai";
import {
  createDraggable,
  createDroppable,
  useDragDropContext,
} from "@thisbeyond/solid-dnd";

import { useAppContext } from "../App";
import "../panels/editor.css";
import downloadBlob from "../utils/downloadBlob";

const Tab: Component<{
  fileId: string;
  fileLabel: string;
  fileActive: boolean;
  onSelect: () => void;
}> = (props) => {
  const app = useAppContext()!;

  const [editingLabel, setEditingLabel] = createSignal(false);

  const draggableRef = createDraggable(props.fileId);
  const droppableRef = createDroppable(props.fileId);
  const [isUnderDrag, setIsUnderDrag] = createSignal(false);
  const [, { onDragEnd, onDragOver }] = useDragDropContext()!;
  onDragOver(({ droppable }) => setIsUnderDrag(droppable?.id === props.fileId));
  onDragEnd(() => setIsUnderDrag(false));

  let inputRef: HTMLInputElement | undefined;

  const saveLabel = () => {
    setEditingLabel((editingLabel) => {
      if (!editingLabel || !inputRef?.value) return editingLabel;
      app.setFileLabel(props.fileId, inputRef?.value);
      return false;
    });
  };

  const fileDownload = () => {
    const fileState = app.fileStates.find((f) => f.id == props.fileId);
    const fileData = fileState?.editorState.doc.toString();
    if (!fileData) return new Blob([]);
    const blobFile = new Blob([fileData], { type: "text/plain" });
    downloadBlob(blobFile, props.fileLabel);
  };

  return (
    <div
      ref={(e) => {
        draggableRef(e);
        droppableRef(e);
      }}
      classList={{
        ["tab"]: true,
        ["tab-selected"]: props.fileActive,
        ["tab-insert-marker"]: isUnderDrag(),
        ["file-label-text"]: true,
      }}
      onPointerDown={props.onSelect}
    >
      <Show when={!editingLabel()}>
        <AiOutlineEdit
          class="text-button"
          onClick={() => {
            setEditingLabel(true);
            inputRef?.focus();
          }}
        />
      </Show>
      <Show when={editingLabel()} fallback={props.fileLabel}>
        <form
          onSubmit={(e) => {
            e.preventDefault();
            saveLabel();
          }}
        >
          <input
            class="tab-label-editor-input file-label-text"
            ref={inputRef}
            name="filename"
            readOnly={!editingLabel()}
            onBlur={saveLabel}
            value={props.fileLabel}
            // I learnt something today, HTML input elements are really stupid and don't change size from their value
            size={props.fileLabel.length}
          />
        </form>
      </Show>
      <Show when={app.fileStates.length > 1 && props.fileActive}>
        <AiOutlineDownload
          class="text-button"
          style={{ "margin-left": "0.5rem" }}
          onClick={() => fileDownload()}
        />
      </Show>
      <Show when={app.fileStates.length > 1}>
        <AiOutlineDelete
          class="text-button"
          style={{ "margin-left": "0.5rem" }}
          // Not sure if delete logic should be handled by the parent component
          onClick={() =>
            window.confirm(
              "Are you sure you want to delete this file? This cannot be undone."
            ) && app.deleteFile(props.fileId)
          }
        />
      </Show>
    </div>
  );
};

export default Tab;
