import { EditorState, EditorSelection } from "@codemirror/state";
import { EditorView, layer, RectangleMarker, keymap } from "@codemirror/view";
import {
  drawSelection,
} from "@codemirror/view";
import {
  defaultKeymap,
} from "@codemirror/commands";


import { Component, JSX, createEffect, on } from "solid-js";

import './input.css';
import { useAppContext } from "../App";
import PanelHeader from "../components/PanelHeader.tsx";
import Divider from "../components/Divider.tsx";
// import { defaultExtensions } from "../misc";

const InputPanel: Component<{ style?: JSX.CSSProperties }> = (props) => {
  const app = useAppContext()!;
  // when the compiler is idle, allow the user to edit freely
  // when the compiler is running code, the user can only append
  const getInputText = () => app.input().text || "";

  let editorView: EditorView | undefined;

  createEffect(on([() => editorView, app.input], () => {
    if (!editorView) return;

    // update the editorView when the input changes so that the layers re-render
    editorView.dispatch();
  }));

  return <div class="panel input-panel" style={props.style}>
    <PanelHeader title={'Input'} getContent={getInputText}/>
    <Divider/>
    <div class="panel input-panel" style={props.style} ref={e => {
      editorView = new EditorView({
        parent: e,
        state: EditorState.create({
          doc: app.input().text,
          extensions: [
            EditorView.lineWrapping,
            drawSelection(),
            keymap.of(defaultKeymap),
            EditorView.updateListener.of((update) => {
              const {amountRead} = app.input();
              if (!update.docChanged) return;

              if (amountRead && update.changes.touchesRange(0, amountRead - 1)) {
                // revert the change if the update affects the readonly portion of the input
                editorView?.setState(update.startState);
              } else {
                const newText = update.state.doc.toString();
                // Update both the app state and local state
                app.setInput((prev) => ({...prev, text: newText}));
              }
            }),
            layer({
              above: true, class: "input-marker-layer", markers(view) {
                const {text, amountRead} = app.input();

                const markers = [];
                if (amountRead) markers.push(...RectangleMarker.forRange(view, 'input-readonly-marker', EditorSelection.single(0, amountRead).main));

                if (amountRead !== null) markers.push(...RectangleMarker.forRange(view, (amountRead >= text.length ? 'input-cursor-waiting-marker' : 'input-cursor-marker'), EditorSelection.single(amountRead).main));

                return markers;
              }, update(_update, _layer) {
                // always update if something changes or if the above createEffect does an empty dispatch
                // this triggers the above markers() function
                return true;
              },
            })
          ]
        })
      });
    }}/>
  </div>;
};

export default InputPanel;
