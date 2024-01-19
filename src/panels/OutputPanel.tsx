import { Component, createEffect, on, JSX } from "solid-js";

import { EditorView, drawSelection } from "@codemirror/view";
import { EditorState } from "@codemirror/state";

import { useAppContext } from "../App";

const OutputPanel: Component<{ style?: JSX.CSSProperties }> = (props) => {
  const app = useAppContext()!;
  // this component could handle logic for line by line output and auto scrolling
  // that is why this component even exists
  let editorView: EditorView | undefined;

  createEffect(
    on([() => !!editorView, app.output], () => {
      const output = app?.output();
      if (!editorView || !output) return;
      editorView.dispatch({
        changes: {
          from: 0,
          to: editorView.state.doc.length,
          insert: output.content,
        },
      });
    })
  );

  return <div class="panel output-panel" style={props.style} ref={e => {
    editorView = new EditorView({
      parent: e,
      state: EditorState.create({
        extensions: [
          drawSelection(),
          EditorView.lineWrapping,
          EditorView.editable.of(false),
        ],
      }),
    });
  }} />;
};

export default OutputPanel;
