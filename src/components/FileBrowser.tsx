import { Show } from "solid-js";
import { useAppContext } from "../App";
import { Portal } from "solid-js/web";

type FileBrowserProps = {};

export default function FileBrowserModal(props: FileBrowserProps) {
  const app = useAppContext();

  if (!app) {
    return <></>;
  }

  return (
    <Show when={app.fileBrowserOpen()}>
      <Portal>
        <div
          class="readme-modal-container"
          onClick={() => app.setFileBrowserOpen(false)}
        >
          <div class="settings-modal">TESTESTEST</div>
        </div>
      </Portal>
    </Show>
  );
}
