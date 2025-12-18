/// <reference types="vite/client" />

interface ImportMetaEnv {
  readonly VITE_GIT_COMMIT_HASH: string;
  readonly VITE_GIT_COMMIT_BRANCH: string;
  readonly VITE_CODE_VERSION_COLOUR_HINT: string;
}

interface ImportMeta {
  readonly env: ImportMetaEnv;
}
