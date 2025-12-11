"use client";

import { useCallback, useRef, useState } from "react";

interface RomLoaderProps {
  onRomLoad: (data: Uint8Array, fileName: string) => Promise<void>;
  isLoading?: boolean;
}

export function RomLoader({
  onRomLoad,
  isLoading: externalLoading = false,
}: RomLoaderProps) {
  const fileInputRef = useRef<HTMLInputElement>(null);
  const [isDragging, setIsDragging] = useState(false);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const loading = isLoading || externalLoading;

  const handleFile = useCallback(
    async (file: File) => {
      setError(null);

      if (!file.name.match(/\.(gb|gbc|bin|rom)$/i)) {
        setError("Please select a valid Game Boy ROM file (.gb, .gbc)");
        return;
      }

      if (file.size > 8 * 1024 * 1024) {
        setError("ROM file is too large (maximum 8MB)");
        return;
      }

      if (file.size < 32 * 1024) {
        setError("ROM file is too small (minimum 32KB)");
        return;
      }

      setIsLoading(true);

      try {
        console.log(`Reading ROM file: ${file.name} (${file.size} bytes)`);
        const arrayBuffer = await file.arrayBuffer();
        const data = new Uint8Array(arrayBuffer);

        if (data.length === 0) {
          throw new Error("Failed to read ROM file");
        }

        console.log(`ROM data loaded: ${data.length} bytes`);
        await onRomLoad(data, file.name);
      } catch (err) {
        const errorMessage =
          err instanceof Error ? err.message : "Failed to load ROM";
        console.error("Failed to load ROM:", err);
        setError(errorMessage);
      } finally {
        setIsLoading(false);
        if (fileInputRef.current) {
          fileInputRef.current.value = "";
        }
      }
    },
    [onRomLoad]
  );

  const handleFileSelect = useCallback(
    (e: React.ChangeEvent<HTMLInputElement>) => {
      const file = e.target.files?.[0];
      if (file) {
        handleFile(file);
      }
    },
    [handleFile]
  );

  const handleDrop = useCallback(
    (e: React.DragEvent) => {
      e.preventDefault();
      e.stopPropagation();
      setIsDragging(false);

      const file = e.dataTransfer.files[0];
      if (file) {
        handleFile(file);
      }
    },
    [handleFile]
  );

  const handleDragOver = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    setIsDragging(true);
  }, []);

  const handleDragLeave = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    setIsDragging(false);
  }, []);

  const handleClick = useCallback(() => {
    if (!loading) {
      fileInputRef.current?.click();
    }
  }, [loading]);

  return (
    <div className="flex flex-col items-center gap-8 animate-fade-in w-full max-w-lg px-4">
      {/* Logo */}
      <div className="text-center">
        <div className="flex items-center justify-center gap-3 mb-2">
          <GameBoyLogo className="w-12 h-12 md:w-14 md:h-14" />
        </div>
        <h1 className="text-4xl md:text-5xl font-bold tracking-tight">
          <span className="text-(--color-text-primary)">Gameboy</span>
          <span className="text-gradient">4me</span>
        </h1>
        <p className="text-(--color-text-muted) mt-2 text-sm md:text-base">
          Play Game Boy &amp; Game Boy Color games in your browser
        </p>
      </div>

      {/* Error message */}
      {error && (
        <div className="w-full p-4 rounded-xl bg-red-500/10 border border-red-500/30 text-red-600 dark:text-red-400 text-center text-sm">
          {error}
        </div>
      )}

      {/* Drop zone */}
      <div
        className={`drop-zone w-full ${isDragging ? "dragging" : ""} ${
          loading ? "cursor-wait opacity-70" : ""
        }`}
        onClick={handleClick}
        onDrop={handleDrop}
        onDragOver={handleDragOver}
        onDragLeave={handleDragLeave}
      >
        <input
          ref={fileInputRef}
          type="file"
          accept=".gb,.gbc,.bin,.rom"
          onChange={handleFileSelect}
          className="hidden"
          disabled={loading}
        />

        <div className="flex flex-col items-center gap-4 text-center py-4">
          {loading ? (
            <>
              <LoadingSpinner />
              <div>
                <p className="text-(--color-text-primary) font-medium">
                  Loading ROM...
                </p>
                <p className="text-(--color-text-muted) text-sm mt-1">
                  This may take a moment
                </p>
              </div>
            </>
          ) : (
            <>
              <CartridgeIcon />
              <div>
                <p className="text-(--color-text-primary) font-medium">
                  Drop a ROM here or tap to browse
                </p>
                <p className="text-(--color-text-muted) text-sm mt-1">
                  Supports .gb and .gbc files
                </p>
              </div>
            </>
          )}
        </div>
      </div>

      {/* Features list */}
      <div className="grid grid-cols-2 sm:grid-cols-4 gap-4 w-full">
        <Feature icon="🎮" title="Touch Controls" />
        <Feature icon="💾" title="Auto Save" />
        <Feature icon="🔊" title="Full Audio" />
        <Feature icon="📱" title="Works Offline" />
      </div>
    </div>
  );
}

function Feature({ icon, title }: { icon: string; title: string }) {
  return (
    <div className="flex flex-col items-center gap-2 p-3 rounded-xl bg-(--color-bg-secondary)">
      <span className="text-2xl">{icon}</span>
      <span className="text-xs text-(--color-text-secondary) font-medium">
        {title}
      </span>
    </div>
  );
}

function GameBoyLogo({ className }: { className?: string }) {
  return (
    <svg className={className} viewBox="0 0 64 64" fill="none">
      <rect
        x="12"
        y="4"
        width="40"
        height="56"
        rx="6"
        className="fill-(--color-accent)"
      />
      <rect
        x="16"
        y="10"
        width="32"
        height="24"
        rx="2"
        className="fill-(--color-bg-primary)"
      />
      <rect x="20" y="14" width="24" height="16" rx="1" fill="#c8b89a" />
      <circle cx="24" cy="46" r="6" className="fill-(--color-bg-primary)" />
      <circle cx="40" cy="42" r="4" className="fill-(--color-bg-primary)" />
      <circle cx="40" cy="50" r="4" className="fill-(--color-bg-primary)" />
    </svg>
  );
}

function CartridgeIcon() {
  return (
    <svg
      width="64"
      height="64"
      viewBox="0 0 64 64"
      fill="none"
      className="text-(--color-text-muted)"
    >
      <rect
        x="12"
        y="8"
        width="40"
        height="48"
        rx="4"
        stroke="currentColor"
        strokeWidth="2"
      />
      <rect
        x="18"
        y="14"
        width="28"
        height="20"
        rx="2"
        fill="currentColor"
        opacity="0.2"
      />
      <rect
        x="22"
        y="40"
        width="8"
        height="8"
        rx="1"
        stroke="currentColor"
        strokeWidth="1.5"
      />
      <rect
        x="34"
        y="40"
        width="8"
        height="8"
        rx="1"
        stroke="currentColor"
        strokeWidth="1.5"
      />
      <rect
        x="8"
        y="52"
        width="48"
        height="4"
        rx="2"
        fill="currentColor"
        opacity="0.3"
      />
    </svg>
  );
}

function LoadingSpinner() {
  return (
    <svg
      className="animate-spin h-12 w-12 text-(--color-accent)"
      viewBox="0 0 24 24"
    >
      <circle
        className="opacity-25"
        cx="12"
        cy="12"
        r="10"
        stroke="currentColor"
        strokeWidth="4"
        fill="none"
      />
      <path
        className="opacity-75"
        fill="currentColor"
        d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z"
      />
    </svg>
  );
}
