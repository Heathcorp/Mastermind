import {Portal} from "solid-js/web";
import {SolidMarkdown} from "solid-markdown";
import remarkGfm from "remark-gfm";
import {IoClose} from "solid-icons/io";
import {Component, createEffect, createSignal, JSX, Show} from "solid-js";
import { useAppContext } from "../App";
import intro from "../../docs/intro.md?raw";
import cells from "../../docs/cells.md?raw";
import io from "../../docs/io.md?raw";
import conditionals from "../../docs/conditionals.md?raw";
import loops from "../../docs/loops.md?raw";
import inlinebrainfuck from "../../docs/inlinebrainfuck.md?raw";
import functions from "../../docs/functions.md?raw";
import structs from "../../docs/structs.md?raw";
import standardlib from "../../docs/standardlib.md?raw";
import twodimensional from "../../docs/twodimensional.md?raw";
import optimisations from "../../docs/optimisations.md?raw";
import {FaSolidArrowLeftLong, FaSolidArrowRightLong} from "solid-icons/fa";
const DocsModal: Component<{ style?: JSX.CSSProperties }> = () => {
    const app = useAppContext()!;
    const docs = {'Introduction': intro,
        'Cells (Variables)': cells,
        'Input/Output': io,
        'Conditionals': conditionals,
        'Loops': loops,
        'Inline Brainfuck': inlinebrainfuck,
        'Functions': functions,
        'Structs': structs,
        'Standard Library': standardlib,
        '2D Mastermind': twodimensional,
        'Optimisations': optimisations,
    };
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