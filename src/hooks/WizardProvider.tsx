// Shared React context for the XState wizard machine.
// Provides a single machine instance to the entire component tree,
// and listens for Tauri write-progress events.

import { createContext, useCallback, useContext, useEffect, useMemo, type ReactNode } from 'react';
import { useMachine } from '@xstate/react';
import { listen } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';
import { wizardMachine } from '../machines/wizardMachine';
import type { WizardContext as WizCtx, WizardEvent } from '../machines/wizardMachine';
import type { WizardStepName, WizardState, BlankType, FirmwareProgress, HfProgressPayload } from '../machines/types';
import * as api from '../lib/api';

type StepName =
  | 'idle'
  | 'detectingDevice'
  | 'checkingFirmware'
  | 'firmwareOutdated'
  | 'updatingFirmware'
  | 'redetectingDevice'
  | 'deviceConnected'
  | 'scanningCard'
  | 'cardIdentified'
  | 'waitingForBlank'
  | 'blankDetected'
  | 'writing'
  | 'verifying'
  | 'verificationComplete'
  | 'complete'
  | 'error'
  | 'hfProcessing'
  | 'hfDumpReady';

// Map XState state value strings to WizardState step names
const STATE_TO_STEP: Record<StepName, WizardStepName> = {
  idle: 'Idle',
  detectingDevice: 'DetectingDevice',
  checkingFirmware: 'CheckingFirmware',
  firmwareOutdated: 'FirmwareOutdated',
  updatingFirmware: 'UpdatingFirmware',
  redetectingDevice: 'RedetectingDevice',
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
  hfProcessing: 'HfProcessing',
  hfDumpReady: 'HfDumpReady',
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
  /** Start firmware flash */
  updateFirmware: () => void;
  /** Skip firmware update and proceed to scan */
  skipFirmware: () => void;
  /** Cancel in-progress firmware flash */
  cancelFirmware: () => void;
  /** Select hardware variant (used when capabilities mismatch prevents auto-detection) */
  selectVariant: (variant: 'rdv4' | 'rdv4-bt' | 'generic') => void;
  /** Go back to device-connected state, clearing card data but keeping device */
  backToScan: () => Promise<void>;
  /** Soft reset: return to device-connected from complete/error, keeping device */
  softReset: () => Promise<void>;
  /** Disconnect device and return to idle */
  disconnect: () => Promise<void>;
  /** Load a saved card into the wizard as if it was just scanned */
  loadSavedCard: (card: { frequency: string; cardType: string; uid: string; raw: string; decoded: Record<string, string>; cloneable: boolean; recommendedBlank: string }) => Promise<void>;
  /** Re-detect blank card after erase (BlankDetected -> WaitingForBlank) */
  reDetectBlank: () => Promise<void>;
  /** Start HF key recovery / dump process (Classic: autopwn, UL/NTAG/iCLASS: dump) */
  startHfProcess: () => void;
  /** Cancel a running HF operation (kills child process + resets FSM) */
  cancelHf: () => Promise<void>;
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

  // Listen for HF progress events emitted by Rust during autopwn/dump
  useEffect(() => {
    const unlisten = listen<HfProgressPayload>('hf-progress', (event) => {
      send({
        type: 'HF_PROGRESS',
        phase: event.payload.phase,
        keysFound: event.payload.keys_found,
        keysTotal: event.payload.keys_total,
        elapsed: event.payload.elapsed_secs,
      });
    });
    return () => {
      unlisten.then((fn) => fn());
    };
  }, [send]);

  // Listen for firmware flash events emitted by Rust backend
  useEffect(() => {
    const unlistenProgress = listen<FirmwareProgress>('firmware-progress', (event) => {
      send({
        type: 'FIRMWARE_PROGRESS',
        progress: Math.min(100, Math.max(0, event.payload.percent)),
        message: event.payload.message,
        phase: event.payload.phase,
      });
    });
    const unlistenComplete = listen<FirmwareProgress>('firmware-complete', () => {
      send({ type: 'FIRMWARE_COMPLETE' });
    });
    const unlistenFailed = listen<FirmwareProgress>('firmware-failed', (event) => {
      send({ type: 'FIRMWARE_FAILED', message: event.payload.message });
    });
    return () => {
      unlistenProgress.then((fn) => fn());
      unlistenComplete.then((fn) => fn());
      unlistenFailed.then((fn) => fn());
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
      stateValue === 'checkingFirmware' ||
      stateValue === 'updatingFirmware' ||
      stateValue === 'redetectingDevice' ||
      stateValue === 'scanningCard' ||
      stateValue === 'waitingForBlank' ||
      stateValue === 'writing' ||
      stateValue === 'verifying' ||
      stateValue === 'hfProcessing',
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

  const updateFirmware = useCallback(async () => {
    const { port, hardwareVariant } = state.context;
    if (!port || !hardwareVariant) return;
    send({ type: 'UPDATE_FIRMWARE' });
    try {
      await api.flashFirmware(port, hardwareVariant);
    } catch (err) {
      console.error('updateFirmware: flashFirmware invoke failed', err);
      const msg = typeof err === 'string' ? err
        : err instanceof Error ? err.message
        : typeof err === 'object' && err !== null ? (Object.values(err)[0] as string) ?? JSON.stringify(err)
        : String(err);
      send({ type: 'FIRMWARE_FAILED', message: msg });
    }
  }, [send, state.context]);

  const skipFirmware = useCallback(() => {
    send({ type: 'SKIP_FIRMWARE' });
  }, [send]);

  const cancelFirmware = useCallback(async () => {
    try {
      await api.cancelFlash();
    } catch (err) {
      console.error('cancelFirmware: cancelFlash failed', err);
    }
    send({ type: 'CANCEL_FIRMWARE' });
  }, [send]);

  const selectVariant = useCallback((variant: 'rdv4' | 'rdv4-bt' | 'generic') => {
    send({ type: 'SELECT_VARIANT', variant });
  }, [send]);

  const backToScan = useCallback(async () => {
    try {
      await invoke<WizardState>('wizard_action', {
        action: { action: 'BackToScan' },
      });
      send({ type: 'BACK_TO_SCAN' });
    } catch (err) {
      console.error('backToScan: Rust BackToScan failed, resetting', err);
      reset();
    }
  }, [send, reset]);

  const softReset = useCallback(async () => {
    try {
      await invoke<WizardState>('wizard_action', {
        action: { action: 'SoftReset' },
      });
      send({ type: 'SOFT_RESET' });
    } catch (err) {
      console.error('softReset: Rust SoftReset failed, resetting', err);
      reset();
    }
  }, [send, reset]);

  const disconnect = useCallback(async () => {
    try {
      await invoke<WizardState>('wizard_action', {
        action: { action: 'Disconnect' },
      });
      send({ type: 'DISCONNECT' });
    } catch (err) {
      console.error('disconnect: Rust Disconnect failed, resetting', err);
      reset();
    }
  }, [send, reset]);

  const reDetectBlank = useCallback(async () => {
    try {
      await invoke<WizardState>('wizard_action', {
        action: { action: 'ReDetectBlank' },
      });
      send({ type: 'RE_DETECT_BLANK' });
    } catch (err) {
      console.error('reDetectBlank: Rust ReDetectBlank failed, resetting', err);
      reset();
    }
  }, [send, reset]);

  const startHfProcess = useCallback(() => {
    send({ type: 'START_HF_PROCESS' });
  }, [send]);

  const cancelHf = useCallback(async () => {
    try {
      await api.cancelHfOperation();
    } catch (err) {
      console.error('cancelHf: cancelHfOperation failed', err);
    }
    try {
      await invoke<WizardState>('wizard_action', {
        action: { action: 'CancelHfProcess' },
      });
    } catch (err) {
      console.error('cancelHf: Rust CancelHfProcess failed', err);
    }
    send({ type: 'CANCEL_HF' });
  }, [send]);

  const loadSavedCard = useCallback(async (card: {
    frequency: string;
    cardType: string;
    uid: string;
    raw: string;
    decoded: Record<string, string>;
    cloneable: boolean;
    recommendedBlank: string;
  }) => {
    try {
      await invoke<WizardState>('wizard_action', {
        action: {
          action: 'LoadSavedCard',
          payload: {
            frequency: card.frequency,
            card_type: card.cardType,
            uid: card.uid,
            raw: card.raw,
            decoded: card.decoded,
            cloneable: card.cloneable,
            recommended_blank: card.recommendedBlank,
          },
        },
      });
      send({
        type: 'LOAD_SAVED_CARD',
        frequency: card.frequency as WizCtx['frequency'] & string,
        cardType: card.cardType as WizCtx['cardType'] & string,
        cardData: { uid: card.uid, raw: card.raw, decoded: card.decoded },
        cloneable: card.cloneable,
        recommendedBlank: card.recommendedBlank as WizCtx['recommendedBlank'] & string,
      });
    } catch (err) {
      console.error('loadSavedCard: Rust LoadSavedCard failed, resetting', err);
      reset();
    }
  }, [send, reset]);

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
      updateFirmware,
      skipFirmware,
      cancelFirmware,
      selectVariant,
      backToScan,
      softReset,
      disconnect,
      loadSavedCard,
      reDetectBlank,
      startHfProcess,
      cancelHf,
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
      // Firmware
      ctx.firmwareStatus, ctx.clientVersion, ctx.deviceFirmwareVersion,
      ctx.hardwareVariant, ctx.firmwarePathExists, ctx.firmwareProgress, ctx.firmwareMessage,
      // HF processing
      ctx.hfPhase, ctx.hfKeysFound, ctx.hfKeysTotal, ctx.hfElapsed, ctx.hfDumpInfo,
      // Callbacks
      detect, scan, skipToBlank, write, finish, reset,
      updateFirmware, skipFirmware, cancelFirmware, selectVariant,
      backToScan, softReset, disconnect, loadSavedCard, reDetectBlank,
      startHfProcess, cancelHf, send,
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
