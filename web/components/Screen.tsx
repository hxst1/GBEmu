"use client";

import { useRef, useEffect } from "react";
import { useEmulator } from "./EmulatorContext";

const SCREEN_WIDTH = 160;
const SCREEN_HEIGHT = 144;

export function Screen() {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const { framebuffer, isPaused, isRunning } = useEmulator();

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;

    const ctx = canvas.getContext("2d", { alpha: false });
    if (!ctx) return;

    if (!framebuffer || framebuffer.length === 0) {
      // Draw idle screen
      ctx.fillStyle = "#c8b89a";
      ctx.fillRect(0, 0, SCREEN_WIDTH, SCREEN_HEIGHT);
      return;
    }

    const imageData = ctx.createImageData(SCREEN_WIDTH, SCREEN_HEIGHT);
    imageData.data.set(framebuffer);
    ctx.putImageData(imageData, 0, 0);
  }, [framebuffer]);

  return (
    <div className="relative">
      <canvas
        ref={canvasRef}
        width={SCREEN_WIDTH}
        height={SCREEN_HEIGHT}
        className="game-screen w-full aspect-160/144 bg-[#c8b89a]"
      />

      {/* Pause overlay */}
      {isPaused && isRunning && (
        <div className="absolute inset-0 flex items-center justify-center bg-black/50 rounded">
          <div className="flex flex-col items-center gap-2">
            <PauseIcon className="w-10 h-10 text-white" />
            <span className="text-white text-sm font-medium">Paused</span>
          </div>
        </div>
      )}

      {/* Screen reflection effect */}
      <div className="absolute inset-0 pointer-events-none rounded bg-linear-to-br from-white/5 to-transparent" />
    </div>
  );
}

function PauseIcon({ className }: { className?: string }) {
  return (
    <svg className={className} viewBox="0 0 24 24" fill="currentColor">
      <rect x="6" y="4" width="4" height="16" rx="1" />
      <rect x="14" y="4" width="4" height="16" rx="1" />
    </svg>
  );
}
