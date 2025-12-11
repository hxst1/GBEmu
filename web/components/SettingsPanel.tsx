'use client';

import { useEmulator } from './EmulatorContext';

interface SettingsPanelProps {
  onClose: () => void;
}

export function SettingsPanel({ onClose }: SettingsPanelProps) {
  const { volume, setVolume, isPaused, pause, resume, reset } = useEmulator();

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/70 backdrop-blur-sm">
      <div className="bg-gray-900 rounded-2xl p-6 w-full max-w-sm mx-4 animate-fade-in">
        <div className="flex justify-between items-center mb-6">
          <h2 className="text-white text-xl font-bold">Settings</h2>
          <button
            onClick={onClose}
            className="text-gray-400 hover:text-white transition-colors"
          >
            <CloseIcon />
          </button>
        </div>

        <div className="space-y-6">
          {/* Volume control */}
          <div>
            <label className="block text-gray-400 text-sm mb-2">
              Volume: {Math.round(volume * 100)}%
            </label>
            <input
              type="range"
              min="0"
              max="1"
              step="0.01"
              value={volume}
              onChange={(e) => setVolume(parseFloat(e.target.value))}
              className="w-full accent-green-400"
            />
          </div>

          {/* Pause/Resume */}
          <div>
            <button
              onClick={() => isPaused ? resume() : pause()}
              className="w-full py-3 px-4 bg-gray-800 hover:bg-gray-700 text-white rounded-lg transition-colors flex items-center justify-center gap-2"
            >
              {isPaused ? <PlayIcon /> : <PauseIcon />}
              {isPaused ? 'Resume' : 'Pause'}
            </button>
          </div>

          {/* Reset */}
          <div>
            <button
              onClick={() => {
                if (confirm('Are you sure you want to reset? Unsaved progress will be lost.')) {
                  reset();
                }
              }}
              className="w-full py-3 px-4 bg-red-900/50 hover:bg-red-900 text-red-300 rounded-lg transition-colors flex items-center justify-center gap-2"
            >
              <ResetIcon />
              Reset Game
            </button>
          </div>

          {/* Keyboard shortcuts */}
          <div className="mt-6 pt-6 border-t border-gray-800">
            <h3 className="text-gray-400 text-sm mb-3">Keyboard Shortcuts</h3>
            <div className="grid grid-cols-2 gap-2 text-xs">
              <ShortcutItem keys="Arrow Keys" action="D-Pad" />
              <ShortcutItem keys="Z" action="A Button" />
              <ShortcutItem keys="X" action="B Button" />
              <ShortcutItem keys="Enter" action="Start" />
              <ShortcutItem keys="Backspace" action="Select" />
              <ShortcutItem keys="P" action="Pause" />
              <ShortcutItem keys="F5" action="Save State" />
              <ShortcutItem keys="F8" action="Load State" />
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

function ShortcutItem({ keys, action }: { keys: string; action: string }) {
  return (
    <div className="flex justify-between text-gray-500">
      <span className="text-gray-300 font-mono bg-gray-800 px-2 py-0.5 rounded">{keys}</span>
      <span>{action}</span>
    </div>
  );
}

// Icons
function CloseIcon() {
  return (
    <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
      <line x1="18" y1="6" x2="6" y2="18" />
      <line x1="6" y1="6" x2="18" y2="18" />
    </svg>
  );
}

function PlayIcon() {
  return (
    <svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor">
      <polygon points="5 3 19 12 5 21 5 3" />
    </svg>
  );
}

function PauseIcon() {
  return (
    <svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor">
      <rect x="6" y="4" width="4" height="16" />
      <rect x="14" y="4" width="4" height="16" />
    </svg>
  );
}

function ResetIcon() {
  return (
    <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
      <path d="M1 4v6h6" />
      <path d="M3.51 15a9 9 0 1 0 2.13-9.36L1 10" />
    </svg>
  );
}
