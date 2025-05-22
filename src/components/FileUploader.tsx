import { Setter, Show } from "solid-js";
import { useAppContext } from "../App";
import { Portal } from "solid-js/web";
import { AiOutlineUpload } from "solid-icons/ai";
import { createDropzone } from "@soorria/solid-dropzone";

type FileUploaderProps = {
  //   editingFile: Accessor<string | undefined>;
  setEditingFile: Setter<string | undefined>;
};

export default function FileUploaderModal({
  setEditingFile,
}: FileUploaderProps) {
  const app = useAppContext();
  if (!app) return;

  const switchToFile = (fileId: string) => {
    const firstId = app.fileStates[0]?.id;
    if (!firstId) return;

    // Grabbing the ID of the first file in the file state list
    // and moving the selected file to the first slot, then opening it
    app.reorderFiles(fileId, firstId);
    setEditingFile(fileId);
  };

  const onDrop = (acceptedFiles: File[]) => {
    acceptedFiles.forEach(async (file: File) => {
      const newId = await app.createFile(file);
      switchToFile(newId);
      app.setFileUploaderOpen(false);
    });
  };

  const dropzone = createDropzone({ onDrop });

  return (
    <Show when={app.fileUploaderOpen()}>
      <Portal>
        <div
          class="readme-modal-container"
          onClick={() => app.setFileUploaderOpen(false)}
        >
          <div class="settings-modal">
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
