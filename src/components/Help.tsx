import {Portal} from "solid-js/web";
import {SolidMarkdown} from "solid-markdown";
import remarkGfm from "remark-gfm";
import {IoClose} from "solid-icons/io";
import {Component, JSX, Show} from "solid-js";
import { useAppContext } from "../App";
import readmeContent from "../../README.md?raw";

const HelpModal: Component<{ style?: JSX.CSSProperties }> = () => {
    const app = useAppContext()!;
    return (<Show when={app.helpOpen()}>
        {/* The weirdest solid js feature, puts the component into the top level html body */}
        <Portal>
            <div
                class="readme-modal-container"
                onClick={() => app.setHelpOpen(false)}
            >
                <div class="readme-modal" onClick={(e) => e.stopPropagation()}>
                    <div class="markdown-container">
                        <SolidMarkdown remarkPlugins={[remarkGfm]}>
                            {readmeContent}
                        </SolidMarkdown>
                    </div>
                    <IoClose
                        title="close help"
                        class="text-button"
                        style={{
                            "font-size": "1.5rem",
                            position: "absolute",
                            right: "1rem",
                            top: "1rem",
                        }}
                        onClick={() => app.setHelpOpen(false)}
                    />
                </div>
            </div>
        </Portal>
    </Show>)}

export default HelpModal;