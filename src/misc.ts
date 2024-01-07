import {
  lineNumbers,
  highlightActiveLineGutter,
  highlightSpecialChars,
  drawSelection,
  dropCursor,
  rectangularSelection,
  crosshairCursor,
  highlightActiveLine,
  keymap,
} from "@codemirror/view";
import {
  foldGutter,
  indentOnInput,
  syntaxHighlighting,
  defaultHighlightStyle,
  bracketMatching,
  foldKeymap,
} from "@codemirror/language";
import {
  history,
  defaultKeymap,
  historyKeymap,
  indentWithTab,
} from "@codemirror/commands";
import { highlightSelectionMatches, searchKeymap } from "@codemirror/search";
import {
  closeBrackets,
  autocompletion,
  closeBracketsKeymap,
  completionKeymap,
} from "@codemirror/autocomplete";
import { lintKeymap } from "@codemirror/lint";
import { LRLanguage, LanguageSupport } from "@codemirror/language";

import { vscodeDark } from "@uiw/codemirror-themes-all";
import { EditorState } from "@codemirror/state";

import { parser } from "./lexer/mastermind_parser";
import { styleTags, tags } from "@lezer/highlight";

export const defaultExtensions = [
  mastermindLanguageSupport(),
  vscodeDark,
  lineNumbers(),
  highlightActiveLineGutter(),
  highlightSpecialChars(),
  history(),
  foldGutter(),
  drawSelection(),
  dropCursor(),
  EditorState.allowMultipleSelections.of(true),
  indentOnInput(),
  syntaxHighlighting(defaultHighlightStyle, { fallback: true }),
  bracketMatching(),
  closeBrackets(),
  autocompletion(),
  rectangularSelection(),
  crosshairCursor(),
  highlightActiveLine(),
  highlightSelectionMatches(),
  keymap.of([
    ...closeBracketsKeymap,
    ...defaultKeymap,
    ...searchKeymap,
    ...historyKeymap,
    ...foldKeymap,
    ...completionKeymap,
    ...lintKeymap,
    // TODO: add a warning to let users escape the editor by pressing the esc key
    //  remember codemirror docos
    indentWithTab,
  ]),
];

function mastermindLanguageSupport() {
  return new LanguageSupport(LRLanguage.define({
    parser: parser.configure({
      props: [styleTags({
        "DefClause/Def": tags.function(tags.definitionKeyword),
        "DefClause/Name": tags.function(tags.definition(tags.variableName)),
        "CallClause/Name": tags.function(tags.variableName),
        "LetClause/Let": tags.definitionKeyword,
        "VariableDefinition/Name": tags.variableName,


        "OutputClause/Output": tags.controlKeyword,
        "InputClause/Input": tags.controlKeyword,
        "DrainCopyClause/DrainCopy DrainCopyClause/Into": tags.controlKeyword,
        "WhileClause/While": tags.controlKeyword,
        "IfElseClause/If IfElseClause/Not IfElseClause/Else": tags.controlKeyword,

        Comment: tags.lineComment,
        Include: tags.moduleKeyword,
        IncludePath: tags.string,

        Boolean: tags.bool,
        Number: tags.integer,
        Character: tags.character,
        String: tags.string,
        "VariableTarget/Name": tags.variableName,

        SquareBrackets: tags.squareBracket,
        Block: tags.brace,
        Parentheses: tags.paren,

        EqualOp: tags.updateOperator,
        AddEqualOp: tags.updateOperator,
        AddOp: tags.arithmeticOperator,
        IncDecOp: tags.updateOperator,
        "Semicolon Comma": tags.separator
      })]
    }),
    languageData: {
      commentTokens: { line: "//" },
    }
  }));
} 
