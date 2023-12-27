import initWasm, { wasm_compile, wasm_run_bf } from "../compiler/pkg";
import { MastermindConfig } from "./panels/SettingsPanel";

initWasm();

function _compile(
  fileMap: Record<string, string>,
  entryFileName: string,
  optimisations: MastermindConfig
) {
  return wasm_compile(fileMap, entryFileName, optimisations);
}

// TODO: add input streaming stuff
function _run(code: string) {
  const result = wasm_run_bf(code);
  return result;
}

const err_msg =
  "There was an uncaught error in WASM-based Mastermind compilation or Brainfuck execution.\n\
This should never happen, if issues persist please raise an issue on the Mastermind GitHub page. \
(https://github.com/Heathcorp/Mastermind)\n\
(raw rust panic message below)\n";

// please don't send multiple commands to this worker at the same time
onmessage = ({ data }) => {
  switch (data.command) {
    case "COMPILE":
      // overwrite the console error function as I can't seem to catch wasm errors
      const old_error = console.error;
      console.error = (...args) => {
        old_error(err_msg);
        old_error(...args);

        postMessage({
          transaction: data.transaction,
          success: false,
          message: `${err_msg}\n${args.toString()}`,
        });
        // remove the wrapper function, is this legit js?
        console.error = old_error;
      };
      const compiledCode = _compile(
        data.arguments.fileMap,
        data.arguments.entryFileName,
        data.arguments.optimisations
      );
      postMessage({
        transaction: data.transaction,
        success: true,
        message: compiledCode,
      });
      break;
    case "RUN":
      const codeOutput = _run(data.arguments.code);
      postMessage({
        transaction: data.transaction,
        success: true,
        message: codeOutput,
      });
      break;
  }
};
