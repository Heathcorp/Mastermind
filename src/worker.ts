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

// TODO: add input stuff
function _run(code: string) {
  const result = wasm_run_bf(code);
  return result;
}

onmessage = ({ data }) => {
  switch (data.command) {
    case "COMPILE":
      try {
        const compiledCode = _compile(
          data.arguments.fileMap,
          data.arguments.entryFileName,
          data.arguments.optimisations
        );
        postMessage({ transaction: data.transaction, message: compiledCode });
      } catch (e) {
        console.log(e);
        postMessage({ transaction: data.transaction, message: "error!" });
      }
      break;
    case "RUN":
      const codeOutput = _run(data.arguments.code);
      postMessage({ transaction: data.transaction, message: codeOutput });
      break;
  }
};
