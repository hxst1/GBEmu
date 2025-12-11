"use client";

import { useCallback } from "react";
import { get, set, del, keys, createStore } from "idb-keyval";

// Create custom stores for different data types
const sramStore = createStore("gbemu-sram", "saves");
const stateStore = createStore("gbemu-states", "savestates");

export function useSaveData() {
  // Save SRAM (battery-backed save)
  const saveSram = useCallback(async (gameName: string, data: Uint8Array) => {
    const key = `sram:${gameName}`;
    const saveData = {
      data: Array.from(data),
      timestamp: Date.now(),
    };
    await set(key, saveData, sramStore);
  }, []);

  // Load SRAM
  const loadSram = useCallback(
    async (gameName: string): Promise<Uint8Array | null> => {
      const key = `sram:${gameName}`;
      const saveData = await get<{ data: number[]; timestamp: number }>(
        key,
        sramStore
      );

      if (saveData?.data) {
        return new Uint8Array(saveData.data);
      }
      return null;
    },
    []
  );

  // Save state
  const saveStateData = useCallback(
    async (gameName: string, data: Uint8Array) => {
      const key = `state:${gameName}`;
      const saveData = {
        data: Array.from(data),
        timestamp: Date.now(),
      };
      await set(key, saveData, stateStore);
    },
    []
  );

  // Load state
  const loadStateData = useCallback(
    async (gameName: string): Promise<Uint8Array | null> => {
      const key = `state:${gameName}`;
      const saveData = await get<{ data: number[]; timestamp: number }>(
        key,
        stateStore
      );

      if (saveData?.data) {
        return new Uint8Array(saveData.data);
      }
      return null;
    },
    []
  );

  // List all saves
  const listSaves = useCallback(async () => {
    const saves: { name: string; type: string; date: Date }[] = [];

    // Get SRAM saves
    const sramKeys = await keys(sramStore);
    for (const key of sramKeys) {
      if (typeof key === "string" && key.startsWith("sram:")) {
        const data = await get<{ timestamp: number }>(key, sramStore);
        if (data) {
          saves.push({
            name: key.replace("sram:", ""),
            type: "sram",
            date: new Date(data.timestamp),
          });
        }
      }
    }

    // Get save states
    const stateKeys = await keys(stateStore);
    for (const key of stateKeys) {
      if (typeof key === "string" && key.startsWith("state:")) {
        const data = await get<{ timestamp: number }>(key, stateStore);
        if (data) {
          saves.push({
            name: key.replace("state:", ""),
            type: "state",
            date: new Date(data.timestamp),
          });
        }
      }
    }

    // Sort by date, newest first
    saves.sort((a, b) => b.date.getTime() - a.date.getTime());

    return saves;
  }, []);

  // Delete save
  const deleteSave = useCallback(async (gameName: string, type: string) => {
    const store = type === "sram" ? sramStore : stateStore;
    const key = `${type}:${gameName}`;
    await del(key, store);
  }, []);

  // Export save data as file
  const exportSave = useCallback(
    async (gameName: string, type: string): Promise<Blob | null> => {
      const store = type === "sram" ? sramStore : stateStore;
      const key = `${type}:${gameName}`;
      const saveData = await get<{ data: number[] }>(key, store);

      if (saveData?.data) {
        return new Blob([new Uint8Array(saveData.data)], {
          type: "application/octet-stream",
        });
      }
      return null;
    },
    []
  );

  // Import save data from file
  const importSave = useCallback(
    async (gameName: string, type: string, data: Uint8Array) => {
      const store = type === "sram" ? sramStore : stateStore;
      const key = `${type}:${gameName}`;
      const saveData = {
        data: Array.from(data),
        timestamp: Date.now(),
      };
      await set(key, saveData, store);
    },
    []
  );

  return {
    saveSram,
    loadSram,
    saveStateData,
    loadStateData,
    listSaves,
    deleteSave,
    exportSave,
    importSave,
  };
}
