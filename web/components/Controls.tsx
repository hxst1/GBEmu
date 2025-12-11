"use client";

import { useEffect, useCallback, useRef } from "react";
import { useEmulator, BUTTONS, ButtonCode } from "./EmulatorContext";

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
  KeyW: BUTTONS.UP,
  KeyS: BUTTONS.DOWN,
  KeyA: BUTTONS.LEFT,
  KeyD: BUTTONS.RIGHT,
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

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      const code = e.code;
      const key = e.key;

      if (KEY_MAP[code]) {
        e.preventDefault();
        if (!pressedKeysRef.current.has(code)) {
          pressedKeysRef.current.add(code);
          pressButton(KEY_MAP[code]);
        }
        return;
      }

      if (["ArrowUp", "ArrowDown", "ArrowLeft", "ArrowRight"].includes(key)) {
        e.preventDefault();
        const buttonMap: Record<string, ButtonCode> = {
          ArrowUp: BUTTONS.UP,
          ArrowDown: BUTTONS.DOWN,
          ArrowLeft: BUTTONS.LEFT,
          ArrowRight: BUTTONS.RIGHT,
        };
        if (!pressedKeysRef.current.has(key)) {
          pressedKeysRef.current.add(key);
          pressButton(buttonMap[key]);
        }
        return;
      }

      if (code === "KeyP" || code === "Escape") {
        e.preventDefault();
        isPaused ? resume() : pause();
      }

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
        pressedKeysRef.current.delete(code);
        releaseButton(KEY_MAP[code]);
        return;
      }

      if (["ArrowUp", "ArrowDown", "ArrowLeft", "ArrowRight"].includes(key)) {
        e.preventDefault();
        const buttonMap: Record<string, ButtonCode> = {
          ArrowUp: BUTTONS.UP,
          ArrowDown: BUTTONS.DOWN,
          ArrowLeft: BUTTONS.LEFT,
          ArrowRight: BUTTONS.RIGHT,
        };
        pressedKeysRef.current.delete(key);
        releaseButton(buttonMap[key]);
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

        el.addEventListener("touchstart", handleStart, { passive: false });
        el.addEventListener("touchend", handleEnd, { passive: false });
        el.addEventListener("touchcancel", handleEnd, { passive: false });
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
    <div className="w-full mt-6 select-none">
      {/* Main controls */}
      <div className="flex justify-between items-center px-2">
        {/* D-Pad */}
        <div className="dpad-container">
          <div className="dpad-center" />

          <GameButton button={BUTTONS.UP} className="dpad-btn dpad-up">
            <ChevronIcon direction="up" />
          </GameButton>

          <GameButton button={BUTTONS.DOWN} className="dpad-btn dpad-down">
            <ChevronIcon direction="down" />
          </GameButton>

          <GameButton button={BUTTONS.LEFT} className="dpad-btn dpad-left">
            <ChevronIcon direction="left" />
          </GameButton>

          <GameButton button={BUTTONS.RIGHT} className="dpad-btn dpad-right">
            <ChevronIcon direction="right" />
          </GameButton>
        </div>

        {/* A/B Buttons */}
        <div className="flex gap-3 -rotate-20">
          <div className="flex flex-col items-center">
            <GameButton button={BUTTONS.B} className="action-btn">
              B
            </GameButton>
          </div>

          <div className="flex flex-col items-center -mt-6">
            <GameButton button={BUTTONS.A} className="action-btn">
              A
            </GameButton>
          </div>
        </div>
      </div>

      {/* Start/Select */}
      <div className="flex justify-center gap-6 mt-6">
        <GameButton button={BUTTONS.SELECT} className="menu-btn">
          Select
        </GameButton>

        <GameButton button={BUTTONS.START} className="menu-btn">
          Start
        </GameButton>
      </div>

      {/* Keyboard hints */}
      <div className="hidden md:block mt-6 text-center text-(--color-text-muted) text-xs">
        <p>
          <span className="font-medium">Arrows/WASD:</span> D-Pad |{" "}
          <span className="font-medium">Z/K:</span> A |{" "}
          <span className="font-medium">X/J:</span> B |{" "}
          <span className="font-medium">Enter:</span> Start |{" "}
          <span className="font-medium">P:</span> Pause
        </p>
      </div>
    </div>
  );
}

function ChevronIcon({
  direction,
}: {
  direction: "up" | "down" | "left" | "right";
}) {
  const paths = {
    up: "M18 15l-6-6-6 6",
    down: "M6 9l6 6 6-6",
    left: "M15 18l-6-6 6-6",
    right: "M9 18l6-6-6-6",
  };

  return (
    <svg
      width="14"
      height="14"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="3"
      strokeLinecap="round"
      strokeLinejoin="round"
      className="text-gray-400"
    >
      <polyline points={paths[direction]} />
    </svg>
  );
}
