import {Component, createEffect, on, JSX, createSignal} from "solid-js";

import { EditorView, drawSelection } from "@codemirror/view";
import { EditorState } from "@codemirror/state";

import { useAppContext } from "../App";
import PanelHeader from "../components/PanelHeader.tsx";
import Divider from "../components/Divider.tsx";

const OutputPanel: Component<{ style?: JSX.CSSProperties }> = (props) => {
  const app = useAppContext()!;
  // this component could handle logic for line by line output and auto scrolling
  // that is why this component even exists
  let editorView: EditorView | undefined;
  let [errorState, setErrorState] = createSignal(false);

  const styles = () => {
    if (errorState()){
      return {
        ...props.style,
          "color": "var(--NEGATIVE)"
      }
    }
    return {...props.style}
  };
  const getOutputText = () => app.output()?.content || "";

  createEffect(
    on([() => !!editorView, app.output], () => {
      const output = app?.output();
      if (!editorView || !output) return;
      if (output.type == "ERROR") {
        setErrorState(true);
      } else {
        setErrorState(false);
      }
      editorView.dispatch({
        changes: {
          from: 0,
          to: editorView.state.doc.length,
          insert: output.content,
        },
      });
    })
  );

  return <div class="panel input-panel" style={props.style}>
      <PanelHeader title={'Output'} getContent={getOutputText}/>
      <Divider/>
      <div class="panel output-panel" style={styles()} ref={e => {
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
      }}/>
  </div>
    ;
    };

    export default OutputPanel;
