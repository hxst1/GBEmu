'use client';

import { useState, useEffect, useCallback } from 'react';
import { useEmulator } from './EmulatorContext';
import { useSaveData } from '@/hooks/useSaveData';

interface SaveManagerProps {
  onClose: () => void;
}

export function SaveManager({ onClose }: SaveManagerProps) {
  const { saveState, loadState, gameTitle } = useEmulator();
  const { listSaves, deleteSave } = useSaveData();
  const [saves, setSaves] = useState<{ name: string; type: string; date: Date }[]>([]);
  const [isLoading, setIsLoading] = useState(true);

  // Load saves list
  const refreshSaves = useCallback(async () => {
    setIsLoading(true);
    try {
      const savesList = await listSaves();
      setSaves(savesList);
    } catch (error) {
      console.error('Failed to list saves:', error);
    } finally {
      setIsLoading(false);
    }
  }, [listSaves]);

  useEffect(() => {
    refreshSaves();
  }, [refreshSaves]);

  const handleSave = async () => {
    await saveState();
    await refreshSaves();
  };

  const handleLoad = async () => {
    await loadState();
    onClose();
  };

  const handleDelete = async (name: string, type: string) => {
    if (confirm(`Delete ${type} for "${name}"?`)) {
      await deleteSave(name, type);
      await refreshSaves();
    }
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/70 backdrop-blur-sm">
      <div className="bg-gray-900 rounded-2xl p-6 w-full max-w-md mx-4 max-h-[80vh] flex flex-col animate-fade-in">
        <div className="flex justify-between items-center mb-6">
          <h2 className="text-white text-xl font-bold">Save Manager</h2>
          <button
            onClick={onClose}
            className="text-gray-400 hover:text-white transition-colors"
          >
            <CloseIcon />
          </button>
        </div>

        {/* Current game actions */}
        <div className="grid grid-cols-2 gap-3 mb-6">
          <button
            onClick={handleSave}
            className="py-3 px-4 bg-green-600 hover:bg-green-500 text-white rounded-lg transition-colors flex items-center justify-center gap-2"
          >
            <SaveIcon />
            Save State
          </button>
          <button
            onClick={handleLoad}
            className="py-3 px-4 bg-blue-600 hover:bg-blue-500 text-white rounded-lg transition-colors flex items-center justify-center gap-2"
          >
            <LoadIcon />
            Load State
          </button>
        </div>

        {/* Saves list */}
        <div className="flex-1 overflow-y-auto">
          <h3 className="text-gray-400 text-sm mb-3">Saved Data</h3>
          
          {isLoading ? (
            <div className="text-center py-8 text-gray-500">
              Loading...
            </div>
          ) : saves.length === 0 ? (
            <div className="text-center py-8 text-gray-500">
              No saved data found
            </div>
          ) : (
            <div className="space-y-2">
              {saves.map((save, index) => (
                <div
                  key={`${save.name}-${save.type}-${index}`}
                  className="flex items-center justify-between p-3 bg-gray-800 rounded-lg"
                >
                  <div className="flex-1 min-w-0">
                    <p className="text-white text-sm font-medium truncate">
                      {save.name}
                    </p>
                    <p className="text-gray-500 text-xs">
                      {save.type === 'sram' ? 'Save Data' : 'Save State'} •{' '}
                      {formatDate(save.date)}
                    </p>
                  </div>
                  <button
                    onClick={() => handleDelete(save.name, save.type)}
                    className="ml-2 p-2 text-red-400 hover:text-red-300 hover:bg-red-900/50 rounded transition-colors"
                  >
                    <TrashIcon />
                  </button>
                </div>
              ))}
            </div>
          )}
        </div>

        {/* Info */}
        <div className="mt-4 pt-4 border-t border-gray-800 text-gray-500 text-xs text-center">
          <p>Save data is stored locally in your browser.</p>
          <p>Game saves (SRAM) are automatically saved every 30 seconds.</p>
        </div>
      </div>
    </div>
  );
}

function formatDate(date: Date): string {
  const now = new Date();
  const diff = now.getTime() - date.getTime();
  const minutes = Math.floor(diff / 60000);
  const hours = Math.floor(minutes / 60);
  const days = Math.floor(hours / 24);

  if (minutes < 1) return 'Just now';
  if (minutes < 60) return `${minutes}m ago`;
  if (hours < 24) return `${hours}h ago`;
  if (days < 7) return `${days}d ago`;
  
  return date.toLocaleDateString();
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

function SaveIcon() {
  return (
    <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
      <path d="M19 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11l5 5v11a2 2 0 0 1-2 2z" />
      <polyline points="17 21 17 13 7 13 7 21" />
      <polyline points="7 3 7 8 15 8" />
    </svg>
  );
}

function LoadIcon() {
  return (
    <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
      <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" />
      <polyline points="7 10 12 15 17 10" />
      <line x1="12" y1="15" x2="12" y2="3" />
    </svg>
  );
}

function TrashIcon() {
  return (
    <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
      <polyline points="3 6 5 6 21 6" />
      <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2" />
    </svg>
  );
}
