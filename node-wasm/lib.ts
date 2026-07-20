import { EventEmitter } from "eventemitter3";
import { v4 as uuidv4 } from "uuid";
import initWasm, { DataNetworkClient } from "./pkg";

let initPromise: Promise<any> | null = null;

async function ensureInitialized() {
  if (!initPromise) {
    initPromise = initWasm();
  }
  await initPromise;
}

/**
 * Creates a new DataNetworkProvider instance.
 *
 * @param config - Configuration object for the provider
 * @returns A promise that resolves to an initialized DataNetworkProvider instance
 *
 * @remarks
 * This function creates an EIP-1193 compliant Ethereum provider that can be used
 * with popular web3 libraries like viem, ethers.js or web3.js. Treat this the same as
 * you would `window.ethereum` when constructing a provider.
 *
 * @example
 * ```typescript
 * const provider = await createDataNetworkProvider({
 *   executionRpc: "https://story-rpc.publicnode.com",
 *   consensusRpc: "https://story-consensus-rpc.publicnode.com",
 *   network: "mainnet",
 *   trustHeight: "20038975",
 *   trustHash: "37963EEE3CD1478F17CAAFFA4B9916ACD04C25DDF409386EDF9F65ACB3A55E44"
 * });
 * ```
 */
export async function createDataNetworkProvider(config: Config): Promise<DataNetworkProvider> {
  await ensureInitialized();
  return DataNetworkProvider.createInternal(config);
}

/**
 * An EIP-1193 compliant Ethereum provider powered by the DATA Network light client.
 *
 * @remarks
 * DataNetworkProvider implements the Ethereum Provider API (EIP-1193) and can be used
 * as a provider for DATA Network in web3 applications.
 *
 * The provider supports all standard Ethereum JSON-RPC methods and maintains
 * compatibility with popular libraries like viem, ethers.js, and web3.js.
 */
export class DataNetworkProvider {
  #client;
  #chainId;
  #eventEmitter;
  #closed = false;
  #subscriptionIds: Set<string> = new Set();

  private constructor(config: Config) {
    const executionRpc = config.executionRpc;
    const verifiableApi = config.verifiableApi;
    const consensusRpc = config.consensusRpc;
    const network = config.network ?? "mainnet";
    const dbType = config.dbType ?? "localstorage";

    this.#client = new DataNetworkClient(
      executionRpc,
      verifiableApi,
      consensusRpc,
      network,
      config.trustHeight,
      config.trustHash,
      dbType
    );

    this.#chainId = this.#client.chain_id();
    this.#eventEmitter = new EventEmitter();
  }

  /** @internal */
  static createInternal(config: Config): DataNetworkProvider {
    return new DataNetworkProvider(config);
  }


  /**
   * Waits for the light client to sync with the network.
   * 
   * @returns A promise that resolves when the client is fully synced
   * 
   * @remarks
   * This method blocks until the light client has synchronized with the network
   * and is ready to process requests. It's recommended to call this before
   * making any RPC requests to ensure accurate data.
   * 
   * @example
   * ```typescript
   * const provider = await createDataNetworkProvider(config);
   * await provider.waitSynced();
   * console.log("Provider is ready!");
   * ```
   */
  async waitSynced() {
    await this.#client.wait_synced();
  }

  /**
   * Sends an RPC request to the provider.
   * 
   * @param req - The RPC request object containing method and params
   * @returns A promise that resolves with the RPC response
   * @throws {Error} If the RPC method is not supported or the request fails
   * 
   * @remarks
   * This is the main entry point for all Ethereum JSON-RPC requests. It implements
   * the EIP-1193 provider interface and supports all standard Ethereum RPC methods.
   * 
   * @example
   * ```typescript
   * // Get the latest block number
   * const blockNumber = await provider.request({
   *   method: "eth_blockNumber",
   *   params: []
   * });
   * 
   * // Get account balance
   * const balance = await provider.request({
   *   method: "eth_getBalance",
   *   params: [address, "latest"]
   * });
   * ```
   */
  async request(req: Request): Promise<any> {
    if (this.#closed) {
      throw new Error("Provider has been shut down");
    }
    try {
      return await this.#req(req);
    } catch (err) {
      throw new Error(err.toString());
    }
  }

  async #req(req: Request): Promise<any> {
    switch (req.method) {
      case "eth_getBalance": {
        return this.#client.get_balance(req.params[0], req.params[1]);
      }
      case "eth_chainId": {
        return `0x${this.#chainId.toString(16)}`;
      }
      case "eth_blockNumber": {
        return this.#client.get_block_number();
      }
      case "eth_getTransactionByHash": {
        let tx = await this.#client.get_transaction_by_hash(req.params[0]);
        return mapToObj(tx);
      }
      case "eth_getTransactionCount": {
        return this.#client.get_transaction_count(req.params[0], req.params[1]);
      }
      case "eth_getBlockTransactionCountByHash": {
        return this.#client.get_block_transaction_count_by_hash(req.params[0]);
      }
      case "eth_getBlockTransactionCountByNumber": {
        return this.#client.get_block_transaction_count_by_number(
          req.params[0]
        );
      }
      case "eth_getCode": {
        return this.#client.get_code(req.params[0], req.params[1]);
      }
      case "eth_getStorageAt": {
        return this.#client.get_storage_at(req.params[0], req.params[1], req.params[2]);
      }
      case "eth_getProof": {
        return this.#client.get_proof(req.params[0], req.params[1], req.params[2]);
      }
      case "eth_call": {
        return this.#client.call(req.params[0], req.params[1], req.params[2]);
      }
      case "eth_estimateGas": {
        return this.#client.estimate_gas(req.params[0], req.params[1], req.params[2]);
      }
      case "eth_createAccessList": {
        return this.#client.create_access_list(req.params[0], req.params[1], req.params[2]);
      }
      case "eth_gasPrice": {
        return this.#client.gas_price();
      }
      case "eth_maxPriorityFeePerGas": {
        return this.#client.max_priority_fee_per_gas();
      }
      case "eth_sendRawTransaction": {
        return this.#client.send_raw_transaction(req.params[0]);
      }
      case "eth_getTransactionReceipt": {
        const receipt = await this.#client.get_transaction_receipt(req.params[0]);
        return mapToObj(receipt);
      }
      case "eth_getTransactionByBlockHashAndIndex": {
        const tx = await this.#client.get_transaction_by_block_hash_and_index(
          req.params[0],
          req.params[1]
        );
        return mapToObj(tx);
      }
      case "eth_getTransactionByBlockNumberAndIndex": {
        const tx = await this.#client.get_transaction_by_block_number_and_index(
          req.params[0],
          req.params[1]
        );
        return mapToObj(tx);
      }
      case "eth_getBlockReceipts": {
        const receipts = await this.#client.get_block_receipts(req.params[0]);
        return receipts.map(mapToObj);
      }
      case "eth_getLogs": {
        const logs = await this.#client.get_logs(req.params[0]);
        return logs.map(mapToObj);
      }
      case "eth_getFilterLogs": {
        const logs = await this.#client.get_filter_logs(req.params[0]);
        return logs.map(mapToObj);
      }
      case "eth_uninstallFilter": {
        return this.#client.uninstall_filter(req.params[0]);
      }
      case "eth_newFilter": {
        return this.#client.new_filter(req.params[0]);
      }
      case "eth_newBlockFilter": {
        return this.#client.new_block_filter();
      }
      case "net_version": {
        return this.#chainId;
      }
      case "eth_getBlockByNumber": {
        const block = await this.#client.get_block_by_number(req.params[0], req.params[1]);
        return mapToObj(block);
      }
      case "eth_getBlockByHash": {
        const block = await this.#client.get_block_by_hash(req.params[0], req.params[1]);
        return mapToObj(block);
      }
      case "web3_clientVersion": {
        return this.#client.client_version();
      }
      case "eth_subscribe": {
        return this.#handleSubscribe(req);
      }
      case "eth_unsubscribe": {
        const id = req.params[0];
        this.#subscriptionIds.delete(id);
        return this.#client.unsubscribe(id);
      }
      default: {
        throw new Error(`method not supported: ${req.method}`);
      }
    }
  }

  async #handleSubscribe(req: Request) {
    try {
      const id = uuidv4();
      // Capture only the emitter reference, not `this`
      const emitter = this.#eventEmitter;
      await this.#client.subscribe(req.params[0], id, (data: any, subId: string) => {
        const result = data instanceof Map ? mapToObj(data) : data;
        const payload = {
          type: 'eth_subscription',
          data: {
            subscription: subId,
            result,
          },
        };
        emitter.emit("message", payload);
      });
      this.#subscriptionIds.add(id);
      return id;
    } catch (err) {
      throw new Error(err.toString());
    }
  }

  /**
   * Registers an event listener for provider events.
   * 
   * @param eventName - The name of the event to listen for
   * @param handler - The callback function to handle the event
   * 
   * @remarks
   * Supports standard EIP-1193 provider events including:
   * - `message` - For subscription updates
   * - `connect` - When the provider connects
   * - `disconnect` - When the provider disconnects
   * - `chainChanged` - When the chain ID changes
   * - `accountsChanged` - When accounts change (if applicable)
   * 
   * @example
   * ```typescript
   * provider.on("message", (message) => {
   *   console.log("Received message:", message);
   * });
   * 
   * // Subscribe to new blocks
   * const subId = await provider.request({
   *   method: "eth_subscribe",
   *   params: ["newHeads"]
   * });
   * ```
   */
  on(
    eventName: string,
    handler: (data: any) => void
  ): this {
    this.#eventEmitter.on(eventName, handler);
    return this;
  }

  /**
   * Removes an event listener from the provider.
   * 
   * @param eventName - The name of the event to stop listening for
   * @param handler - The callback function to remove
   * 
   * @remarks
   * Removes a previously registered event listener. The handler must be
   * the same function reference that was passed to `on()`.
   * 
   * @example
   * ```typescript
   * const handler = (data) => console.log(data);
   * 
   * // Add listener
   * provider.on("message", handler);
   * 
   * // Remove listener
   * provider.removeListener("message", handler);
   * ```
   */
  removeListener(
    eventName: string,
    handler: (data: any) => void
  ): this {
    this.#eventEmitter.off(eventName, handler);
    return this;
  }

  /**
   * Shuts down the provider and releases all resources.
   * 
   * @returns A promise that resolves when the provider has been shut down
   * 
   * @remarks
   * After shutdown:
   * - All future `request()` calls will reject with an error
   * - All active subscriptions are unsubscribed
   * - All event listeners are removed
   * - Background tasks are stopped
   * 
   * The provider instance will be garbage collected after the user drops all references.
   * 
   * @example
   * ```typescript
   * const provider = await createDataNetworkProvider(config);
   * 
   * // ... use the provider ...
   * 
   * // Clean up when done
   * await provider.shutdown();
   * ```
   */
  async shutdown(): Promise<void> {
    if (this.#closed) {
      return;
    }
    this.#closed = true;

    for (const id of this.#subscriptionIds) {
      try {
        this.#client.unsubscribe(id);
      } catch {
        // Ignore errors during cleanup
      }
    }
    this.#subscriptionIds.clear();
    await this.#client.shutdown();
    this.#eventEmitter.removeAllListeners();
    this.#client.free();
  }
  
  /**
   * This method is equivalent to `shutdown()`
   */
  async destroy(): Promise<void> {
    await this.shutdown();
  }
}

/**
 * Configuration options for creating a DATA Network provider.
 */
export type Config = {
  /**
   * The RPC endpoint for execution layer requests.
   * Required unless verifiableApi is provided.
   * @example "https://story-rpc.publicnode.com"
   */
  executionRpc?: string;
  
  /**
   * The verifiable API endpoint.
   * Not recommended for use currently.
   */
  verifiableApi?: string;
  
  /**
   * The CometBFT RPC endpoint used to fetch light blocks.
   * @example "https://story-consensus-rpc.publicnode.com"
   */
  consensusRpc?: string;

  /**
   * Height of the trusted CometBFT block.
   * Must be provided together with trustHash unless localStorage already contains trust options.
   */
  trustHeight?: string;

  /**
   * Hash of the trusted CometBFT block.
   * Must be provided together with trustHeight unless localStorage already contains trust options.
   */
  trustHash?: string;

  /**
   * The network to connect to.
   * Defaults to DATA Network mainnet.
   */
  network?: Network;

  /**
   * Where to persist CometBFT trust options.
   * @defaultValue "localstorage"
   * @remarks
   * - `localstorage` - Store updated trust options in browser localStorage
   * - `config` - Keep the configured trust options in memory
   */
  dbType?: "localstorage" | "config";
};

export type Network = "mainnet" | "aeneid";

type Request = {
  method: string;
  params: any[];
};

/**
 * Converts a Map to an object, including nested Maps and arrays of Maps.
 * IMPORTANT: This function will mutate input!
 * 
 * @param map - The Map to convert
 * @returns The converted object
 */
function mapToObj(map: Map<any, any> | undefined): Record<string, any> | undefined {
  if (!map) return undefined;

  const result: Record<string, any> = {};
  
  for (const [key, value] of map) {
    if (value === undefined) continue;

    if (value instanceof Map) {
      result[key] = mapToObj(value);
    } else if (Array.isArray(value)) {
      for (let i = 0; i < value.length; i++) {
        if (value[i] instanceof Map) {
          // Mutate in-place
          value[i] = mapToObj(value[i]);
        }
      }
      result[key] = value;
    } else {
      result[key] = value;
    }
  }
  return result;
}
