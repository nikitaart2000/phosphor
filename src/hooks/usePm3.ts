// Hook for listening to raw PM3 CLI output events from the Tauri backend.
// The backend emits 'pm3:output' events with line-by-line CLI output,
// and 'pm3:status' events for run state changes.

import { useState, useEffect, useCallback, useRef } from 'react';
import { listen } from '@tauri-apps/api/event';

interface Pm3OutputPayload {
  line: string;
  timestamp: string;
}

interface Pm3StatusPayload {
  running: boolean;
}

const MAX_LINES = 500;

export interface UsePm3Return {
  /** Buffered PM3 output lines (most recent last, capped at 500) */
  lines: string[];
  /** Whether a PM3 command is currently executing */
  isRunning: boolean;
  /** Clear the output buffer */
  clear: () => void;
}

export function usePm3(): UsePm3Return {
  const [lines, setLines] = useState<string[]>([]);
  const [isRunning, setIsRunning] = useState(false);
  const linesRef = useRef<string[]>([]);

  useEffect(() => {
    let unmounted = false;

    const setupListeners = async () => {
      const unlistenOutput = await listen<Pm3OutputPayload>('pm3:output', (event) => {
        if (unmounted) return;
        const newLine = event.payload.line;
        linesRef.current = [...linesRef.current.slice(-(MAX_LINES - 1)), newLine];
        setLines([...linesRef.current]);
      });

      const unlistenStatus = await listen<Pm3StatusPayload>('pm3:status', (event) => {
        if (unmounted) return;
        setIsRunning(event.payload.running);
      });

      return () => {
        unmounted = true;
        unlistenOutput();
        unlistenStatus();
      };
    };

    const cleanupPromise = setupListeners();

    return () => {
      unmounted = true;
      cleanupPromise.then((cleanup) => cleanup());
    };
  }, []);

  const clear = useCallback(() => {
    linesRef.current = [];
    setLines([]);
  }, []);

  return { lines, isRunning, clear };
}
