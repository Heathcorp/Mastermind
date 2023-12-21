import { Component, Show, createSignal } from "solid-js";

import { AiOutlineDelete, AiOutlineEdit } from "solid-icons/ai";
import {
  createDraggable,
  createDroppable,
  useDragDropContext,
} from "@thisbeyond/solid-dnd";

import { useAppContext } from "../App";
import "../panels/editor.css";

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
      }}
      onPointerDown={props.onSelect}
    >
      {editingLabel() ? (
        // TODO: refactor this file renaming behaviour
        <form
          onSubmit={(e) => {
            e.preventDefault();
            // TODO: refactor this, maybe find a form library? At least make this a reusable component
            app.setFileLabel(
              props.fileId,
              (
                e.target.children as HTMLCollection & {
                  label: HTMLInputElement;
                }
              ).label.value
            );

            setEditingLabel(false);
          }}
        >
          <input name="label" value={props.fileLabel} />
        </form>
      ) : (
        <>
          <AiOutlineEdit
            class="text-button"
            onClick={() => setEditingLabel(true)}
          />
          {props.fileLabel}
          <Show when={app.fileStates().length > 1}>
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
        </>
      )}
    </div>
  );
};

export default Tab;
