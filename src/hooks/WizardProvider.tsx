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
        // Rust backend sends progress as 0.0–1.0; XState/UI expects 0–100.
        progress: Math.min(100, Math.max(0, event.payload.progress * 100)),
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

  const detect = useCallback(() => {
    if (isLoading) return;
    send({ type: 'DETECT' });
  }, [send, isLoading]);
  const scan = useCallback(() => {
    if (isLoading) return;
    send({ type: 'SCAN' });
  }, [send, isLoading]);
  const write = useCallback(() => {
    if (isLoading) return;
    send({ type: 'WRITE' });
  }, [send, isLoading]);
  const reset = useCallback(async () => {
    try {
      await api.resetWizard();
    } catch (err) {
      console.error('reset: Rust resetWizard failed, still resetting XState', err);
    }
    // Always send RESET to XState — reset is the recovery action
    send({ type: 'RESET' });
  }, [send]);
  const skipToBlank = useCallback(
    async (expectedBlank: BlankType) => {
      // Sync Rust FSM: CardIdentified → WaitingForBlank
      try {
        await api.proceedToWrite(expectedBlank);
        send({ type: 'SKIP_TO_BLANK', expectedBlank });
      } catch (err) {
        console.error('skipToBlank: Rust proceedToWrite failed, resetting both FSMs', err);
        reset();
      }
    },
    [send, reset],
  );
  const finish = useCallback(async () => {
    // Capture context values before any await to avoid stale closure
    const { cardType, cardData, blankType, port, verifySuccess } = state.context;

    // Sync Rust FSM: VerificationComplete → Complete
    try {
      if (cardType && cardData) {
        await api.markComplete(
          {
            card_type: cardType,
            uid: cardData.uid,
            display_name: cardType,
          },
          {
            card_type: blankType ?? 'T5577',
            uid: cardData.uid,
            display_name: blankType ?? 'T5577',
          },
        );
      }
      send({ type: 'FINISH' });
      // Save clone record to history after successful verification
      if (verifySuccess && cardType && cardData && port) {
        try {
          await api.saveCloneRecord({
            id: null,
            source_type: cardType,
            source_uid: cardData.uid,
            target_type: blankType ?? 'T5577',
            target_uid: cardData.uid,
            port: port,
            success: true,
            timestamp: new Date().toISOString(),
            notes: null,
          });
        } catch { /* best-effort history save */ }
      }
    } catch (err) {
      console.error('finish: Rust markComplete failed, resetting both FSMs', err);
      reset();
    }
  }, [send, state.context, reset]);

  // Destructure context for granular memo deps — avoids re-renders from
  // write-progress updates reaching components that only need step/device info.
  const ctx = state.context;

  const wizardReturn = useMemo<UseWizardReturn>(
    () => ({
      context: ctx,
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
    [
      // Step & loading
      currentStep, isStep, isLoading,
      // Device
      ctx.port, ctx.model, ctx.firmware,
      // Card
      ctx.frequency, ctx.cardType, ctx.cardData,
      ctx.cloneable, ctx.recommendedBlank,
      // Blank
      ctx.expectedBlank, ctx.blankType, ctx.readyToWrite,
      // Write progress
      ctx.writeProgress, ctx.currentBlock, ctx.totalBlocks,
      // Verification
      ctx.verifySuccess, ctx.mismatchedBlocks,
      // Completion
      ctx.completionTimestamp,
      // Error
      ctx.errorMessage, ctx.errorUserMessage,
      ctx.errorRecoverable, ctx.errorRecoveryAction, ctx.errorSource,
      // Callbacks
      detect, scan, skipToBlank, write, finish, reset, send,
    ],
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
