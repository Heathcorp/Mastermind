import {Component, JSX} from "solid-js";
import {useAppContext} from "../App.tsx";
import {AiFillGithub, AiFillQuestionCircle, AiFillSetting} from "solid-icons/ai";
import { FaSolidBookOpen } from 'solid-icons/fa'
import HelpModal from "../components/Help.tsx";
import DocsModal from "../components/Docs.tsx";
import "./settings.css";
import SettingsModal from "../components/Settings.tsx";

const SideBar: Component<{ style?: JSX.CSSProperties }> = (props) => {
    const app = useAppContext()!;

    return (
        <div class="sidebar" style={{"flex-direction": "column", ...props.style, "border": "10px", "width": "45px"}}>
            <a
                class="socials-icon text-button"
                href="https://github.com/Heathcorp/Mastermind"
                target="_blank"
            >
                <AiFillGithub title="Git repository"/>
            </a>
            <a
                class="socials-icon text-button"
                target="_blank"
                onClick={() => app.setHelpOpen(true)}
            >
                < AiFillQuestionCircle title="help"/>
                <HelpModal></HelpModal>
            </a>
            <a
                class="socials-icon text-button"
                target="_blank"
                onClick={() => app.setSettingsOpen(true)}
            >
                < AiFillSetting title="Settings"/>
                <SettingsModal></SettingsModal>
            </a>
            <a
                class="socials-icon text-button"
                target="_blank"
                onClick={() => app.setDocsOpen(true)}
            >
                < FaSolidBookOpen title="docs"/>
                <DocsModal></DocsModal>
            </a>
        </div>
    );
}

export default SideBar;