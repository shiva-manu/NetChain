/// <reference types="vite/client" />

interface ImportMetaEnv {
  readonly VITE_NETCHAIN_RPC_URL?: string;
  readonly VITE_NETCHAIN_MONITORING_URL?: string;
  readonly VITE_NETCHAIN_WS_URL?: string;
}

interface ImportMeta {
  readonly env: ImportMetaEnv;
}
