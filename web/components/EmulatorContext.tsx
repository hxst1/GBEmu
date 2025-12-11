"use client";

import React, {
  createContext,
  useContext,
  useState,
  useCallback,
  useRef,
  useEffect,
} from "react";
import { useAudio } from "@/hooks/useAudio";
import { useSaveData } from "@/hooks/useSaveData";

// Button constants matching Rust
export const BUTTONS = {
  RIGHT: 0,
  LEFT: 1,
  UP: 2,
  DOWN: 3,
  A: 4,
  B: 5,
  SELECT: 6,
  START: 7,
} as const;

export type ButtonCode = (typeof BUTTONS)[keyof typeof BUTTONS];

interface EmulatorContextType {
  isRunning: boolean;
  isPaused: boolean;
  gameTitle: string;
  error: string | null;
  framebuffer: Uint8Array | null;
  loadRom: (data: Uint8Array, fileName: string) => Promise<void>;
  reset: () => void;
  pause: () => void;
  resume: () => void;
  pressButton: (button: ButtonCode) => void;
  releaseButton: (button: ButtonCode) => void;
  saveState: () => Promise<void>;
  loadState: () => Promise<void>;
  setVolume: (volume: number) => void;
  volume: number;
  fps: number;
}

const EmulatorContext = createContext<EmulatorContextType | null>(null);

export function useEmulator() {
  const context = useContext(EmulatorContext);
  if (!context) {
    throw new Error("useEmulator must be used within EmulatorProvider");
  }
  return context;
}

interface WasmGameBoy {
  free(): void;
  reset(): void;
  run_frame(): number;
  get_framebuffer(): Uint8Array;
  screen_width(): number;
  screen_height(): number;
  press_button(code: number): void;
  release_button(code: number): void;
  save_sram(): Uint8Array | undefined;
  load_sram(data: Uint8Array): void;
  save_state(): Uint8Array;
  load_state(data: Uint8Array): void;
  game_title(): string;
  is_cgb_game(): boolean;
  get_audio_buffer(): Float32Array;
  clear_audio_buffer(): void;
  audio_sample_rate(): number;
}

// Global WASM module cache
let wasmModule: any = null;

export function EmulatorProvider({ children }: { children: React.ReactNode }) {
  const [isRunning, setIsRunning] = useState(false);
  const [isPaused, setIsPaused] = useState(false);
  const [gameTitle, setGameTitle] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [framebuffer, setFramebuffer] = useState<Uint8Array | null>(null);
  const [volume, setVolume] = useState(0.5);
  const [fps, setFps] = useState(0);

  const emulatorRef = useRef<WasmGameBoy | null>(null);
  const animationRef = useRef<number | null>(null);
  const lastFrameTimeRef = useRef<number>(0);
  const frameCountRef = useRef<number>(0);
  const fpsIntervalRef = useRef<ReturnType<typeof setInterval> | null>(null);
  const currentRomNameRef = useRef<string>("");
  const isPausedRef = useRef(false); // Ref for pause state in loop
  const { initAudio, queueAudio, setAudioVolume, closeAudio } = useAudio();
  const { saveSram, loadSram, saveStateData, loadStateData } = useSaveData();

  // Load WASM module
  const loadWasm = useCallback(async () => {
    if (wasmModule) return wasmModule;

    try {
      // Dynamic import of the JS glue code
      const wasm = await import("@/lib/wasm/gbemu_core.js");

      // Initialize with the WASM file path from public folder
      // Using object syntax to avoid deprecated warning
      await wasm.default({ module_or_path: "/wasm/gbemu_core_bg.wasm" });

      wasmModule = wasm;
      return wasm;
    } catch (err) {
      console.error("Failed to load WASM:", err);
      setError("Failed to load emulator. Please refresh the page.");
      throw err;
    }
  }, []);

  // Load ROM
  const loadRom = useCallback(
    async (data: Uint8Array, fileName: string) => {
      setError(null);

      try {
        const wasm = await loadWasm();

        // Stop any existing emulation
        if (animationRef.current) {
          cancelAnimationFrame(animationRef.current);
          animationRef.current = null;
        }

        if (fpsIntervalRef.current) {
          clearInterval(fpsIntervalRef.current);
          fpsIntervalRef.current = null;
        }

        // Create new emulator instance
        const emulator = new wasm.WasmGameBoy(data);
        emulatorRef.current = emulator;
        currentRomNameRef.current = fileName.replace(/\.[^/.]+$/, "");

        setGameTitle(emulator.game_title());

        // Try to load existing SRAM
        const existingSram = await loadSram(currentRomNameRef.current);
        if (existingSram) {
          try {
            emulator.load_sram(existingSram);
            console.log("Loaded existing save data");
          } catch (e) {
            console.warn("Failed to load save data:", e);
          }
        }

        // Initialize audio
        await initAudio(emulator.audio_sample_rate());
        setAudioVolume(volume);

        setIsRunning(true);
        setIsPaused(false);
        isPausedRef.current = false;

        // Start emulation loop
        startEmulation();
      } catch (err) {
        console.error("Failed to load ROM:", err);
        setError(
          `Failed to load ROM: ${
            err instanceof Error ? err.message : "Unknown error"
          }`
        );
      }
    },
    [loadWasm, loadSram, initAudio, setAudioVolume, volume]
  );

  // Emulation loop
  const startEmulation = useCallback(() => {
    if (!emulatorRef.current) return;

    const targetFrameTime = 1000 / 59.7275; // GB runs at ~59.7 FPS
    let accumulator = 0;

    const loop = (timestamp: number) => {
      if (!emulatorRef.current) return;

      if (isPausedRef.current) {
        animationRef.current = requestAnimationFrame(loop);
        return;
      }

      const deltaTime = timestamp - lastFrameTimeRef.current;
      lastFrameTimeRef.current = timestamp;

      accumulator += deltaTime;

      // Run frames to catch up (cap at 3 frames to prevent spiral)
      let framesRun = 0;
      while (accumulator >= targetFrameTime && framesRun < 3) {
        emulatorRef.current.run_frame();
        accumulator -= targetFrameTime;
        frameCountRef.current++;
        framesRun++;

        // Process audio
        const audioBuffer = emulatorRef.current.get_audio_buffer();
        if (audioBuffer.length > 0) {
          queueAudio(audioBuffer);
          emulatorRef.current.clear_audio_buffer();
        }
      }

      // Reset accumulator if we're too far behind
      if (accumulator > targetFrameTime * 3) {
        accumulator = 0;
      }

      // Update framebuffer
      const fb = emulatorRef.current.get_framebuffer();
      setFramebuffer(new Uint8Array(fb));

      animationRef.current = requestAnimationFrame(loop);
    };

    lastFrameTimeRef.current = performance.now();
    animationRef.current = requestAnimationFrame(loop);

    // FPS counter
    fpsIntervalRef.current = setInterval(() => {
      setFps(frameCountRef.current);
      frameCountRef.current = 0;
    }, 1000);
  }, [queueAudio]);

  // Reset emulator
  const reset = useCallback(() => {
    if (emulatorRef.current) {
      emulatorRef.current.reset();
    }
  }, []);

  // Pause/Resume
  const pause = useCallback(() => {
    isPausedRef.current = true;
    setIsPaused(true);
  }, []);

  const resume = useCallback(() => {
    isPausedRef.current = false;
    setIsPaused(false);
    lastFrameTimeRef.current = performance.now();
  }, []);

  // Button controls
  const pressButton = useCallback((button: ButtonCode) => {
    if (emulatorRef.current) {
      emulatorRef.current.press_button(button);
    }
  }, []);

  const releaseButton = useCallback((button: ButtonCode) => {
    if (emulatorRef.current) {
      emulatorRef.current.release_button(button);
    }
  }, []);

  // Save state
  const saveState = useCallback(async () => {
    if (!emulatorRef.current || !currentRomNameRef.current) return;

    try {
      const stateData = emulatorRef.current.save_state();
      await saveStateData(currentRomNameRef.current, stateData);

      // Also save SRAM
      const sramData = emulatorRef.current.save_sram();
      if (sramData) {
        await saveSram(currentRomNameRef.current, sramData);
      }

      console.log("State saved");
    } catch (err) {
      console.error("Failed to save state:", err);
      setError("Failed to save state");
    }
  }, [saveStateData, saveSram]);

  // Load state
  const loadState = useCallback(async () => {
    if (!emulatorRef.current || !currentRomNameRef.current) return;

    try {
      const stateData = await loadStateData(currentRomNameRef.current);
      if (stateData) {
        emulatorRef.current.load_state(stateData);
        console.log("State loaded");
      }
    } catch (err) {
      console.error("Failed to load state:", err);
      setError("Failed to load state");
    }
  }, [loadStateData]);

  // Volume control
  const handleSetVolume = useCallback(
    (newVolume: number) => {
      setVolume(newVolume);
      setAudioVolume(newVolume);
    },
    [setAudioVolume]
  );

  // Auto-save on interval
  useEffect(() => {
    if (!isRunning || !emulatorRef.current) return;

    const autoSaveInterval = setInterval(async () => {
      const sramData = emulatorRef.current?.save_sram();
      if (sramData && currentRomNameRef.current) {
        await saveSram(currentRomNameRef.current, sramData);
      }
    }, 30000); // Auto-save every 30 seconds

    return () => clearInterval(autoSaveInterval);
  }, [isRunning, saveSram]);

  // Cleanup on unmount
  useEffect(() => {
    return () => {
      if (animationRef.current) {
        cancelAnimationFrame(animationRef.current);
      }
      if (fpsIntervalRef.current) {
        clearInterval(fpsIntervalRef.current);
      }
      closeAudio();
    };
  }, [closeAudio]);

  const value: EmulatorContextType = {
    isRunning,
    isPaused,
    gameTitle,
    error,
    framebuffer,
    loadRom,
    reset,
    pause,
    resume,
    pressButton,
    releaseButton,
    saveState,
    loadState,
    setVolume: handleSetVolume,
    volume,
    fps,
  };

  return (
    <EmulatorContext.Provider value={value}>
      {children}
    </EmulatorContext.Provider>
  );
}
