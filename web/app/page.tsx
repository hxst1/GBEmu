"use client";

import { useState, useCallback } from "react";
import { EmulatorProvider, useEmulator } from "@/components/EmulatorContext";
import { RomLoader } from "@/components/RomLoader";
import { SettingsPanel } from "@/components/SettingsPanel";
import { SaveManager } from "@/components/SaveManager";
import { Header } from "@/components/Header";
import { GameBoyShell } from "@/components/GameBoyShell";

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
    <div className="min-h-dvh flex flex-col">
      <Header
        gameTitle={gameTitle}
        isRunning={isRunning}
        onSettingsClick={() => setShowSettings(true)}
        onSavesClick={() => setShowSaves(true)}
      />

      {/* Error display */}
      {error && (
        <div className="fixed top-20 left-4 right-4 z-40 bg-red-500/90 text-white px-4 py-3 rounded-xl animate-fade-in shadow-lg max-w-md mx-auto">
          <div className="flex items-center gap-2">
            <svg
              className="w-5 h-5 shrink-0"
              fill="none"
              viewBox="0 0 24 24"
              stroke="currentColor"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
              />
            </svg>
            <p className="text-sm">{error}</p>
          </div>
        </div>
      )}

      {/* Main content */}
      <main className="flex-1 flex flex-col items-center justify-center px-4 py-6 md:py-8">
        {!isRunning ? (
          <RomLoader onRomLoad={handleRomLoad} isLoading={isLoading} />
        ) : (
          <GameBoyShell />
        )}
      </main>

      {/* Footer */}
      {!isRunning && (
        <footer className="py-4 text-center">
          <p className="text-(--color-text-muted) text-xs">
            Your games run entirely in your browser. Nothing is uploaded.
          </p>
        </footer>
      )}

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
