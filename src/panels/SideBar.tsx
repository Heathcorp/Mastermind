import {Component, JSX} from "solid-js";
import {useAppContext} from "../App.tsx";
import {AiFillGithub, AiFillQuestionCircle} from "solid-icons/ai";
import HelpModal from "../components/Help.tsx";
import "./settings.css";

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
        </div>
            );
            }

            export default SideBar;