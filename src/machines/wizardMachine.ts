// XState v5 state machine for the PM3 clone wizard flow.
// Each async state invokes a Tauri backend command via fromPromise actors.

import { setup, assign, fromPromise } from 'xstate';
import type {
  Frequency,
  CardType,
  BlankType,
  CardData,
  RecoveryAction,
  WizardState,
} from './types';
import * as api from '../lib/api';

// -- Machine context --

export interface WizardContext {
  // Device
  port: string | null;
  model: string | null;
  firmware: string | null;

  // Source card
  frequency: Frequency | null;
  cardType: CardType | null;
  cardData: CardData | null;
  cloneable: boolean;
  recommendedBlank: BlankType | null;

  // Blank card
  expectedBlank: BlankType | null;
  blankType: BlankType | null;
  readyToWrite: boolean;

  // Write progress
  writeProgress: number;
  currentBlock: number | null;
  totalBlocks: number | null;

  // Verification
  verifySuccess: boolean | null;
  mismatchedBlocks: number[];

  // Completion
  completionTimestamp: string | null;

  // Error
  errorMessage: string | null;
  errorUserMessage: string | null;
  errorRecoverable: boolean;
  errorRecoveryAction: RecoveryAction | null;
  errorSource: 'scan' | 'write' | 'detect' | 'verify' | 'blank' | null;
}

const initialContext: WizardContext = {
  port: null,
  model: null,
  firmware: null,
  frequency: null,
  cardType: null,
  cardData: null,
  cloneable: false,
  recommendedBlank: null,
  expectedBlank: null,
  blankType: null,
  readyToWrite: false,
  writeProgress: 0,
  currentBlock: null,
  totalBlocks: null,
  verifySuccess: null,
  mismatchedBlocks: [],
  completionTimestamp: null,
  errorMessage: null,
  errorUserMessage: null,
  errorRecoverable: false,
  errorRecoveryAction: null,
  errorSource: null,
};

// -- Events --

export type WizardEvent =
  | { type: 'DETECT' }
  | { type: 'DEVICE_FOUND'; port: string; model: string; firmware: string }
  | { type: 'SCAN' }
  | { type: 'CARD_FOUND'; frequency: Frequency; cardType: CardType; cardData: CardData; cloneable: boolean; recommendedBlank: BlankType }
  | { type: 'SKIP_TO_BLANK'; expectedBlank: BlankType }
  | { type: 'BLANK_READY'; blankType: BlankType; readyToWrite: boolean }
  | { type: 'WRITE' }
  | { type: 'WRITE_PROGRESS'; progress: number; currentBlock: number | null; totalBlocks: number | null }
  | { type: 'WRITE_COMPLETE' }
  | { type: 'VERIFY_RESULT'; success: boolean; mismatchedBlocks: number[] }
  | { type: 'FINISH' }
  | { type: 'ERROR'; message: string; userMessage: string; recoverable: boolean; recoveryAction: RecoveryAction | null }
  | { type: 'RESET' };

// -- Machine definition --

export const wizardMachine = setup({
  types: {} as {
    context: WizardContext;
    events: WizardEvent;
  },
  actors: {
    detectDevice: fromPromise<WizardState>(async () => {
      return api.detectDevice();
    }),
    scanCard: fromPromise<WizardState>(async () => {
      return api.scanCard();
    }),
    detectBlank: fromPromise<WizardState, WizardContext>(async ({ input }) => {
      return api.detectBlank(input.port!);
    }),
    writeClone: fromPromise<WizardState, WizardContext>(async ({ input }) => {
      return api.writeCloneWithData(
        input.port!,
        input.cardType!,
        input.cardData!.uid,
        input.cardData!.decoded,
        input.blankType ?? undefined,
      );
    }),
    verifyClone: fromPromise<WizardState, WizardContext>(async ({ input }) => {
      return api.verifyCloneWithData(
        input.port!,
        input.cardData!.uid,
        input.cardType!,
        input.cardData!.decoded,
        input.blankType ?? undefined,
      );
    }),
    resetBackend: fromPromise<WizardState>(async () => {
      return api.resetWizard();
    }),
  },
}).createMachine({
  id: 'wizard',
  initial: 'idle',
  context: initialContext,

  states: {
    idle: {
      on: {
        DETECT: { target: 'detectingDevice' },
      },
    },

    detectingDevice: {
      invoke: {
        src: 'detectDevice',
        onDone: {
          target: 'deviceConnected',
          actions: assign({
            port: ({ event }) => {
              const ws = event.output;
              if (ws.step === 'DeviceConnected') return ws.data.port;
              return null;
            },
            model: ({ event }) => {
              const ws = event.output;
              if (ws.step === 'DeviceConnected') return ws.data.model;
              return null;
            },
            firmware: ({ event }) => {
              const ws = event.output;
              if (ws.step === 'DeviceConnected') return ws.data.firmware;
              return null;
            },
          }),
        },
        onError: {
          target: 'error',
          actions: assign({
            errorMessage: ({ event }) => String(event.error),
            errorUserMessage: () => 'Could not detect a Proxmark3 device. Check the USB connection and try again.',
            errorRecoverable: () => true,
            errorRecoveryAction: () => 'Reconnect' as RecoveryAction,
            errorSource: () => 'detect' as const,
          }),
        },
      },
    },

    deviceConnected: {
      on: {
        SCAN: { target: 'scanningCard' },
        RESET: { target: 'idle', actions: assign(() => initialContext) },
      },
    },

    scanningCard: {
      invoke: {
        src: 'scanCard',
        onDone: [
          {
            guard: ({ event }) => {
              const ws = event.output;
              return ws.step === 'CardIdentified';
            },
            target: 'cardIdentified',
            actions: assign({
              frequency: ({ event }) => {
                const ws = event.output;
                if (ws.step === 'CardIdentified') return ws.data.frequency;
                return null;
              },
              cardType: ({ event }) => {
                const ws = event.output;
                if (ws.step === 'CardIdentified') return ws.data.card_type;
                return null;
              },
              cardData: ({ event }) => {
                const ws = event.output;
                if (ws.step === 'CardIdentified') return ws.data.card_data;
                return null;
              },
              cloneable: ({ event }) => {
                const ws = event.output;
                if (ws.step === 'CardIdentified') return ws.data.cloneable;
                return false;
              },
              recommendedBlank: ({ event }) => {
                const ws = event.output;
                if (ws.step === 'CardIdentified') return ws.data.recommended_blank;
                return null;
              },
            }),
          },
          {
            target: 'error',
            actions: assign({
              errorMessage: ({ event }) => {
                const ws = event.output;
                if (ws.step === 'Error') return ws.data.message;
                return 'Unexpected scan result';
              },
              errorUserMessage: ({ event }) => {
                const ws = event.output;
                if (ws.step === 'Error') return ws.data.user_message;
                return 'No card detected. Place a card on the reader and try again.';
              },
              errorRecoverable: () => true,
              errorRecoveryAction: () => 'Retry' as RecoveryAction,
              errorSource: () => 'scan' as const,
            }),
          },
        ],
        onError: {
          target: 'error',
          actions: assign({
            errorMessage: ({ event }) => String(event.error),
            errorUserMessage: () => 'Card scan failed. Check device connection.',
            errorRecoverable: () => true,
            errorRecoveryAction: () => 'Retry' as RecoveryAction,
            errorSource: () => 'scan' as const,
          }),
        },
      },
    },

    cardIdentified: {
      on: {
        SKIP_TO_BLANK: {
          target: 'waitingForBlank',
          actions: assign({
            expectedBlank: ({ event }) => event.expectedBlank,
          }),
        },
        RESET: { target: 'idle', actions: assign(() => initialContext) },
      },
    },

    waitingForBlank: {
      invoke: {
        src: 'detectBlank',
        input: ({ context }: { context: WizardContext }) => context,
        onDone: [
          {
            guard: ({ event }) => event.output.step === 'BlankDetected',
            target: 'blankDetected',
            actions: assign({
              blankType: ({ event }) => {
                const ws = event.output;
                if (ws.step === 'BlankDetected') return ws.data.blank_type;
                return null;
              },
              readyToWrite: ({ event }) => {
                const ws = event.output;
                if (ws.step === 'BlankDetected') return ws.data.ready_to_write;
                return false;
              },
            }),
          },
          {
            target: 'error',
            actions: assign({
              errorMessage: () => 'Blank card not detected or incompatible',
              errorUserMessage: () => 'Place the correct blank card on the reader.',
              errorRecoverable: () => true,
              errorRecoveryAction: () => 'Retry' as RecoveryAction,
              errorSource: () => 'blank' as const,
            }),
          },
        ],
        onError: {
          target: 'error',
          actions: assign({
            errorMessage: ({ event }) => String(event.error),
            errorUserMessage: () => 'Failed to detect blank card.',
            errorRecoverable: () => true,
            errorRecoveryAction: () => 'Retry' as RecoveryAction,
            errorSource: () => 'blank' as const,
          }),
        },
      },
      on: {
        RESET: { target: 'idle', actions: assign(() => initialContext) },
      },
    },

    blankDetected: {
      on: {
        WRITE: {
          guard: ({ context }) => context.readyToWrite,
          target: 'writing',
        },
        RESET: { target: 'idle', actions: assign(() => initialContext) },
      },
    },

    writing: {
      invoke: {
        src: 'writeClone',
        input: ({ context }: { context: WizardContext }) => context,
        onDone: [
          {
            guard: ({ event }) => {
              const ws = event.output;
              return ws.step === 'Verifying' || ws.step === 'VerificationComplete' || ws.step === 'Complete';
            },
            target: 'verifying',
          },
          {
            target: 'error',
            actions: assign({
              errorMessage: ({ event }) => {
                const ws = event.output;
                if (ws.step === 'Error') return ws.data.message;
                return 'Write operation returned unexpected state';
              },
              errorUserMessage: ({ event }) => {
                const ws = event.output;
                if (ws.step === 'Error') return ws.data.user_message;
                return 'Write failed. Do not remove the card.';
              },
              errorRecoverable: ({ event }) => {
                const ws = event.output;
                if (ws.step === 'Error') return ws.data.recoverable;
                return false;
              },
              errorRecoveryAction: ({ event }) => {
                const ws = event.output;
                if (ws.step === 'Error') return ws.data.recovery_action;
                return null;
              },
              errorSource: () => 'write' as const,
            }),
          },
        ],
        onError: {
          target: 'error',
          actions: assign({
            errorMessage: ({ event }) => String(event.error),
            errorUserMessage: () => 'Write operation failed. Do not remove the card.',
            errorRecoverable: () => true,
            errorRecoveryAction: () => 'Retry' as RecoveryAction,
            errorSource: () => 'write' as const,
          }),
        },
      },
      on: {
        WRITE_PROGRESS: {
          actions: assign({
            writeProgress: ({ event }) => event.progress,
            currentBlock: ({ event }) => event.currentBlock,
            totalBlocks: ({ event }) => event.totalBlocks,
          }),
        },
      },
    },

    verifying: {
      invoke: {
        src: 'verifyClone',
        input: ({ context }: { context: WizardContext }) => context,
        onDone: [
          {
            guard: ({ event }) => event.output.step === 'VerificationComplete',
            target: 'verificationComplete',
            actions: assign({
              verifySuccess: ({ event }) => {
                const ws = event.output;
                if (ws.step === 'VerificationComplete') return ws.data.success;
                return false;
              },
              mismatchedBlocks: ({ event }) => {
                const ws = event.output;
                if (ws.step === 'VerificationComplete') return ws.data.mismatched_blocks;
                return [];
              },
            }),
          },
          {
            target: 'error',
            actions: assign({
              errorMessage: () => 'Verification returned unexpected state',
              errorUserMessage: () => 'Verification could not complete.',
              errorRecoverable: () => true,
              errorRecoveryAction: () => 'Retry' as RecoveryAction,
              errorSource: () => 'verify' as const,
            }),
          },
        ],
        onError: {
          target: 'error',
          actions: assign({
            errorMessage: ({ event }) => String(event.error),
            errorUserMessage: () => 'Verification failed.',
            errorRecoverable: () => true,
            errorRecoveryAction: () => 'Retry' as RecoveryAction,
            errorSource: () => 'verify' as const,
          }),
        },
      },
    },

    verificationComplete: {
      on: {
        FINISH: {
          target: 'complete',
          actions: assign({
            completionTimestamp: () => new Date().toISOString(),
          }),
        },
        RESET: { target: 'idle', actions: assign(() => initialContext) },
      },
    },

    complete: {
      on: {
        RESET: { target: 'idle', actions: assign(() => initialContext) },
      },
    },

    error: {
      on: {
        RESET: { target: 'idle', actions: assign(() => initialContext) },
        DETECT: {
          guard: ({ context }) => context.errorRecoveryAction === 'Reconnect',
          target: 'detectingDevice',
        },
        WRITE: {
          guard: ({ context }) =>
            context.errorRecoveryAction === 'Retry' &&
            (context.errorSource === 'write' || context.errorSource === 'blank'),
          target: 'waitingForBlank',
        },
        SCAN: {
          guard: ({ context }) =>
            context.errorRecoveryAction === 'Retry' &&
            context.errorSource !== 'write' &&
            context.errorSource !== 'blank',
          target: 'scanningCard',
        },
      },
    },
  },
});

