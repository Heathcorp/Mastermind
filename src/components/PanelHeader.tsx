import "./components.css";
import {FiCopy} from "solid-icons/fi";

function PanelHeader({title, content} : {title: string, content: string}) {

    return <div class="panel-header">
        <p class="panel-text">{title || "Unnamed Panel"}</p>
        <div style={{"display": "flex", "flex-direction": "row", cursor: "copy", "align-self": "flex-end"}}
             onClick={() => {
                 if (!content) return;
                 window.navigator.clipboard
                     .writeText(content)
                     .then(() => window.alert("Output copied to clipboard!"));
             }}
        >
            <FiCopy/>
            <p class="panel-text"> ({content.length} bytes)</p>
        </div>
    </div>;
}

export default PanelHeader;
