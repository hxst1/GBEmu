"use client";

import { useRef, useEffect, useState } from "react";
import { useEmulator } from "./EmulatorContext";

const SCREEN_WIDTH = 160;
const SCREEN_HEIGHT = 144;
const SCALE = 2;

export function Screen() {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const { framebuffer, fps, isPaused } = useEmulator();
  const [scale, setScale] = useState(SCALE);

  // Calculate optimal scale based on viewport
  useEffect(() => {
    const calculateScale = () => {
      const maxWidth = window.innerWidth - 32;
      const maxHeight = window.innerHeight * 0.45;

      const scaleX = Math.floor(maxWidth / SCREEN_WIDTH);
      const scaleY = Math.floor(maxHeight / SCREEN_HEIGHT);

      setScale(Math.max(1, Math.min(scaleX, scaleY, 4)));
    };

    calculateScale();
    window.addEventListener("resize", calculateScale);
    return () => window.removeEventListener("resize", calculateScale);
  }, []);

  // Render framebuffer to canvas
  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas || !framebuffer) return;

    const ctx = canvas.getContext("2d");
    if (!ctx) return;

    // Create ImageData from framebuffer
    // We need to create a new Uint8ClampedArray from a copy of the data
    // to ensure it's backed by a regular ArrayBuffer (not SharedArrayBuffer)
    const imageData = new ImageData(
      new Uint8ClampedArray(framebuffer),
      SCREEN_WIDTH,
      SCREEN_HEIGHT
    );

    // Disable image smoothing for crisp pixels
    ctx.imageSmoothingEnabled = false;

    // Draw the image
    ctx.putImageData(imageData, 0, 0);
  }, [framebuffer]);

  return (
    <div className="gb-screen-container animate-fade-in">
      {/* Screen bezel */}
      <div className="relative bg-[#0f380f] rounded p-1">
        {/* Power LED */}
        <div className="absolute -top-6 left-4 flex items-center gap-2">
          <div
            className={`w-2 h-2 rounded-full ${
              isPaused ? "bg-yellow-400" : "bg-red-500"
            } shadow-lg`}
            style={{
              boxShadow: isPaused ? "0 0 8px #facc15" : "0 0 8px #ef4444",
            }}
          />
          <span className="text-[10px] text-gray-400 uppercase tracking-wider">
            {isPaused ? "Paused" : "Power"}
          </span>
        </div>

        {/* FPS counter */}
        <div className="absolute -top-6 right-4">
          <span className="text-[10px] text-gray-400 font-mono">{fps} FPS</span>
        </div>

        {/* Canvas */}
        <canvas
          ref={canvasRef}
          width={SCREEN_WIDTH}
          height={SCREEN_HEIGHT}
          className="block"
          style={{
            width: SCREEN_WIDTH * scale,
            height: SCREEN_HEIGHT * scale,
            imageRendering: "pixelated",
          }}
        />

        {/* Screen label */}
        <div className="mt-2 flex justify-center">
          <span className="text-[10px] text-gray-600 tracking-[0.3em] uppercase">
            Dot Matrix with Stereo Sound
          </span>
        </div>
      </div>

      {/* Pause overlay */}
      {isPaused && (
        <div className="absolute inset-0 flex items-center justify-center bg-black/50 rounded-lg">
          <span className="text-white text-2xl font-bold tracking-wider animate-pulse">
            PAUSED
          </span>
        </div>
      )}
    </div>
  );
}
