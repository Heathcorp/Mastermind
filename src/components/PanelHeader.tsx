import "./components.css";
import {FiCopy} from "solid-icons/fi";
import {createMemo} from "solid-js";

function PanelHeader({title, getContent} : {title: string, getContent: () => string}) {
    const contentLength = createMemo(() => getContent().length);

    const copyToClipboard = () => {
        const textToCopy = getContent();
        if (!textToCopy) return;

        window.navigator.clipboard
            .writeText(textToCopy)
            .then(() => window.alert("Output copied to clipboard!"));
    };


    return <div class="panel-header">
        <p class="panel-text">{title || "Unnamed Panel"}</p>
        <div style={{"display": "flex", "flex-direction": "row", cursor: "copy", "align-self": "flex-end"}}
             onClick={copyToClipboard}
        >
            <FiCopy/>
            <p class="panel-text"> ({contentLength()} bytes)</p>
        </div>
    </div>;
}

export default PanelHeader;
