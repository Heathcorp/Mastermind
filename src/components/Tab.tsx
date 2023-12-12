import { Component } from "solid-js";
import { AiOutlineDelete } from "solid-icons/ai";

import "./components.css";

const Tab: Component<{
  label: string;
  selected?: boolean;
  onSelect: () => void;
}> = (props) => {
  return (
    <div
      classList={{ ["tab"]: true, ["tab-selected"]: props.selected }}
      onClick={props.onSelect}
    >
      {props.label} <AiOutlineDelete />
    </div>
  );
};

export default Tab;
