/* tslint:disable */
/* eslint-disable */

export class WasmGameBoy {
  free(): void;
  [Symbol.dispose](): void;
  /**
   * Get game title
   */
  game_title(): string;
  /**
   * Load a save state
   */
  load_state(data: Uint8Array): void;
  /**
   * Run for a specific number of cycles
   */
  run_cycles(cycles: number): void;
  /**
   * Create a save state
   */
  save_state(): Uint8Array;
  /**
   * Get frame count
   */
  frame_count(): bigint;
  /**
   * Check if this is a CGB game
   */
  is_cgb_game(): boolean;
  /**
   * Press a button
   * Button codes: 0=Right, 1=Left, 2=Up, 3=Down, 4=A, 5=B, 6=Select, 7=Start
   */
  press_button(code: number): void;
  /**
   * Get framebuffer width
   */
  screen_width(): number;
  /**
   * Get total cycles executed
   */
  total_cycles(): bigint;
  /**
   * Get framebuffer height
   */
  screen_height(): number;
  /**
   * Release a button
   */
  release_button(code: number): void;
  /**
   * Get framebuffer as a copy (for safer JS access)
   */
  get_framebuffer(): Uint8Array;
  /**
   * Get audio samples (stereo interleaved)
   */
  get_audio_buffer(): Float32Array;
  /**
   * Get audio sample rate
   */
  audio_sample_rate(): number;
  /**
   * Clear audio buffer after reading
   */
  clear_audio_buffer(): void;
  /**
   * Create a new Game Boy emulator instance
   */
  constructor(rom_data: Uint8Array);
  /**
   * Reset the emulator
   */
  reset(): void;
  /**
   * Load SRAM
   */
  load_sram(data: Uint8Array): void;
  /**
   * Run one frame and return pointer to framebuffer
   * The framebuffer is RGBA8888 format, 160x144 pixels
   */
  run_frame(): number;
  /**
   * Save SRAM (battery-backed save data)
   */
  save_sram(): Uint8Array | undefined;
}

export function button_a(): number;

export function button_b(): number;

export function button_down(): number;

export function button_left(): number;

export function button_right(): number;

export function button_select(): number;

export function button_start(): number;

export function button_up(): number;

/**
 * Initialize panic hook for better error messages
 */
export function init(): void;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly __wbg_wasmgameboy_free: (a: number, b: number) => void;
  readonly button_a: () => number;
  readonly button_b: () => number;
  readonly button_down: () => number;
  readonly button_left: () => number;
  readonly button_right: () => number;
  readonly button_select: () => number;
  readonly button_start: () => number;
  readonly button_up: () => number;
  readonly wasmgameboy_audio_sample_rate: (a: number) => number;
  readonly wasmgameboy_clear_audio_buffer: (a: number) => void;
  readonly wasmgameboy_frame_count: (a: number) => bigint;
  readonly wasmgameboy_game_title: (a: number) => [number, number];
  readonly wasmgameboy_get_audio_buffer: (a: number) => [number, number];
  readonly wasmgameboy_get_framebuffer: (a: number) => [number, number];
  readonly wasmgameboy_is_cgb_game: (a: number) => number;
  readonly wasmgameboy_load_sram: (a: number, b: number, c: number) => [number, number];
  readonly wasmgameboy_load_state: (a: number, b: number, c: number) => [number, number];
  readonly wasmgameboy_new: (a: number, b: number) => [number, number, number];
  readonly wasmgameboy_press_button: (a: number, b: number) => void;
  readonly wasmgameboy_release_button: (a: number, b: number) => void;
  readonly wasmgameboy_reset: (a: number) => void;
  readonly wasmgameboy_run_cycles: (a: number, b: number) => void;
  readonly wasmgameboy_run_frame: (a: number) => number;
  readonly wasmgameboy_save_sram: (a: number) => [number, number];
  readonly wasmgameboy_save_state: (a: number) => [number, number];
  readonly wasmgameboy_screen_height: (a: number) => number;
  readonly wasmgameboy_screen_width: (a: number) => number;
  readonly wasmgameboy_total_cycles: (a: number) => bigint;
  readonly init: () => void;
  readonly __wbindgen_free: (a: number, b: number, c: number) => void;
  readonly __wbindgen_malloc: (a: number, b: number) => number;
  readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
  readonly __wbindgen_externrefs: WebAssembly.Table;
  readonly __externref_table_dealloc: (a: number) => void;
  readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;

/**
* Instantiates the given `module`, which can either be bytes or
* a precompiled `WebAssembly.Module`.
*
* @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
*
* @returns {InitOutput}
*/
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
* If `module_or_path` is {RequestInfo} or {URL}, makes a request and
* for everything else, calls `WebAssembly.instantiate` directly.
*
* @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
*
* @returns {Promise<InitOutput>}
*/
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
