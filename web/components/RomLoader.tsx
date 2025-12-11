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

      // Validate file extension
      if (!file.name.match(/\.(gb|gbc|bin|rom)$/i)) {
        setError("Please select a valid Game Boy ROM file (.gb, .gbc)");
        return;
      }

      // Validate file size (max 8MB)
      if (file.size > 8 * 1024 * 1024) {
        setError("ROM file is too large (maximum 8MB)");
        return;
      }

      // Validate file size (min 32KB)
      if (file.size < 32 * 1024) {
        setError("ROM file is too small (minimum 32KB)");
        return;
      }

      setIsLoading(true);

      try {
        console.log(`Reading ROM file: ${file.name} (${file.size} bytes)`);

        const arrayBuffer = await file.arrayBuffer();
        const data = new Uint8Array(arrayBuffer);

        // Verify data was read correctly
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
        // Reset input so same file can be loaded again
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
    <div className="flex flex-col items-center gap-8 animate-fade-in">
      {/* Logo */}
      <div className="text-center">
        <h1 className="text-4xl md:text-6xl font-bold text-white tracking-wider">
          GB<span className="text-green-400">Emu</span>
        </h1>
        <p className="text-gray-400 mt-2">Game Boy & Game Boy Color Emulator</p>
      </div>

      {/* Error message */}
      {error && (
        <div className="w-full max-w-md p-4 rounded-lg bg-red-500/20 border border-red-500/50 text-red-300 text-center">
          {error}
        </div>
      )}

      {/* Drop zone */}
      <div
        className={`
          w-full max-w-md p-8 rounded-2xl border-2 border-dashed
          transition-all duration-200
          ${loading ? "cursor-wait" : "cursor-pointer"}
          ${
            isDragging
              ? "border-green-400 bg-green-400/10"
              : "border-gray-600 hover:border-gray-500 bg-white/5"
          }
        `}
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

        <div className="flex flex-col items-center gap-4 text-center">
          {loading ? (
            <>
              <LoadingSpinner />
              <p className="text-white">Loading ROM...</p>
              <p className="text-gray-500 text-sm">This may take a moment...</p>
            </>
          ) : (
            <>
              <CartridgeIcon />
              <div>
                <p className="text-white font-medium">
                  Drop a ROM here or click to browse
                </p>
                <p className="text-gray-500 text-sm mt-1">
                  Supports .gb and .gbc files
                </p>
              </div>
            </>
          )}
        </div>
      </div>

      {/* Features list */}
      <div className="grid grid-cols-2 md:grid-cols-4 gap-4 text-center text-sm">
        <Feature icon="🎮" text="Touch Controls" />
        <Feature icon="💾" text="Auto Save" />
        <Feature icon="🔊" text="Full Audio" />
        <Feature icon="📱" text="PWA Support" />
      </div>

      {/* Instructions */}
      <div className="text-gray-500 text-xs text-center max-w-md">
        <p>
          This emulator runs entirely in your browser. Your ROMs and save data
          never leave your device.
        </p>
      </div>
    </div>
  );
}

function Feature({ icon, text }: { icon: string; text: string }) {
  return (
    <div className="flex flex-col items-center gap-1 text-gray-400">
      <span className="text-2xl">{icon}</span>
      <span>{text}</span>
    </div>
  );
}

function CartridgeIcon() {
  return (
    <svg
      width="64"
      height="64"
      viewBox="0 0 64 64"
      fill="none"
      className="text-gray-400"
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
        opacity="0.3"
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
        opacity="0.5"
      />
    </svg>
  );
}

function LoadingSpinner() {
  return (
    <svg className="animate-spin h-12 w-12 text-green-400" viewBox="0 0 24 24">
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
