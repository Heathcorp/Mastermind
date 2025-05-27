import {
  Accessor,
  createEffect,
  createSignal,
  For,
  on,
  Setter,
  Show,
} from "solid-js";
import { useAppContext } from "../App";
import { Portal } from "solid-js/web";
// import JSZip from "jszip";
// import downloadBlob from "../utils/downloadBlob";

type FileBrowserProps = {
  editingFile: Accessor<string | undefined>;
  setEditingFile: Setter<string | undefined>;
};

export default function FileBrowserModal({
  editingFile,
  setEditingFile,
}: FileBrowserProps) {
  const app = useAppContext();

  const [current, setCurrent] = createSignal<string>();

  if (!app) {
    return <></>;
  }

  createEffect(
    on([editingFile], () => {
      if (!editingFile()) return;
      setCurrent(editingFile());
    })
  );

  const switchToFile = (fileId: string) => {
    const firstId = app.fileStates[0]?.id;
    if (!firstId) return;

    // Grabbing the ID of the first file in the file state list
    // and moving the selected file to the first slot, then opening it
    app.reorderFiles(fileId, firstId);
    setEditingFile(fileId);
  };

  return (
    <Show when={app.fileBrowserOpen()}>
      <Portal>
        <div
          class="readme-modal-container"
          onClick={() => app.setFileBrowserOpen(false)}
        >
          <div class="file-browser-modal">
            <div class="file-browser-container">
              <For each={app.fileStates}>
                {(state) => (
                    <FileTile
                        file={state}
                        switchToFile={switchToFile}
                        current={(state.id as string) === current()}
                    />
                )}
              </For>
            </div>
          </div>
        </div>
      </Portal>
    </Show>
  );
}

function FileTile({
  file,
  switchToFile,
  current,
}: {
  file: any;
  switchToFile: (fileId: string) => void;
  current: boolean;
}) {
  const test = () => {
    switchToFile(file.id as string);
  };

  return (
    <div
      classList={{ ["file-tile"]: true, ["current-file"]: current }}
      onClick={() => test()}
    >
      {file.label}
    </div>
  );
}
