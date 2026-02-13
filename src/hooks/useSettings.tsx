import { createContext, useCallback, useContext, useEffect, useState, type ReactNode } from 'react';

interface PhosphorSettings {
  expertMode: boolean;
}

const DEFAULT_SETTINGS: PhosphorSettings = {
  expertMode: false,
};

const STORAGE_KEY = 'phosphor-settings';

interface SettingsContextValue {
  settings: PhosphorSettings;
  updateSettings: (partial: Partial<PhosphorSettings>) => void;
}

const SettingsCtx = createContext<SettingsContextValue | null>(null);

function loadSettings(): PhosphorSettings {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (raw) {
      const parsed = JSON.parse(raw);
      return { ...DEFAULT_SETTINGS, ...parsed };
    }
  } catch {
    // Corrupted storage -- fall back to defaults
  }
  return { ...DEFAULT_SETTINGS };
}

function saveSettings(settings: PhosphorSettings) {
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(settings));
  } catch {
    // Storage full or unavailable -- silently ignore
  }
}

export function SettingsProvider({ children }: { children: ReactNode }) {
  const [settings, setSettings] = useState<PhosphorSettings>(loadSettings);

  // Persist to localStorage whenever settings change
  useEffect(() => {
    saveSettings(settings);
  }, [settings]);

  const updateSettings = useCallback((partial: Partial<PhosphorSettings>) => {
    setSettings(prev => ({ ...prev, ...partial }));
  }, []);

  return (
    <SettingsCtx.Provider value={{ settings, updateSettings }}>
      {children}
    </SettingsCtx.Provider>
  );
}

export function useSettings(): SettingsContextValue {
  const ctx = useContext(SettingsCtx);
  if (!ctx) {
    throw new Error('useSettings must be used within a SettingsProvider');
  }
  return ctx;
}
