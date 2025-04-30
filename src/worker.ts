import initWasm, { wasm_compile, wasm_run_bf } from "../compiler/pkg";
import {MastermindConfig} from "./panels/CompilerPanel.tsx";

import { v4 as uuidv4 } from "uuid";

initWasm();

const err_msg =
  "There was an uncaught error in WASM-based Mastermind compilation or Brainfuck execution.\n\
This should never happen, if issues persist please raise an issue on the Mastermind GitHub page. \
(https://github.com/Heathcorp/Mastermind)\n\
(raw rust panic message below)\n";

// please don't send multiple commands to this worker at the same time
self.addEventListener("message", ({ data }: MessageEvent<
  ({
    transaction: string,
    command: "COMPILE",
    arguments: {
      fileMap: Record<string, string>,
      entryFileName: string,
      config: MastermindConfig
    }
  } |
  {
    transaction: string,
    command: "RUN",
    arguments: {
      code: string,
      enable_2d_grid: boolean
    }
  })
>) => {
  switch (data.command) {
    case "COMPILE":
      // overwrite the console error function as I can't seem to catch wasm panics
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
        // TODO: make this more robust as it is not guaranteed to clean up?
        console.error = old_error;
      };
      try {
        const compiledCode = wasm_compile(
          data.arguments.fileMap,
          data.arguments.entryFileName,
          data.arguments.config
        );

        postMessage({
          transaction: data.transaction,
          success: true,
          message: compiledCode,
        });
      } catch (e) {
        // rust function returned Err(string), didn't panic
        postMessage({
          transaction: data.transaction,
          success: false,
          message: `Error:\n${e}`
        });
      }
      break;
    case "RUN":
      try {
        _run(data.arguments.code, data.arguments.enable_2d_grid, data.transaction).then(codeOutput => {
          console.log(codeOutput);
          postMessage({
            transaction: data.transaction,
            success: true,
            message: codeOutput,
          });
        }).catch((reason) => {
          console.log("FAIL", reason);
          postMessage({
            transaction: data.transaction,
            success: false,
            message: reason,
          });
        });
      } catch (e) {
        postMessage({
          transaction: data.transaction,
          success: false,
          message: `Error:\n${e}`
        });
      }

      break;
  }
});

function _run(code: string, enable_2d_grid: boolean, runTransaction: string) {
  const result = wasm_run_bf(code, enable_2d_grid,

    function (byte: number) {
      // output a byte from the BVM
      postMessage({ transaction: runTransaction, command: "OUTPUT_BYTE", arguments: { byte } });
    },

    function (): Promise<number> {
      // TODO: set a timeout maybe for fault tolerance
      // get a byte from the input buffer and return it to the BVM
      return new Promise((resolve, _reject) => {
        const t = uuidv4();

        const callback = ({ data }: MessageEvent<{
          transaction: string,
          command: string,
          arguments: { byte: number }
        }>) => {
          if (data.transaction !== t || data.command !== "INPUT_BYTE") return;
          // received this back from main thread

          resolve(data.arguments.byte);

          self.removeEventListener("message", callback);
        }
        self.addEventListener("message", callback);

        // send a request back to the main thread for a character
        // this is a bit poorly done, the main transaction is for the run command, the argument transaction is for this input request
        postMessage({ command: "REQUEST_INPUT", transaction: runTransaction, arguments: { transaction: t } });
      })
    }

  );
  return result;
}
