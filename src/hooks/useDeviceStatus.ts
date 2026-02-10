// Hook that polls the backend for device connection status.
// Checks every POLL_INTERVAL_MS whether a PM3 device is connected.

import { useState, useEffect, useRef, useCallback } from 'react';
import { getWizardState } from '../lib/api';
import { TIMEOUTS } from '../lib/constants';

export interface DeviceStatus {
  /** Whether a PM3 device is currently connected */
  connected: boolean;
  /** Serial port path (e.g. COM3, /dev/ttyACM0) or null */
  port: string | null;
  /** Device model string or null */
  model: string | null;
  /** Firmware version or null */
  firmware: string | null;
  /** Whether the last poll encountered an error */
  pollError: boolean;
  /** Force an immediate poll */
  refresh: () => void;
}

export function useDeviceStatus(): DeviceStatus {
  const [connected, setConnected] = useState(false);
  const [port, setPort] = useState<string | null>(null);
  const [model, setModel] = useState<string | null>(null);
  const [firmware, setFirmware] = useState<string | null>(null);
  const [pollError, setPollError] = useState(false);
  const intervalRef = useRef<ReturnType<typeof setInterval> | null>(null);

  const poll = useCallback(async () => {
    try {
      const state = await getWizardState();
      setPollError(false);

      if (state.step === 'DeviceConnected') {
        setConnected(true);
        setPort(state.data.port);
        setModel(state.data.model);
        setFirmware(state.data.firmware);
      } else if (state.step === 'Idle' || state.step === 'Error') {
        setConnected(false);
        setPort(null);
        setModel(null);
        setFirmware(null);
      }
      // For other states (scanning, writing, etc.) keep the current device info
    } catch {
      setPollError(true);
      setConnected(false);
      setPort(null);
      setModel(null);
      setFirmware(null);
    }
  }, []);

  const refresh = useCallback(() => {
    poll();
  }, [poll]);

  useEffect(() => {
    // Initial poll
    poll();

    intervalRef.current = setInterval(poll, TIMEOUTS.DEVICE_POLL_MS);

    return () => {
      if (intervalRef.current !== null) {
        clearInterval(intervalRef.current);
      }
    };
  }, [poll]);

  return { connected, port, model, firmware, pollError, refresh };
}
