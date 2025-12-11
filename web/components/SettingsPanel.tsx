"use client";

import { useEmulator } from "./EmulatorContext";

interface SettingsPanelProps {
  onClose: () => void;
}

export function SettingsPanel({ onClose }: SettingsPanelProps) {
  const { volume, setVolume, pause, resume, reset, isPaused } = useEmulator();

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center p-4">
      {/* Backdrop */}
      <div
        className="absolute inset-0 bg-black/60 backdrop-blur-sm"
        onClick={onClose}
      />

      {/* Panel */}
      <div className="relative card w-full max-w-sm animate-slide-up">
        {/* Header */}
        <div className="flex items-center justify-between mb-6">
          <h2 className="text-lg font-bold text-(--color-text-primary)">
            Settings
          </h2>
          <button
            onClick={onClose}
            className="p-2 rounded-lg text-(--color-text-muted) hover:text-(--color-text-primary) hover:bg-(--color-bg-tertiary) transition-colors"
          >
            <CloseIcon className="w-5 h-5" />
          </button>
        </div>

        {/* Volume */}
        <div className="mb-6">
          <label className="block text-sm font-medium text-(--color-text-secondary) mb-2">
            Volume: {Math.round(volume * 100)}%
          </label>
          <input
            type="range"
            min="0"
            max="1"
            step="0.05"
            value={volume}
            onChange={(e) => setVolume(parseFloat(e.target.value))}
            className="w-full h-2 rounded-full appearance-none cursor-pointer bg-(--color-bg-tertiary)"
            style={{
              background: `linear-gradient(to right, var(--color-accent) 0%, var(--color-accent) ${
                volume * 100
              }%, var(--color-bg-tertiary) ${
                volume * 100
              }%, var(--color-bg-tertiary) 100%)`,
            }}
          />
        </div>

        {/* Playback controls */}
        <div className="space-y-3">
          <button
            onClick={() => (isPaused ? resume() : pause())}
            className="w-full flex items-center gap-3 p-3 rounded-lg bg-(--color-bg-tertiary) hover:bg-(--color-accent)/20 transition-colors text-left"
          >
            {isPaused ? (
              <PlayIcon className="w-5 h-5 text-(--color-accent)" />
            ) : (
              <PauseIcon className="w-5 h-5 text-(--color-accent)" />
            )}
            <span className="text-(--color-text-primary) font-medium">
              {isPaused ? "Resume" : "Pause"}
            </span>
            <span className="ml-auto text-xs text-(--color-text-muted)">P</span>
          </button>

          <button
            onClick={() => {
              reset();
              onClose();
            }}
            className="w-full flex items-center gap-3 p-3 rounded-lg bg-(--color-bg-tertiary) hover:bg-(--color-accent)/20 transition-colors text-left"
          >
            <ResetIcon className="w-5 h-5 text-(--color-accent)" />
            <span className="text-(--color-text-primary) font-medium">
              Reset Game
            </span>
          </button>
        </div>

        {/* Keyboard shortcuts info */}
        <div className="mt-6 pt-4 border-t border-(--color-bg-tertiary)">
          <h3 className="text-sm font-medium text-(--color-text-secondary) mb-3">
            Keyboard Shortcuts
          </h3>
          <div className="grid grid-cols-2 gap-2 text-xs text-(--color-text-muted)">
            <div className="flex justify-between">
              <span>D-Pad</span>
              <span className="text-(--color-text-secondary)">
                Arrows / WASD
              </span>
            </div>
            <div className="flex justify-between">
              <span>A Button</span>
              <span className="text-(--color-text-secondary)">Z / K</span>
            </div>
            <div className="flex justify-between">
              <span>B Button</span>
              <span className="text-(--color-text-secondary)">X / J</span>
            </div>
            <div className="flex justify-between">
              <span>Start</span>
              <span className="text-(--color-text-secondary)">Enter</span>
            </div>
            <div className="flex justify-between">
              <span>Select</span>
              <span className="text-(--color-text-secondary)">Backspace</span>
            </div>
            <div className="flex justify-between">
              <span>Pause</span>
              <span className="text-(--color-text-secondary)">P / Esc</span>
            </div>
            <div className="flex justify-between">
              <span>Save State</span>
              <span className="text-(--color-text-secondary)">F5</span>
            </div>
            <div className="flex justify-between">
              <span>Load State</span>
              <span className="text-(--color-text-secondary)">F8</span>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

function CloseIcon({ className }: { className?: string }) {
  return (
    <svg
      className={className}
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="2"
    >
      <line x1="18" y1="6" x2="6" y2="18" />
      <line x1="6" y1="6" x2="18" y2="18" />
    </svg>
  );
}

function PlayIcon({ className }: { className?: string }) {
  return (
    <svg className={className} viewBox="0 0 24 24" fill="currentColor">
      <polygon points="5 3 19 12 5 21 5 3" />
    </svg>
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

function ResetIcon({ className }: { className?: string }) {
  return (
    <svg
      className={className}
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="2"
    >
      <polyline points="1 4 1 10 7 10" />
      <path d="M3.51 15a9 9 0 1 0 2.13-9.36L1 10" />
    </svg>
  );
}
