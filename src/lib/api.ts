// Typed Tauri invoke wrappers for PM3 backend commands.

import { invoke } from '@tauri-apps/api/core';
import type { WizardState, CloneRecord } from '../machines/types';

/**
 * Detect connected Proxmark3 device.
 * Triggers hardware scan and returns updated wizard state.
 */
export async function detectDevice(): Promise<WizardState> {
  return invoke<WizardState>('detect_device');
}

/**
 * Get the current wizard state from the backend.
 * Used for polling and state synchronization.
 */
export async function getWizardState(): Promise<WizardState> {
  return invoke<WizardState>('get_wizard_state');
}

/**
 * Scan a card on the connected device.
 * Identifies card type, frequency, and reads data.
 */
export async function scanCard(): Promise<WizardState> {
  return invoke<WizardState>('scan_card');
}

/**
 * Detect blank card on reader.
 * Runs lf t55xx detect (for T5577) or lf em 4x05 info (for EM4305) on the backend.
 */
export async function detectBlank(port: string): Promise<WizardState> {
  return invoke<WizardState>('detect_blank', { port });
}

/**
 * Execute the clone write operation with full card context.
 * Writes source card data to the blank card.
 */
export async function writeCloneWithData(
  port: string,
  cardType: string,
  uid: string,
  decoded: Record<string, string>,
  blankType?: string,
): Promise<WizardState> {
  return invoke<WizardState>('write_clone_with_data', {
    port,
    card_type: cardType,
    uid,
    decoded,
    blank_type: blankType,
  });
}

/**
 * Verify the written clone against source data.
 * Reads back the blank and compares block-by-block.
 * Pass sourceDecoded to enable field-by-field verification.
 */
export async function verifyCloneWithData(
  port: string,
  sourceUid: string,
  sourceCardType: string,
  sourceDecoded?: Record<string, string>,
): Promise<WizardState> {
  return invoke<WizardState>('verify_clone', {
    port,
    source_uid: sourceUid,
    source_card_type: sourceCardType,
    source_decoded: sourceDecoded,
  });
}

/**
 * Retrieve clone history from the local database.
 * Returns all past clone operations with metadata.
 */
export async function getHistory(): Promise<CloneRecord[]> {
  return invoke<CloneRecord[]>('get_history');
}

/**
 * Reset the wizard to idle state via wizard_action Reset.
 * Clears all in-progress operation data on the backend.
 */
export async function resetWizard(): Promise<WizardState> {
  return invoke<WizardState>('wizard_action', {
    action: { action: 'Reset' },
  });
}
