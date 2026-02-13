import { createContext, useCallback, useContext, useEffect, useRef, useState, type ReactNode } from 'react';
import { listen } from '@tauri-apps/api/event';

export interface LogLine {
  text: string;
  isError: boolean;
  timestamp: number;
}

interface Pm3OutputPayload {
  text: string;
  isError: boolean;
}

const MAX_LINES = 500;

interface TerminalLogValue {
  lines: LogLine[];
  clear: () => void;
}

const TerminalLogCtx = createContext<TerminalLogValue | null>(null);

export function TerminalLogProvider({ children }: { children: ReactNode }) {
  const [lines, setLines] = useState<LogLine[]>([]);
  const linesRef = useRef(lines);
  linesRef.current = lines;

  useEffect(() => {
    const unlisten = listen<Pm3OutputPayload>('pm3-output', (event) => {
      const newLine: LogLine = {
        text: event.payload.text,
        isError: event.payload.isError,
        timestamp: Date.now(),
      };
      setLines(prev => {
        const next = [...prev, newLine];
        return next.length > MAX_LINES ? next.slice(next.length - MAX_LINES) : next;
      });
    });
    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  const clear = useCallback(() => setLines([]), []);

  return (
    <TerminalLogCtx.Provider value={{ lines, clear }}>
      {children}
    </TerminalLogCtx.Provider>
  );
}

export function useTerminalLog(): TerminalLogValue {
  const ctx = useContext(TerminalLogCtx);
  if (!ctx) {
    throw new Error('useTerminalLog must be used within a TerminalLogProvider');
  }
  return ctx;
}
