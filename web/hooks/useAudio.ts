"use client";

import { useRef, useCallback } from "react";

export function useAudio() {
  const audioContextRef = useRef<AudioContext | null>(null);
  const gainNodeRef = useRef<GainNode | null>(null);
  const nextPlayTimeRef = useRef<number>(0);
  const isInitializedRef = useRef(false);

  // Initialize audio context - NON-BLOCKING for iOS compatibility
  const initAudio = useCallback(async (sampleRate: number): Promise<void> => {
    // Close existing context if any
    if (audioContextRef.current) {
      try {
        await audioContextRef.current.close();
      } catch (e) {
        // Ignore close errors
      }
    }

    try {
      // Create AudioContext with desired sample rate
      const AudioContextClass =
        window.AudioContext ||
        (window as unknown as { webkitAudioContext: typeof AudioContext })
          .webkitAudioContext;

      const ctx = new AudioContextClass({ sampleRate });
      audioContextRef.current = ctx;

      // Create gain node for volume control
      const gainNode = ctx.createGain();
      gainNode.connect(ctx.destination);
      gainNode.gain.value = 0.5;
      gainNodeRef.current = gainNode;

      // Reset play time
      nextPlayTimeRef.current = 0;
      isInitializedRef.current = true;

      // Try to resume immediately (works if user already interacted)
      if (ctx.state === "suspended") {
        // Use Promise.race with timeout to avoid blocking on iOS
        const resumePromise = ctx.resume();
        const timeoutPromise = new Promise<void>((resolve) =>
          setTimeout(resolve, 500)
        );

        await Promise.race([resumePromise, timeoutPromise]);
      }

      // Check state after resume attempt
      if (ctx.state === "running") {
        console.log("Audio context resumed immediately");
      } else if (ctx.state === "suspended") {
        console.log("Audio context suspended, will resume on user interaction");
        // Set up listeners to resume on user interaction
        setupResumeListeners(ctx);
      } else {
        console.log("Audio context already running");
      }

      console.log(
        `Audio initialized: sampleRate=${ctx.sampleRate}, state=${ctx.state}`
      );
    } catch (err) {
      console.error("Failed to initialize audio:", err);
      // Don't throw - allow emulator to run without audio
      isInitializedRef.current = false;
    }
  }, []);

  // Setup listeners to resume audio on user interaction (for iOS)
  const setupResumeListeners = (ctx: AudioContext) => {
    const resumeAudio = () => {
      if (ctx.state === "suspended") {
        ctx
          .resume()
          .then(() => {
            console.log("Audio resumed after user interaction");
          })
          .catch(() => {
            // Ignore resume errors
          });
      }
    };

    // Multiple event types for better iOS coverage
    const events = ["touchstart", "touchend", "mousedown", "keydown", "click"];

    const handler = () => {
      resumeAudio();
      // Remove listeners after first successful resume
      if (ctx.state === "running") {
        events.forEach((event) => {
          document.removeEventListener(event, handler, true);
        });
      }
    };

    events.forEach((event) => {
      document.addEventListener(event, handler, {
        capture: true,
        passive: true,
      });
    });
  };

  // Queue audio samples for playback
  const queueAudio = useCallback((samples: Float32Array) => {
    const ctx = audioContextRef.current;
    const gainNode = gainNodeRef.current;

    if (!ctx || !gainNode || !isInitializedRef.current) {
      return;
    }

    // Don't queue if context is suspended (iOS waiting for interaction)
    if (ctx.state !== "running") {
      return;
    }

    try {
      // Samples are interleaved stereo (L, R, L, R, ...)
      const numFrames = samples.length / 2;

      if (numFrames === 0) {
        return;
      }

      // Create audio buffer
      const buffer = ctx.createBuffer(2, numFrames, ctx.sampleRate);
      const leftChannel = buffer.getChannelData(0);
      const rightChannel = buffer.getChannelData(1);

      // De-interleave samples
      for (let i = 0; i < numFrames; i++) {
        leftChannel[i] = samples[i * 2];
        rightChannel[i] = samples[i * 2 + 1];
      }

      // Create buffer source
      const source = ctx.createBufferSource();
      source.buffer = buffer;
      source.connect(gainNode);

      // Calculate when to play
      const currentTime = ctx.currentTime;

      // Initialize or reset play time if too far behind/ahead
      if (
        nextPlayTimeRef.current === 0 ||
        nextPlayTimeRef.current < currentTime - 0.1 ||
        nextPlayTimeRef.current > currentTime + 0.3
      ) {
        // Start slightly ahead to avoid underruns
        nextPlayTimeRef.current = currentTime + 0.02;
      }

      // Schedule playback
      source.start(nextPlayTimeRef.current);

      // Update next play time
      nextPlayTimeRef.current += buffer.duration;
    } catch (err) {
      // Silently ignore audio errors to not spam console
      // Reset play time on error
      nextPlayTimeRef.current = 0;
    }
  }, []);

  // Set volume (0.0 to 1.0)
  const setAudioVolume = useCallback((volume: number) => {
    if (gainNodeRef.current) {
      gainNodeRef.current.gain.value = Math.max(0, Math.min(1, volume));
    }
  }, []);

  // Close audio context
  const closeAudio = useCallback(() => {
    if (audioContextRef.current) {
      try {
        audioContextRef.current.close();
      } catch (e) {
        // Ignore close errors
      }
      audioContextRef.current = null;
      gainNodeRef.current = null;
      nextPlayTimeRef.current = 0;
      isInitializedRef.current = false;
    }
  }, []);

  return {
    initAudio,
    queueAudio,
    setAudioVolume,
    closeAudio,
  };
}
