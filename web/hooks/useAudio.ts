"use client";

import { useCallback, useRef } from "react";

export function useAudio() {
  const audioContextRef = useRef<AudioContext | null>(null);
  const gainNodeRef = useRef<GainNode | null>(null);
  const nextPlayTimeRef = useRef<number>(0);
  const isInitializedRef = useRef(false);

  const initAudio = useCallback(async (sampleRate: number) => {
    // Close existing context
    if (audioContextRef.current) {
      try {
        await audioContextRef.current.close();
      } catch (e) {
        console.warn("Error closing previous audio context:", e);
      }
    }

    try {
      const AudioContextClass =
        window.AudioContext ||
        (window as unknown as { webkitAudioContext: typeof AudioContext })
          .webkitAudioContext;
      const ctx = new AudioContextClass({ sampleRate });

      const gainNode = ctx.createGain();
      gainNode.connect(ctx.destination);
      gainNode.gain.value = 0.5;

      audioContextRef.current = ctx;
      gainNodeRef.current = gainNode;
      nextPlayTimeRef.current = ctx.currentTime;
      isInitializedRef.current = true;

      // Try to resume immediately
      if (ctx.state === "suspended") {
        try {
          await ctx.resume();
          console.log("Audio context resumed immediately");
        } catch (e) {
          console.log("Audio context suspended, waiting for user interaction");
        }
      }

      // Setup listeners for user interaction to resume audio
      const resume = async () => {
        if (
          audioContextRef.current &&
          audioContextRef.current.state === "suspended"
        ) {
          try {
            await audioContextRef.current.resume();
            console.log("Audio context resumed on user interaction");
          } catch (e) {
            console.warn("Failed to resume audio context:", e);
          }
        }
      };

      // Add listeners to various events
      document.addEventListener("click", resume, { once: false });
      document.addEventListener("touchstart", resume, { once: false });
      document.addEventListener("touchend", resume, { once: false });
      document.addEventListener("keydown", resume, { once: false });

      console.log(
        `Audio initialized: sampleRate=${sampleRate}, state=${ctx.state}`
      );
      return ctx;
    } catch (e) {
      console.error("Failed to initialize audio:", e);
      return null;
    }
  }, []);

  const queueAudio = useCallback((samples: Float32Array) => {
    const ctx = audioContextRef.current;
    const gainNode = gainNodeRef.current;

    if (!ctx || !gainNode || samples.length === 0) {
      return;
    }

    // Resume if suspended
    if (ctx.state === "suspended") {
      ctx.resume().catch(() => {});
      return; // Skip this batch, will catch up
    }

    // Samples are stereo interleaved (left, right, left, right, ...)
    const numSamples = Math.floor(samples.length / 2);
    if (numSamples === 0) return;

    const buffer = ctx.createBuffer(2, numSamples, ctx.sampleRate);

    const leftChannel = buffer.getChannelData(0);
    const rightChannel = buffer.getChannelData(1);

    for (let i = 0; i < numSamples; i++) {
      leftChannel[i] = samples[i * 2];
      rightChannel[i] = samples[i * 2 + 1];
    }

    const source = ctx.createBufferSource();
    source.buffer = buffer;
    source.connect(gainNode);

    // Schedule playback
    const now = ctx.currentTime;
    const startTime = Math.max(nextPlayTimeRef.current, now);

    source.start(startTime);
    nextPlayTimeRef.current = startTime + buffer.duration;

    // Prevent audio from getting too far ahead (causes lag)
    // Also prevent it from getting too far behind (causes skipping)
    if (nextPlayTimeRef.current > now + 0.2) {
      // Too far ahead, reset to catch up
      nextPlayTimeRef.current = now + 0.05;
    } else if (nextPlayTimeRef.current < now) {
      // Behind, reset
      nextPlayTimeRef.current = now;
    }
  }, []);

  const setAudioVolume = useCallback((volume: number) => {
    if (gainNodeRef.current) {
      gainNodeRef.current.gain.value = Math.max(0, Math.min(1, volume));
    }
  }, []);

  const closeAudio = useCallback(() => {
    if (audioContextRef.current) {
      try {
        audioContextRef.current.close();
      } catch (e) {
        console.warn("Error closing audio context:", e);
      }
      audioContextRef.current = null;
      gainNodeRef.current = null;
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
