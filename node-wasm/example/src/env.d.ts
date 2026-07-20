/// <reference types="vite/client" />

interface ImportMetaEnv {
  readonly VITE_DATA_EXECUTION_RPC?: string;
  readonly VITE_DATA_CONSENSUS_RPC?: string;
  readonly VITE_DATA_TRUST_HEIGHT?: string;
  readonly VITE_DATA_TRUST_HASH?: string;
}

interface ImportMeta {
  readonly env: ImportMetaEnv;
}
