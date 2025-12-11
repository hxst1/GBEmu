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

// Types matching the actual WASM bindings
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

interface WasmModule {
  WasmGameBoy: new (rom_data: Uint8Array) => WasmGameBoy;
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  default: (options?: { module_or_path?: string }) => Promise<any>;
}

export type ButtonCode = number;

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

interface EmulatorContextType {
  isRunning: boolean;
  isPaused: boolean;
  gameTitle: string;
  error: string | null;
  framebuffer: Uint8Array | null;
  volume: number;
  fps: number;
  isLoading: boolean;
  loadRom: (data: Uint8Array, fileName: string) => Promise<void>;
  pause: () => void;
  resume: () => void;
  reset: () => void;
  pressButton: (button: ButtonCode) => void;
  releaseButton: (button: ButtonCode) => void;
  saveState: () => void;
  loadState: () => void;
  setVolume: (volume: number) => void;
}

const EmulatorContext = createContext<EmulatorContextType | null>(null);

export function useEmulator() {
  const context = useContext(EmulatorContext);
  if (!context) {
    throw new Error("useEmulator must be used within an EmulatorProvider");
  }
  return context;
}

// Global WASM module cache
let wasmModule: WasmModule | null = null;
let wasmLoadingPromise: Promise<WasmModule> | null = null;

export function EmulatorProvider({ children }: { children: React.ReactNode }) {
  const [isRunning, setIsRunning] = useState(false);
  const [isPaused, setIsPaused] = useState(false);
  const [gameTitle, setGameTitle] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [framebuffer, setFramebuffer] = useState<Uint8Array | null>(null);
  const [volume, setVolumeState] = useState(0.5);
  const [fps, setFps] = useState(0);
  const [isLoading, setIsLoading] = useState(false);

  const emulatorRef = useRef<WasmGameBoy | null>(null);
  const animationRef = useRef<number | null>(null);
  const lastFrameTimeRef = useRef<number>(0);
  const frameCountRef = useRef<number>(0);
  const fpsIntervalRef = useRef<ReturnType<typeof setInterval> | null>(null);
  const currentRomNameRef = useRef<string>("");
  const isPausedRef = useRef(false);

  const { initAudio, queueAudio, setAudioVolume, closeAudio } = useAudio();
  const { saveSram, loadSram, saveStateData, loadStateData } = useSaveData();

  // Load WASM module with retry logic
  const loadWasm = useCallback(async (): Promise<WasmModule> => {
    // Return cached module if available
    if (wasmModule) {
      return wasmModule;
    }

    // Return existing loading promise if in progress
    if (wasmLoadingPromise) {
      return wasmLoadingPromise;
    }

    // Create new loading promise with retry
    wasmLoadingPromise = (async (): Promise<WasmModule> => {
      const maxRetries = 3;
      let lastError: Error | null = null;

      for (let attempt = 1; attempt <= maxRetries; attempt++) {
        try {
          console.log(
            `Loading WASM module (attempt ${attempt}/${maxRetries})...`
          );

          // Dynamic import of the JS glue code - cast through unknown to avoid TS errors
          const wasm = (await import(
            "@/lib/wasm/gbemu_core.js"
          )) as unknown as WasmModule;

          // Initialize with the WASM file path from public folder
          await wasm.default({ module_or_path: "/wasm/gbemu_core_bg.wasm" });

          console.log("WASM module loaded successfully");
          wasmModule = wasm;
          return wasm;
        } catch (err) {
          lastError = err as Error;
          console.error(`WASM load attempt ${attempt} failed:`, err);

          if (attempt < maxRetries) {
            // Wait before retry (exponential backoff)
            await new Promise((resolve) => setTimeout(resolve, 500 * attempt));
          }
        }
      }

      wasmLoadingPromise = null;
      throw lastError || new Error("Failed to load WASM module");
    })();

    return wasmLoadingPromise;
  }, []);

  // Validate ROM data
  const validateRom = (
    data: Uint8Array
  ): { valid: boolean; error?: string } => {
    if (!data || data.length === 0) {
      return { valid: false, error: "ROM data is empty" };
    }

    // Minimum ROM size (32KB)
    if (data.length < 32768) {
      return { valid: false, error: "ROM file is too small (minimum 32KB)" };
    }

    // Maximum ROM size (8MB)
    if (data.length > 8388608) {
      return { valid: false, error: "ROM file is too large (maximum 8MB)" };
    }

    return { valid: true };
  };

  // Load ROM
  const loadRom = useCallback(
    async (data: Uint8Array, fileName: string) => {
      setError(null);
      setIsLoading(true);

      try {
        console.log(`Loading ROM: ${fileName} (${data.length} bytes)`);

        // Validate ROM
        const validation = validateRom(data);
        if (!validation.valid) {
          throw new Error(validation.error);
        }

        // Load WASM first
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

        // Close previous audio context
        closeAudio();

        // Create new emulator instance
        console.log("Creating emulator instance...");
        const emulator = new wasm.WasmGameBoy(data);
        emulatorRef.current = emulator;
        currentRomNameRef.current = fileName.replace(/\.[^/.]+$/, "");

        const title = emulator.game_title();
        console.log(`Game title: ${title}`);
        setGameTitle(title);

        // Try to load existing SRAM (with timeout to prevent blocking)
        try {
          console.log("Checking for existing save data...");
          const sramPromise = loadSram(currentRomNameRef.current);
          const timeoutPromise = new Promise<Uint8Array | null>((resolve) =>
            setTimeout(() => resolve(null), 3000)
          );
          const existingSram = await Promise.race([
            sramPromise,
            timeoutPromise,
          ]);
          if (existingSram) {
            emulator.load_sram(existingSram);
            console.log("Loaded existing save data");
          }
        } catch (e) {
          console.warn("Failed to load save data:", e);
        }

        // Initialize audio (non-blocking - don't let audio issues prevent ROM loading)
        const sampleRate = emulator.audio_sample_rate();
        console.log(`Initializing audio at ${sampleRate}Hz`);
        try {
          // Use timeout to prevent hanging on iOS
          const audioPromise = initAudio(sampleRate);
          const timeoutPromise = new Promise<void>((resolve) =>
            setTimeout(resolve, 2000)
          );
          await Promise.race([audioPromise, timeoutPromise]);
          setAudioVolume(volume);
          console.log("Audio setup complete");
        } catch (audioErr) {
          console.warn(
            "Audio initialization failed, continuing without audio:",
            audioErr
          );
        }

        setIsRunning(true);
        setIsPaused(false);
        isPausedRef.current = false;
        lastFrameTimeRef.current = 0;
        frameCountRef.current = 0;

        // Start FPS counter
        fpsIntervalRef.current = setInterval(() => {
          setFps(frameCountRef.current);
          frameCountRef.current = 0;
        }, 1000);

        // Start emulation loop
        startEmulationLoop();

        console.log("ROM loaded successfully");
      } catch (err) {
        const errorMessage =
          err instanceof Error ? err.message : "Failed to load ROM";
        console.error("ROM load error:", err);
        setError(errorMessage);
        setIsRunning(false);
        emulatorRef.current = null;
      } finally {
        setIsLoading(false);
      }
    },
    [loadWasm, initAudio, setAudioVolume, volume, closeAudio, loadSram]
  );

  // Emulation loop
  const startEmulationLoop = useCallback(() => {
    const targetFrameTime = 1000 / 59.7; // Game Boy runs at ~59.7 FPS
    let accumulator = 0;

    const loop = (timestamp: number) => {
      if (!emulatorRef.current) {
        return;
      }

      if (isPausedRef.current) {
        animationRef.current = requestAnimationFrame(loop);
        return;
      }

      if (lastFrameTimeRef.current === 0) {
        lastFrameTimeRef.current = timestamp;
      }

      const deltaTime = timestamp - lastFrameTimeRef.current;
      lastFrameTimeRef.current = timestamp;

      accumulator += deltaTime;

      // Run frames to catch up (cap at 3 frames to prevent spiral)
      let framesRun = 0;
      while (accumulator >= targetFrameTime && framesRun < 3) {
        try {
          emulatorRef.current.run_frame();
          accumulator -= targetFrameTime;
          frameCountRef.current++;
          framesRun++;

          // Process audio
          const audioBuffer = emulatorRef.current.get_audio_buffer();
          if (audioBuffer && audioBuffer.length > 0) {
            queueAudio(audioBuffer);
            emulatorRef.current.clear_audio_buffer();
          }
        } catch (err) {
          console.error("Emulation error:", err);
          setError("Emulation error occurred");
          setIsRunning(false);
          return;
        }
      }

      // Reset accumulator if we're too far behind
      if (accumulator > targetFrameTime * 3) {
        accumulator = 0;
      }

      // Update framebuffer
      try {
        const fb = emulatorRef.current.get_framebuffer();
        setFramebuffer(new Uint8Array(fb));
      } catch (err) {
        console.error("Framebuffer error:", err);
      }

      animationRef.current = requestAnimationFrame(loop);
    };

    animationRef.current = requestAnimationFrame(loop);
  }, [queueAudio]);

  // Pause emulation
  const pause = useCallback(() => {
    isPausedRef.current = true;
    setIsPaused(true);

    // Save SRAM when pausing
    if (emulatorRef.current && currentRomNameRef.current) {
      try {
        const sram = emulatorRef.current.save_sram();
        if (sram && sram.length > 0) {
          saveSram(currentRomNameRef.current, sram);
        }
      } catch (e) {
        console.warn("Failed to save SRAM on pause:", e);
      }
    }
  }, [saveSram]);

  // Resume emulation
  const resume = useCallback(() => {
    isPausedRef.current = false;
    setIsPaused(false);
    lastFrameTimeRef.current = 0;
  }, []);

  // Reset emulation
  const reset = useCallback(() => {
    if (emulatorRef.current) {
      emulatorRef.current.reset();
    }
  }, []);

  // Button handlers - use correct method names from WASM
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
      const state = emulatorRef.current.save_state();
      await saveStateData(currentRomNameRef.current, state);
      console.log("State saved");
    } catch (e) {
      console.error("Failed to save state:", e);
      setError("Failed to save state");
    }
  }, [saveStateData]);

  // Load state
  const loadState = useCallback(async () => {
    if (!emulatorRef.current || !currentRomNameRef.current) return;

    try {
      const state = await loadStateData(currentRomNameRef.current);
      if (state) {
        emulatorRef.current.load_state(state);
        console.log("State loaded");
      } else {
        console.log("No saved state found");
      }
    } catch (e) {
      console.error("Failed to load state:", e);
      setError("Failed to load state");
    }
  }, [loadStateData]);

  // Set volume
  const setVolume = useCallback(
    (newVolume: number) => {
      setVolumeState(newVolume);
      setAudioVolume(newVolume);
    },
    [setAudioVolume]
  );

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

      // Save SRAM on cleanup
      if (emulatorRef.current && currentRomNameRef.current) {
        try {
          const sram = emulatorRef.current.save_sram();
          if (sram && sram.length > 0) {
            saveSram(currentRomNameRef.current, sram);
          }
        } catch (e) {
          // Ignore errors during cleanup
        }
      }
    };
  }, [closeAudio, saveSram]);

  // Auto-save SRAM periodically
  useEffect(() => {
    if (!isRunning || isPaused) return;

    const autoSaveInterval = setInterval(() => {
      if (emulatorRef.current && currentRomNameRef.current) {
        try {
          const sram = emulatorRef.current.save_sram();
          if (sram && sram.length > 0) {
            saveSram(currentRomNameRef.current, sram);
          }
        } catch (e) {
          // Silently fail auto-save
        }
      }
    }, 30000); // Auto-save every 30 seconds

    return () => clearInterval(autoSaveInterval);
  }, [isRunning, isPaused, saveSram]);

  const value: EmulatorContextType = {
    isRunning,
    isPaused,
    gameTitle,
    error,
    framebuffer,
    volume,
    fps,
    isLoading,
    loadRom,
    pause,
    resume,
    reset,
    pressButton,
    releaseButton,
    saveState,
    loadState,
    setVolume,
  };

  return (
    <EmulatorContext.Provider value={value}>
      {children}
    </EmulatorContext.Provider>
  );
}
