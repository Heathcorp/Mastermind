import { Component } from "solid-js";

const OutputPanel: Component<{ outputText: string }> = (props) => {
  // this component could handle logic for line by line output and auto scrolling
  // that is why this component even exists
  return <div class="panel output-panel">{props.outputText}</div>;
};

export default OutputPanel;
