/// <reference types="vite/client" />

interface ImportMetaEnv {
  readonly VITE_DAEMON_HTTP?: string;
  readonly VITE_DAEMON_WS?: string;
}

interface ImportMeta {
  readonly env: ImportMetaEnv;
}
