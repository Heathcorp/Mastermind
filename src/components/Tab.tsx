import { Component } from "solid-js";
import { AiOutlineDelete } from "solid-icons/ai";

import "./components.css";

const Tab: Component<{ label: string; selected?: boolean }> = ({
  label,
  selected = false,
}) => {
  return (
    <div classList={{ ["tab"]: true, ["tab-selected"]: selected }}>
      {label} <AiOutlineDelete />
    </div>
  );
};

export default Tab;
