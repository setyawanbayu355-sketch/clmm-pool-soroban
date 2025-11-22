import { Buffer } from "buffer";
import { Address } from '@stellar/stellar-sdk';
import {
  AssembledTransaction,
  Client as ContractClient,
  ClientOptions as ContractClientOptions,
  MethodOptions,
  Result,
  Spec as ContractSpec,
} from '@stellar/stellar-sdk/contract';
import type {
  u32,
  i32,
  u64,
  i64,
  u128,
  i128,
  u256,
  i256,
  Option,
  Typepoint,
  Duration,
} from '@stellar/stellar-sdk/contract';
export * from '@stellar/stellar-sdk'
export * as contract from '@stellar/stellar-sdk/contract'
export * as rpc from '@stellar/stellar-sdk/rpc'

if (typeof window !== 'undefined') {
  //@ts-ignore Buffer exists
  window.Buffer = window.Buffer || Buffer;
}




/**
 * Key untuk storage
 */
export type DataKey = {tag: "PoolState", values: void} | {tag: "Tick", values: readonly [i32]} | {tag: "Position", values: readonly [string, i32, i32]};


/**
 * Posisi LP per user & range tick
 */
export interface Position {
  /**
 * Liquidity yang dimiliki user di range [tick_lower, tick_upper)
 */
liquidity: i128;
}


/**
 * Info per tick (versi bayi)
 */
export interface TickInfo {
  /**
 * Total liquidity yang “nempel” di tick ini
 */
liquidity_gross: i128;
  /**
 * Perubahan liquidity ketika harga melewati tick ini
 * (biasanya +L di tick_lower, -L di tick_upper)
 */
liquidity_net: i128;
}


/**
 * State utama pool (versi bayi)
 */
export interface PoolState {
  /**
 * Tick aktif sekarang
 */
current_tick: i32;
  /**
 * Total liquidity global
 */
liquidity: i128;
  /**
 * Harga dalam bentuk sqrt(P) fixed-point (sementara diisi manual dulu)
 */
sqrt_price_x64: i128;
}

export interface Client {
  /**
   * Construct and simulate a initialize transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Inisialisasi pool pertama kali
   */
  initialize: ({sqrt_price_x64, current_tick}: {sqrt_price_x64: i128, current_tick: i32}, options?: {
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number;

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number;

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean;
  }) => Promise<AssembledTransaction<null>>

  /**
   * Construct and simulate a get_position transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Baca Position user untuk satu range tick
   */
  get_position: ({owner, tick_lower, tick_upper}: {owner: string, tick_lower: i32, tick_upper: i32}, options?: {
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number;

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number;

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean;
  }) => Promise<AssembledTransaction<Position>>

  /**
   * Construct and simulate a add_liquidity transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Tambah liquidity (VERSI BAYI)
   * 
   * - update liquidity global
   * - update TickInfo di tick_lower dan tick_upper
   * - update Position(user, tick_lower, tick_upper)
   * 
   * Belum ada token transfer, belum fee.
   */
  add_liquidity: ({owner, tick_lower, tick_upper, amount}: {owner: string, tick_lower: i32, tick_upper: i32, amount: i128}, options?: {
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number;

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number;

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean;
  }) => Promise<AssembledTransaction<null>>

  /**
   * Construct and simulate a get_tick_info transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Baca TickInfo untuk 1 tick
   */
  get_tick_info: ({tick}: {tick: i32}, options?: {
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number;

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number;

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean;
  }) => Promise<AssembledTransaction<TickInfo>>

  /**
   * Construct and simulate a get_pool_state transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Baca state pool buat UI / debugging
   */
  get_pool_state: (options?: {
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number;

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number;

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean;
  }) => Promise<AssembledTransaction<PoolState>>

}
export class Client extends ContractClient {
  static async deploy<T = Client>(
    /** Options for initializing a Client as well as for calling a method, with extras specific to deploying. */
    options: MethodOptions &
      Omit<ContractClientOptions, "contractId"> & {
        /** The hash of the Wasm blob, which must already be installed on-chain. */
        wasmHash: Buffer | string;
        /** Salt used to generate the contract's ID. Passed through to {@link Operation.createCustomContract}. Default: random. */
        salt?: Buffer | Uint8Array;
        /** The format used to decode `wasmHash`, if it's provided as a string. */
        format?: "hex" | "base64";
      }
  ): Promise<AssembledTransaction<T>> {
    return ContractClient.deploy(null, options)
  }
  constructor(public readonly options: ContractClientOptions) {
    super(
      new ContractSpec([ "AAAAAgAAABFLZXkgdW50dWsgc3RvcmFnZQAAAAAAAAAAAAAHRGF0YUtleQAAAAADAAAAAAAAAAAAAAAJUG9vbFN0YXRlAAAAAAAAAQAAAAAAAAAEVGljawAAAAEAAAAFAAAAAQAAAAAAAAAIUG9zaXRpb24AAAADAAAAEwAAAAUAAAAF",
        "AAAAAAAAAB5JbmlzaWFsaXNhc2kgcG9vbCBwZXJ0YW1hIGthbGkAAAAAAAppbml0aWFsaXplAAAAAAACAAAAAAAAAA5zcXJ0X3ByaWNlX3g2NAAAAAAACwAAAAAAAAAMY3VycmVudF90aWNrAAAABQAAAAA=",
        "AAAAAQAAAB9Qb3Npc2kgTFAgcGVyIHVzZXIgJiByYW5nZSB0aWNrAAAAAAAAAAAIUG9zaXRpb24AAAABAAAAPkxpcXVpZGl0eSB5YW5nIGRpbWlsaWtpIHVzZXIgZGkgcmFuZ2UgW3RpY2tfbG93ZXIsIHRpY2tfdXBwZXIpAAAAAAAJbGlxdWlkaXR5AAAAAAAACw==",
        "AAAAAQAAABpJbmZvIHBlciB0aWNrICh2ZXJzaSBiYXlpKQAAAAAAAAAAAAhUaWNrSW5mbwAAAAIAAAAtVG90YWwgbGlxdWlkaXR5IHlhbmcg4oCcbmVtcGVs4oCdIGRpIHRpY2sgaW5pAAAAAAAAD2xpcXVpZGl0eV9ncm9zcwAAAAALAAAAYFBlcnViYWhhbiBsaXF1aWRpdHkga2V0aWthIGhhcmdhIG1lbGV3YXRpIHRpY2sgaW5pCihiaWFzYW55YSArTCBkaSB0aWNrX2xvd2VyLCAtTCBkaSB0aWNrX3VwcGVyKQAAAA1saXF1aWRpdHlfbmV0AAAAAAAACw==",
        "AAAAAQAAAB1TdGF0ZSB1dGFtYSBwb29sICh2ZXJzaSBiYXlpKQAAAAAAAAAAAAAJUG9vbFN0YXRlAAAAAAAAAwAAABNUaWNrIGFrdGlmIHNla2FyYW5nAAAAAAxjdXJyZW50X3RpY2sAAAAFAAAAFlRvdGFsIGxpcXVpZGl0eSBnbG9iYWwAAAAAAAlsaXF1aWRpdHkAAAAAAAALAAAAREhhcmdhIGRhbGFtIGJlbnR1ayBzcXJ0KFApIGZpeGVkLXBvaW50IChzZW1lbnRhcmEgZGlpc2kgbWFudWFsIGR1bHUpAAAADnNxcnRfcHJpY2VfeDY0AAAAAAAL",
        "AAAAAAAAAChCYWNhIFBvc2l0aW9uIHVzZXIgdW50dWsgc2F0dSByYW5nZSB0aWNrAAAADGdldF9wb3NpdGlvbgAAAAMAAAAAAAAABW93bmVyAAAAAAAAEwAAAAAAAAAKdGlja19sb3dlcgAAAAAABQAAAAAAAAAKdGlja191cHBlcgAAAAAABQAAAAEAAAfQAAAACFBvc2l0aW9u",
        "AAAAAAAAAL1UYW1iYWggbGlxdWlkaXR5IChWRVJTSSBCQVlJKQoKLSB1cGRhdGUgbGlxdWlkaXR5IGdsb2JhbAotIHVwZGF0ZSBUaWNrSW5mbyBkaSB0aWNrX2xvd2VyIGRhbiB0aWNrX3VwcGVyCi0gdXBkYXRlIFBvc2l0aW9uKHVzZXIsIHRpY2tfbG93ZXIsIHRpY2tfdXBwZXIpCgpCZWx1bSBhZGEgdG9rZW4gdHJhbnNmZXIsIGJlbHVtIGZlZS4AAAAAAAANYWRkX2xpcXVpZGl0eQAAAAAAAAQAAAAAAAAABW93bmVyAAAAAAAAEwAAAAAAAAAKdGlja19sb3dlcgAAAAAABQAAAAAAAAAKdGlja191cHBlcgAAAAAABQAAAAAAAAAGYW1vdW50AAAAAAALAAAAAA==",
        "AAAAAAAAABpCYWNhIFRpY2tJbmZvIHVudHVrIDEgdGljawAAAAAADWdldF90aWNrX2luZm8AAAAAAAABAAAAAAAAAAR0aWNrAAAABQAAAAEAAAfQAAAACFRpY2tJbmZv",
        "AAAAAAAAACNCYWNhIHN0YXRlIHBvb2wgYnVhdCBVSSAvIGRlYnVnZ2luZwAAAAAOZ2V0X3Bvb2xfc3RhdGUAAAAAAAAAAAABAAAH0AAAAAlQb29sU3RhdGUAAAA=" ]),
      options
    )
  }
  public readonly fromJSON = {
    initialize: this.txFromJSON<null>,
        get_position: this.txFromJSON<Position>,
        add_liquidity: this.txFromJSON<null>,
        get_tick_info: this.txFromJSON<TickInfo>,
        get_pool_state: this.txFromJSON<PoolState>
  }
}