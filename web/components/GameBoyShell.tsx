"use client";

import { Screen } from "./Screen";
import { Controls } from "./Controls";
import { useEmulator } from "./EmulatorContext";

export function GameBoyShell() {
  const { gameTitle, fps, isPaused } = useEmulator();

  return (
    <div className="gameboy-shell w-full max-w-sm md:max-w-md animate-slide-up">
      {/* Top speaker grille */}
      <div className="flex justify-end mb-3 pr-2">
        <div className="flex gap-1">
          {[...Array(6)].map((_, i) => (
            <div
              key={i}
              className="w-1 h-4 rounded-full bg-(--color-shell-dark) opacity-60"
            />
          ))}
        </div>
      </div>

      {/* Screen section */}
      <div className="screen-bezel">
        {/* Power LED and label */}
        <div className="flex items-center gap-3 mb-2">
          <div className="flex items-center gap-1.5">
            <div
              className={`w-2 h-2 rounded-full transition-colors ${
                isPaused ? "bg-yellow-500 animate-pulse-subtle" : "bg-green-500"
              }`}
            />
            <span className="text-[0.5rem] text-gray-400 font-semibold uppercase tracking-wider">
              Power
            </span>
          </div>
          <div className="flex-1" />
          <span className="text-[0.5rem] text-gray-500 uppercase tracking-wider">
            {fps > 0 ? `${fps} FPS` : ""}
          </span>
        </div>

        {/* Game Screen */}
        <Screen />

        {/* Brand label */}
        <div className="mt-3 text-center">
          <span className="text-[0.6rem] text-gray-400 font-bold uppercase tracking-[0.2em]">
            Gameboy4me
          </span>
        </div>
      </div>

      {/* Game title display */}
      {gameTitle && (
        <div className="mt-3 text-center">
          <span className="text-xs text-(--color-text-muted) truncate block max-w-[200px] mx-auto">
            {gameTitle}
          </span>
        </div>
      )}

      {/* Controls section */}
      <Controls />

      {/* Bottom decorations */}
      <div className="flex justify-between items-end mt-4 px-4">
        {/* Headphone jack */}
        <div className="flex items-center gap-1">
          <div className="w-3 h-3 rounded-full bg-(--color-shell-dark) border-2 border-(--color-shell-secondary)" />
          <span className="text-[0.45rem] text-(--color-text-muted) uppercase">
            Phones
          </span>
        </div>

        {/* Volume indicator */}
        <div className="flex items-center gap-1">
          <span className="text-[0.45rem] text-(--color-text-muted) uppercase">
            Vol
          </span>
          <div className="w-8 h-1.5 rounded-full bg-(--color-shell-dark)">
            <div className="w-5 h-full rounded-full bg-(--color-shell-accent)" />
          </div>
        </div>
      </div>
    </div>
  );
}
