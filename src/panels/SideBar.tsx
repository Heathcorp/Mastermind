import { Component, JSX } from "solid-js";
import { useAppContext } from "../App.tsx";
import {
  AiFillGithub,
  AiFillQuestionCircle,
  AiFillSetting,
} from "solid-icons/ai";
import { FaSolidBookOpen } from "solid-icons/fa";
import HelpModal from "../components/Help.tsx";
import "./settings.css";
import SettingsModal from "../components/Settings.tsx";

const SideBar: Component<{ style?: JSX.CSSProperties }> = (props) => {
  const app = useAppContext()!;
  console.log(
    import.meta.env.VITE_GIT_COMMIT_BRANCH,
    ":",
    import.meta.env.VITE_GIT_COMMIT_HASH
  );
  return (
    <div
      class="sidebar"
      style={{
        "flex-direction": "column",
        ...props.style,
        border: "10px",
        width: "45px",
      }}
    >
      <a
        class="socials-icon text-button"
        target="_blank"
        href="https://github.com/Heathcorp/Mastermind"
      >
        <AiFillGithub title="Mastermind Git repository" />
      </a>
      <a class="socials-icon text-button" onClick={() => app.setHelpOpen(true)}>
        <AiFillQuestionCircle title="help" />
        <HelpModal />
      </a>
      <a
        class="socials-icon text-button"
        onClick={() => app.setSettingsOpen(true)}
      >
        <AiFillSetting title="Settings" />
        <SettingsModal />
      </a>
      <a
        class="socials-icon text-button"
        target="_blank"
        href={`https://github.com/Heathcorp/Mastermind/blob/${
          import.meta.env.VITE_GIT_COMMIT_HASH
        }/reference.md`}
      >
        <FaSolidBookOpen title="Documentation" />
      </a>
      {/* // TODO: get bootstrap or tailwind */}
      <div style={{ flex: 1 }} />
      <div
        class="badge"
        style={{
          "justify-self": "end",
          "background-color":
            import.meta.env.VITE_GIT_COMMIT_BRANCH == "main"
              ? "#a9e981"
              : import.meta.env.VITE_GIT_COMMIT_BRANCH == "dev"
              ? "#f9f871"
              : "#b35967",
        }}
      >
        {import.meta.env.VITE_GIT_COMMIT_HASH}
      </div>
    </div>
  );
};

export default SideBar;
