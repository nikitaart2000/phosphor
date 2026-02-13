// Application constants: timeouts, labels, status symbols, and jargon mapping.

import type { WizardStepName, RecoveryAction } from '../machines/types';

// -- Timeouts (milliseconds) --

export const TIMEOUTS = {
  /** Device connection poll interval */
  DEVICE_POLL_MS: 5000,
  /** PM3 command execution timeout */
  COMMAND_TIMEOUT_MS: 30000,
  /** Card scan timeout */
  SCAN_TIMEOUT_MS: 15000,
  /** Write operation timeout */
  WRITE_TIMEOUT_MS: 60000,
  /** Verification timeout */
  VERIFY_TIMEOUT_MS: 30000,
  /** Toast notification auto-dismiss */
  TOAST_DURATION_MS: 4000,
  /** Boot sequence animation duration */
  BOOT_DURATION_MS: 3000,
  /** Typewriter character interval */
  TYPEWRITER_CHAR_MS: 30,
} as const;

// -- Step labels (jargon-free, user-facing) --

export const STEP_LABELS: Record<WizardStepName, string> = {
  Idle: 'Ready',
  DetectingDevice: 'Searching for device...',
  CheckingFirmware: 'Checking firmware...',
  FirmwareOutdated: 'Firmware mismatch',
  UpdatingFirmware: 'Updating firmware...',
  RedetectingDevice: 'Re-detecting device...',
  DeviceConnected: 'Device connected',
  ScanningCard: 'Reading card...',
  CardIdentified: 'Card identified',
  WaitingForBlank: 'Place blank card on reader',
  BlankDetected: 'Blank card ready',
  Writing: 'Writing data...',
  Verifying: 'Verifying clone...',
  VerificationComplete: 'Verification complete',
  Complete: 'Clone successful',
  HfProcessing: 'Processing HF card...',
  HfDumpReady: 'Dump ready',
  Error: 'Error',
};

// -- Step descriptions (shown below the label) --

export const STEP_DESCRIPTIONS: Record<WizardStepName, string> = {
  Idle: 'Connect your Proxmark3 to begin.',
  DetectingDevice: 'Looking for a Proxmark3 on USB ports.',
  CheckingFirmware: 'Comparing client and device firmware versions.',
  FirmwareOutdated: 'Device firmware does not match the bundled client version.',
  UpdatingFirmware: 'Flashing firmware to device. Do not disconnect.',
  RedetectingDevice: 'Looking for device after firmware update.',
  DeviceConnected: 'Place the card you want to copy on the reader.',
  ScanningCard: 'Detecting card type and reading data.',
  CardIdentified: 'Review the detected card information below.',
  WaitingForBlank: 'Remove the source card and place a writable blank card.',
  BlankDetected: 'Ready to write. Do not remove the card during this process.',
  Writing: 'Writing card data. Do not remove the card.',
  Verifying: 'Reading back to confirm the data was written correctly.',
  VerificationComplete: 'Review the verification results below.',
  Complete: 'The card has been cloned. You may remove it.',
  HfProcessing: 'Recovering encryption keys and dumping card memory.',
  HfDumpReady: 'Card data dumped. Swap to a blank magic card to write.',
  Error: 'Something went wrong. See details below.',
};

// -- Recovery action labels --

export const RECOVERY_LABELS: Record<RecoveryAction, string> = {
  Retry: 'Try again',
  GoBack: 'Go back',
  Reconnect: 'Reconnect device',
  Manual: 'Manual intervention needed',
};

// -- Terminal status symbols (ASCII, Matrix theme compatible) --

export const STATUS_SYMBOLS = {
  SUCCESS: '[OK]',
  FAILURE: '[FAIL]',
  PENDING: '[..]',
  RUNNING: '[>>]',
  WARNING: '[!!]',
  INFO: '[--]',
} as const;

// -- Frequency display labels --

export const FREQUENCY_LABELS = {
  LF: '125 kHz (LF)',
  HF: '13.56 MHz (HF)',
} as const;

// -- PM3 exit code descriptions --

export const PM3_EXIT_CODES: Record<number, string> = {
  0: 'Command completed successfully',
  [-1]: 'Command failed',
  [-5]: 'Command timed out',
  [-17]: 'Static nonce detected (card may be hardened)',
};

// -- Block progress formatting --

/**
 * Format write progress as a percentage string.
 */
export function formatProgress(progress: number): string {
  return `${Math.round(progress * 100)}%`;
}

/**
 * Format block progress as "current / total".
 */
export function formatBlockProgress(
  current: number | null,
  total: number | null,
): string {
  if (current === null || total === null) {
    return 'Calculating...';
  }
  return `Block ${current} / ${total}`;
}

/**
 * Build a terminal-style block progress bar.
 * Uses block characters for the Matrix terminal aesthetic.
 * Width is the character count of the bar (excluding brackets).
 */
export function buildProgressBar(progress: number, width: number = 30): string {
  const filled = Math.round(progress * width);
  const empty = width - filled;
  const bar = '\u2588'.repeat(filled) + '\u2591'.repeat(empty);
  return `[${bar}]`;
}
