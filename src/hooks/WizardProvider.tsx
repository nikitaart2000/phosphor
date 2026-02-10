// Shared React context for the XState wizard machine.
// Provides a single machine instance to the entire component tree,
// and listens for Tauri write-progress events.

import { createContext, useCallback, useContext, useEffect, useMemo, type ReactNode } from 'react';
import { useMachine } from '@xstate/react';
import { listen } from '@tauri-apps/api/event';
import { wizardMachine } from '../machines/wizardMachine';
import type { WizardContext as WizCtx, WizardEvent } from '../machines/wizardMachine';
import type { WizardStepName, BlankType } from '../machines/types';
import * as api from '../lib/api';

type StepName =
  | 'idle'
  | 'detectingDevice'
  | 'deviceConnected'
  | 'scanningCard'
  | 'cardIdentified'
  | 'waitingForBlank'
  | 'blankDetected'
  | 'writing'
  | 'verifying'
  | 'verificationComplete'
  | 'complete'
  | 'error';

// Map XState state value strings to WizardState step names
const STATE_TO_STEP: Record<StepName, WizardStepName> = {
  idle: 'Idle',
  detectingDevice: 'DetectingDevice',
  deviceConnected: 'DeviceConnected',
  scanningCard: 'ScanningCard',
  cardIdentified: 'CardIdentified',
  waitingForBlank: 'WaitingForBlank',
  blankDetected: 'BlankDetected',
  writing: 'Writing',
  verifying: 'Verifying',
  verificationComplete: 'VerificationComplete',
  complete: 'Complete',
  error: 'Error',
};

export interface UseWizardReturn {
  /** Current machine context with device, card, blank, and error data */
  context: WizCtx;
  /** Current wizard step name (PascalCase, matching Rust enum) */
  currentStep: WizardStepName;
  /** Check if the wizard is at a specific step */
  isStep: (step: WizardStepName) => boolean;
  /** Whether the machine is in any loading/async state */
  isLoading: boolean;
  /** Start device detection */
  detect: () => void;
  /** Start card scanning */
  scan: () => void;
  /** Proceed to blank card stage with specified blank type */
  skipToBlank: (expectedBlank: BlankType) => void;
  /** Start clone write operation */
  write: () => void;
  /** Mark wizard as finished */
  finish: () => void;
  /** Reset wizard to idle */
  reset: () => void;
  /** Raw XState send function for advanced use */
  send: (event: WizardEvent) => void;
}

const WizardCtx = createContext<UseWizardReturn | null>(null);

interface WriteProgressPayload {
  progress: number;
  current_block: number | null;
  total_blocks: number | null;
}

export function WizardProvider({ children }: { children: ReactNode }) {
  const [state, send] = useMachine(wizardMachine);

  // Listen for Tauri write-progress events emitted by the Rust backend
  useEffect(() => {
    const unlisten = listen<WriteProgressPayload>('write-progress', (event) => {
      send({
        type: 'WRITE_PROGRESS',
        progress: event.payload.progress,
        currentBlock: event.payload.current_block,
        totalBlocks: event.payload.total_blocks,
      });
    });
    return () => {
      unlisten.then((fn) => fn());
    };
  }, [send]);

  const stateValue = state.value as StepName;
  const currentStep = STATE_TO_STEP[stateValue] ?? 'Idle';

  const isStep = useCallback(
    (step: WizardStepName): boolean => currentStep === step,
    [currentStep],
  );

  const isLoading = useMemo(
    () =>
      stateValue === 'detectingDevice' ||
      stateValue === 'scanningCard' ||
      stateValue === 'waitingForBlank' ||
      stateValue === 'writing' ||
      stateValue === 'verifying',
    [stateValue],
  );

  const detect = useCallback(() => send({ type: 'DETECT' }), [send]);
  const scan = useCallback(() => send({ type: 'SCAN' }), [send]);
  const skipToBlank = useCallback(
    (expectedBlank: BlankType) => send({ type: 'SKIP_TO_BLANK', expectedBlank }),
    [send],
  );
  const write = useCallback(() => send({ type: 'WRITE' }), [send]);
  const finish = useCallback(() => send({ type: 'FINISH' }), [send]);
  const reset = useCallback(async () => {
    try { await api.resetWizard(); } catch { /* best-effort backend reset */ }
    send({ type: 'RESET' });
  }, [send]);

  const wizardReturn = useMemo<UseWizardReturn>(
    () => ({
      context: state.context,
      currentStep,
      isStep,
      isLoading,
      detect,
      scan,
      skipToBlank,
      write,
      finish,
      reset,
      send,
    }),
    [state.context, currentStep, isStep, isLoading, detect, scan, skipToBlank, write, finish, reset, send],
  );

  return (
    <WizardCtx.Provider value={wizardReturn}>
      {children}
    </WizardCtx.Provider>
  );
}

export function useWizardContext(): UseWizardReturn {
  const ctx = useContext(WizardCtx);
  if (!ctx) {
    throw new Error('useWizardContext must be used within a WizardProvider');
  }
  return ctx;
}
