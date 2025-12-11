"use client";

import { useState, useCallback } from "react";
import { EmulatorProvider, useEmulator } from "@/components/EmulatorContext";
import { Screen } from "@/components/Screen";
import { Controls } from "@/components/Controls";
import { RomLoader } from "@/components/RomLoader";
import { SettingsPanel } from "@/components/SettingsPanel";
import { SaveManager } from "@/components/SaveManager";

function EmulatorUI() {
  const { isRunning, gameTitle, loadRom, error, isLoading } = useEmulator();
  const [showSettings, setShowSettings] = useState(false);
  const [showSaves, setShowSaves] = useState(false);

  const handleRomLoad = useCallback(
    async (romData: Uint8Array, fileName: string) => {
      await loadRom(romData, fileName);
    },
    [loadRom]
  );

  return (
    <div className="min-h-screen flex flex-col items-center justify-center p-4">
      {/* Header */}
      <header className="fixed top-0 left-0 right-0 z-50 flex items-center justify-between px-4 py-2 bg-black/30 backdrop-blur-sm">
        <h1 className="text-white font-bold text-lg tracking-wider">
          GB<span className="text-green-400">Emu</span>
        </h1>

        {gameTitle && (
          <span className="text-white/70 text-sm truncate max-w-[150px]">
            {gameTitle}
          </span>
        )}

        <div className="flex gap-2">
          {isRunning && (
            <>
              <button
                onClick={() => setShowSaves(true)}
                className="p-2 text-white/70 hover:text-white transition-colors"
                title="Save Manager"
              >
                <SaveIcon />
              </button>
              <button
                onClick={() => setShowSettings(true)}
                className="p-2 text-white/70 hover:text-white transition-colors"
                title="Settings"
              >
                <SettingsIcon />
              </button>
            </>
          )}
        </div>
      </header>

      {/* Error display */}
      {error && (
        <div className="fixed top-16 left-4 right-4 z-40 bg-red-500/90 text-white px-4 py-2 rounded-lg animate-fade-in">
          {error}
        </div>
      )}

      {/* Main content */}
      <main className="flex flex-col items-center gap-6 pt-16 pb-8">
        {!isRunning ? (
          <RomLoader onRomLoad={handleRomLoad} isLoading={isLoading} />
        ) : (
          <>
            <Screen />
            <Controls />
          </>
        )}
      </main>

      {/* Modals */}
      {showSettings && <SettingsPanel onClose={() => setShowSettings(false)} />}

      {showSaves && <SaveManager onClose={() => setShowSaves(false)} />}
    </div>
  );
}

export default function Home() {
  return (
    <EmulatorProvider>
      <EmulatorUI />
    </EmulatorProvider>
  );
}

// Icons
function SettingsIcon() {
  return (
    <svg
      width="24"
      height="24"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="2"
    >
      <circle cx="12" cy="12" r="3" />
      <path d="M12 1v2M12 21v2M4.22 4.22l1.42 1.42M18.36 18.36l1.42 1.42M1 12h2M21 12h2M4.22 19.78l1.42-1.42M18.36 5.64l1.42-1.42" />
    </svg>
  );
}

function SaveIcon() {
  return (
    <svg
      width="24"
      height="24"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="2"
    >
      <path d="M19 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11l5 5v11a2 2 0 0 1-2 2z" />
      <polyline points="17 21 17 13 7 13 7 21" />
      <polyline points="7 3 7 8 15 8" />
    </svg>
  );
}
