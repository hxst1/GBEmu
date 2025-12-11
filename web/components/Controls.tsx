"use client";

import { useEffect, useCallback, useRef } from "react";
import { useEmulator, BUTTONS, ButtonCode } from "./EmulatorContext";

// Keyboard mapping using e.code
const KEY_MAP: Record<string, ButtonCode> = {
  ArrowUp: BUTTONS.UP,
  ArrowDown: BUTTONS.DOWN,
  ArrowLeft: BUTTONS.LEFT,
  ArrowRight: BUTTONS.RIGHT,
  KeyZ: BUTTONS.A,
  KeyX: BUTTONS.B,
  Enter: BUTTONS.START,
  ShiftRight: BUTTONS.SELECT,
  Backspace: BUTTONS.SELECT,
  // Alternative WASD
  KeyW: BUTTONS.UP,
  KeyS: BUTTONS.DOWN,
  KeyA: BUTTONS.LEFT,
  KeyD: BUTTONS.RIGHT,
  // Alternative action keys
  KeyK: BUTTONS.A,
  KeyJ: BUTTONS.B,
};

export function Controls() {
  const {
    pressButton,
    releaseButton,
    isPaused,
    pause,
    resume,
    saveState,
    loadState,
  } = useEmulator();
  const pressedKeysRef = useRef<Set<string>>(new Set());

  // Keyboard handlers
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      // Try e.code first, then e.key for arrows
      const code = e.code;
      const key = e.key;

      // Check if it's a mapped key
      if (KEY_MAP[code]) {
        e.preventDefault();
        const button = KEY_MAP[code];
        if (!pressedKeysRef.current.has(code)) {
          pressedKeysRef.current.add(code);
          pressButton(button);
        }
        return;
      }

      // Fallback for arrow keys using e.key
      if (
        key === "ArrowUp" ||
        key === "ArrowDown" ||
        key === "ArrowLeft" ||
        key === "ArrowRight"
      ) {
        e.preventDefault();
        const buttonMap: Record<string, ButtonCode> = {
          ArrowUp: BUTTONS.UP,
          ArrowDown: BUTTONS.DOWN,
          ArrowLeft: BUTTONS.LEFT,
          ArrowRight: BUTTONS.RIGHT,
        };
        const button = buttonMap[key];
        if (!pressedKeysRef.current.has(key)) {
          pressedKeysRef.current.add(key);
          pressButton(button);
        }
        return;
      }

      // Pause/Resume with P or Escape
      if (code === "KeyP" || code === "Escape") {
        e.preventDefault();
        if (isPaused) {
          resume();
        } else {
          pause();
        }
      }

      // Quick save/load
      if (code === "F5") {
        e.preventDefault();
        saveState();
      }
      if (code === "F8") {
        e.preventDefault();
        loadState();
      }
    };

    const handleKeyUp = (e: KeyboardEvent) => {
      const code = e.code;
      const key = e.key;

      if (KEY_MAP[code]) {
        e.preventDefault();
        const button = KEY_MAP[code];
        pressedKeysRef.current.delete(code);
        releaseButton(button);
        return;
      }

      // Fallback for arrow keys
      if (
        key === "ArrowUp" ||
        key === "ArrowDown" ||
        key === "ArrowLeft" ||
        key === "ArrowRight"
      ) {
        e.preventDefault();
        const buttonMap: Record<string, ButtonCode> = {
          ArrowUp: BUTTONS.UP,
          ArrowDown: BUTTONS.DOWN,
          ArrowLeft: BUTTONS.LEFT,
          ArrowRight: BUTTONS.RIGHT,
        };
        const button = buttonMap[key];
        pressedKeysRef.current.delete(key);
        releaseButton(button);
      }
    };

    window.addEventListener("keydown", handleKeyDown);
    window.addEventListener("keyup", handleKeyUp);

    return () => {
      window.removeEventListener("keydown", handleKeyDown);
      window.removeEventListener("keyup", handleKeyUp);
    };
  }, [
    pressButton,
    releaseButton,
    isPaused,
    pause,
    resume,
    saveState,
    loadState,
  ]);

  // Button component with proper touch handling
  const GameButton = useCallback(
    ({
      button,
      className,
      children,
    }: {
      button: ButtonCode;
      className: string;
      children: React.ReactNode;
    }) => {
      const buttonRef = useRef<HTMLButtonElement>(null);
      const isPressed = useRef(false);

      useEffect(() => {
        const el = buttonRef.current;
        if (!el) return;

        const handleStart = (e: Event) => {
          e.preventDefault();
          if (!isPressed.current) {
            isPressed.current = true;
            pressButton(button);
          }
        };

        const handleEnd = (e: Event) => {
          e.preventDefault();
          if (isPressed.current) {
            isPressed.current = false;
            releaseButton(button);
          }
        };

        // Add non-passive touch listeners
        el.addEventListener("touchstart", handleStart, { passive: false });
        el.addEventListener("touchend", handleEnd, { passive: false });
        el.addEventListener("touchcancel", handleEnd, { passive: false });

        // Mouse listeners
        el.addEventListener("mousedown", handleStart);
        el.addEventListener("mouseup", handleEnd);
        el.addEventListener("mouseleave", handleEnd);

        return () => {
          el.removeEventListener("touchstart", handleStart);
          el.removeEventListener("touchend", handleEnd);
          el.removeEventListener("touchcancel", handleEnd);
          el.removeEventListener("mousedown", handleStart);
          el.removeEventListener("mouseup", handleEnd);
          el.removeEventListener("mouseleave", handleEnd);
        };
      }, [button]);

      return (
        <button ref={buttonRef} className={className}>
          {children}
        </button>
      );
    },
    [pressButton, releaseButton]
  );

  return (
    <div className="w-full max-w-md px-4 animate-fade-in select-none">
      {/* Main controls container */}
      <div className="flex justify-between items-center mt-4">
        {/* D-Pad */}
        <div className="dpad-container">
          <div className="dpad-center" />

          <GameButton button={BUTTONS.UP} className="dpad-btn dpad-up">
            <ChevronUp />
          </GameButton>

          <GameButton button={BUTTONS.DOWN} className="dpad-btn dpad-down">
            <ChevronDown />
          </GameButton>

          <GameButton button={BUTTONS.LEFT} className="dpad-btn dpad-left">
            <ChevronLeft />
          </GameButton>

          <GameButton button={BUTTONS.RIGHT} className="dpad-btn dpad-right">
            <ChevronRight />
          </GameButton>
        </div>

        {/* A/B Buttons */}
        <div className="flex gap-4 -rotate-12">
          <div className="flex flex-col items-center gap-1">
            <GameButton button={BUTTONS.B} className="action-btn">
              B
            </GameButton>
          </div>

          <div className="flex flex-col items-center gap-1 -mt-4">
            <GameButton button={BUTTONS.A} className="action-btn">
              A
            </GameButton>
          </div>
        </div>
      </div>

      {/* Start/Select buttons */}
      <div className="flex justify-center gap-8 mt-8 -rotate-25">
        <div className="flex flex-col items-center gap-1">
          <GameButton button={BUTTONS.SELECT} className="menu-btn">
            Select
          </GameButton>
        </div>

        <div className="flex flex-col items-center gap-1">
          <GameButton button={BUTTONS.START} className="menu-btn">
            Start
          </GameButton>
        </div>
      </div>

      {/* Keyboard hints (hidden on mobile) */}
      <div className="hidden md:block mt-8 text-center text-gray-500 text-xs">
        <p>Arrow Keys / WASD: D-Pad | Z/K: A | X/J: B</p>
        <p>Enter: Start | Backspace: Select | P: Pause | F5: Save | F8: Load</p>
      </div>
    </div>
  );
}

// Chevron icons
function ChevronUp() {
  return (
    <svg
      width="16"
      height="16"
      viewBox="0 0 24 24"
      fill="none"
      stroke="#666"
      strokeWidth="3"
    >
      <polyline points="18 15 12 9 6 15" />
    </svg>
  );
}

function ChevronDown() {
  return (
    <svg
      width="16"
      height="16"
      viewBox="0 0 24 24"
      fill="none"
      stroke="#666"
      strokeWidth="3"
    >
      <polyline points="6 9 12 15 18 9" />
    </svg>
  );
}

function ChevronLeft() {
  return (
    <svg
      width="16"
      height="16"
      viewBox="0 0 24 24"
      fill="none"
      stroke="#666"
      strokeWidth="3"
    >
      <polyline points="15 18 9 12 15 6" />
    </svg>
  );
}

function ChevronRight() {
  return (
    <svg
      width="16"
      height="16"
      viewBox="0 0 24 24"
      fill="none"
      stroke="#666"
      strokeWidth="3"
    >
      <polyline points="9 18 15 12 9 6" />
    </svg>
  );
}
