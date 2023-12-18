import { Component, createEffect, on } from "solid-js";

import { EditorView } from "@codemirror/view";
import { EditorState } from "@codemirror/state";

import { dropCursor } from "@codemirror/view";

const OutputPanel: Component<{ outputText: string }> = (props) => {
  // this component could handle logic for line by line output and auto scrolling
  // that is why this component even exists
  let editorRef: HTMLDivElement | undefined;
  let editorView: EditorView | undefined;
  createEffect(
    on([() => editorRef], () => {
      if (editorRef) {
        editorView = new EditorView({
          parent: editorRef,
          state: EditorState.create({
            extensions: [
              dropCursor(),
              // drawSelection(),
              EditorView.lineWrapping,
              EditorView.editable.of(false),
            ],
          }),
        });
      }
    })
  );

  createEffect(
    on([() => !!editorView, () => props.outputText], () => {
      if (!editorView) return;
      editorView.dispatch({
        changes: {
          from: 0,
          to: editorView.state.doc.length,
          insert: props.outputText,
        },
      });
    })
  );
  return <div class="panel output-panel" ref={editorRef}></div>;
};

export default OutputPanel;
