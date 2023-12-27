import { Component, createEffect, on } from "solid-js";

import { EditorView } from "@codemirror/view";
import { EditorState } from "@codemirror/state";

import { dropCursor } from "@codemirror/view";
import { useAppContext } from "../App";

const OutputPanel: Component<{}> = () => {
  const app = useAppContext()!;
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
  return <div class="panel output-panel" ref={editorRef}></div>;
};

export default OutputPanel;
