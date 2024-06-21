/* tslint:disable */
/* eslint-disable */
/**
*/
export class GameWrapper {
  free(): void;
/**
*/
  constructor();
/**
* @param {number} command_type
* @param {number} button_state
* @param {number} player
*/
  process_input(command_type: number, button_state: number, player: number): void;
/**
* @param {number} delta
*/
  update(delta: number): void;
/**
* @returns {Uint8Array}
*/
  render(): Uint8Array;
}

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly __wbg_gamewrapper_free: (a: number) => void;
  readonly gamewrapper_new: () => number;
  readonly gamewrapper_process_input: (a: number, b: number, c: number, d: number) => void;
  readonly gamewrapper_update: (a: number, b: number) => void;
  readonly gamewrapper_render: (a: number, b: number) => void;
  readonly __wbindgen_add_to_stack_pointer: (a: number) => number;
  readonly __wbindgen_free: (a: number, b: number, c: number) => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;
/**
* Instantiates the given `module`, which can either be bytes or
* a precompiled `WebAssembly.Module`.
*
* @param {SyncInitInput} module
*
* @returns {InitOutput}
*/
export function initSync(module: SyncInitInput): InitOutput;

/**
* If `module_or_path` is {RequestInfo} or {URL}, makes a request and
* for everything else, calls `WebAssembly.instantiate` directly.
*
* @param {InitInput | Promise<InitInput>} module_or_path
*
* @returns {Promise<InitOutput>}
*/
export default function __wbg_init (module_or_path?: InitInput | Promise<InitInput>): Promise<InitOutput>;
