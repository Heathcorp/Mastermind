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
import { AiOutlineFileZip, AiOutlineUpload } from "solid-icons/ai";
import JSZip from "jszip";
import downloadBlob from "../utils/downloadBlob";
import { createDropzone } from "@soorria/solid-dropzone";

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

  const zipAndSave = async () => {
    const zip = new JSZip();
    app.fileStates.forEach((fileState) => {
      const blob = new Blob([fileState.editorState.doc.toString()], {
        type: "text/plain",
      });
      zip.file(fileState.label, blob);
    });
    await zip.generateAsync({ type: "blob" }).then((x) => {
      downloadBlob(x);
    });
  };

  const onDrop = (acceptedFiles: File[]) => {
    acceptedFiles.forEach(async (file: File) => {
      const newId = await app.createFile(file);
      switchToFile(newId);
      app.setFileBrowserOpen(false);
    });
  };
  const dropzone = createDropzone({ onDrop });

  return (
    <Show when={app.fileBrowserOpen()}>
      <Portal>
        <div
          class="readme-modal-container"
          onClick={() => app.setFileBrowserOpen(false)}
        >
          <div class="settings-modal">
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
              <div
                classList={{ ["file-tile"]: true, ["file-tile-utility"]: true }}
                onClick={async () => {
                  const newId = await app.createFile();
                  setEditingFile(newId);
                  switchToFile(newId);
                }}
              >
                +
              </div>
              <div
                classList={{ ["file-tile"]: true, ["file-tile-utility"]: true }}
                onClick={() => {
                  zipAndSave();
                }}
              >
                <AiOutlineFileZip size={24} />
              </div>
            </div>
            <div
              onClick={(e) => {
                e.stopPropagation();
              }}
            >
              <div class="file-upload-drop-zone" {...dropzone.getRootProps()}>
                <span style={{ display: "flex", gap: "15px" }}>
                  <AiOutlineUpload size={24} />
                  Click or drop files here to upload
                </span>
              </div>
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
