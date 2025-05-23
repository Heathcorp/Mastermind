import {Portal} from "solid-js/web";
import {SolidMarkdown} from "solid-markdown";
import remarkGfm from "remark-gfm";
import {IoClose} from "solid-icons/io";
import {Component, createEffect, createSignal, JSX, Show} from "solid-js";
import { useAppContext } from "../App";
import temp from "../../docs/temp.md?raw";
import {FaSolidArrowLeftLong, FaSolidArrowRightLong} from "solid-icons/fa";
const DocsModal: Component<{ style?: JSX.CSSProperties }> = () => {
    const app = useAppContext()!;
    const docs = {'Temp': temp};
    const titles = Object.keys(docs);
    const [selected, setSelected] = createSignal(titles[0]);
    const [docsContent, setDocsContent] = createSignal(docs[selected() as keyof typeof docs] ?? "")
    createEffect(() => {
        setDocsContent(docs[selected() as keyof typeof docs] ?? "")
    });

    function nextDoc() {
        setSelected(titles[(titles.indexOf(selected() ?? "") + 1) % titles.length])
    }
    function prevDoc() {
        setSelected(titles[(titles.indexOf(selected() ?? "") - 1 + titles.length) % titles.length])
    }

    return (<Show when={app.docsOpen()}>
        {/* The weirdest solid js feature, puts the component into the top level html body */}
        <Portal>
            <div
                class="readme-modal-container"
                onClick={() => app.setDocsOpen(false)}
            >
                <div class="docs-modal" onClick={(e) => e.stopPropagation()}>
                    <FaSolidArrowLeftLong style={{"cursor": "pointer"}} onClick={prevDoc}/>
                    <select class="header-select" onInput={(e) => setSelected(e.currentTarget.value)}>
                        {titles.map((title) => (
                            <option value={title} selected={title === selected()}>
                                {title}
                            </option>
                        ))}
                    </select>
                    <FaSolidArrowRightLong style={{"cursor": "pointer"}} onClick={nextDoc}/>
                    <div class="markdown-container">
                        <SolidMarkdown remarkPlugins={[remarkGfm]}>
                            {docsContent()}
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
                        onClick={() => app.setDocsOpen(false)}
                    />
                </div>
            </div>
        </Portal>
    </Show>)}

export default DocsModal;