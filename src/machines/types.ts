// Types mirroring Rust backend enums and structs for PM3 wizard flow.

export type Frequency = 'LF' | 'HF';

export type CardType =
  // LF cloneable (original 11)
  | 'EM4100'
  | 'HIDProx'
  | 'Indala'
  | 'IOProx'
  | 'AWID'
  | 'FDX_B'
  | 'Paradox'
  | 'Viking'
  | 'Pyramid'
  | 'Keri'
  | 'NexWatch'
  // LF cloneable (new 11)
  | 'Presco'
  | 'Nedap'
  | 'GProxII'
  | 'Gallagher'
  | 'PAC'
  | 'Noralsy'
  | 'Jablotron'
  | 'SecuraKey'
  | 'Visa2000'
  | 'Motorola'
  | 'IDTECK'
  // LF non-cloneable (display only)
  | 'COTAG'
  | 'EM4x50'
  | 'Hitag'
  // HF cards
  | 'MifareClassic1K'
  | 'MifareClassic4K'
  | 'MifareUltralight'
  | 'NTAG'
  | 'DESFire'
  | 'IClass';

export type BlankType =
  | 'T5577'
  | 'EM4305'
  | 'MagicMifareGen1a'
  | 'MagicMifareGen2'
  | 'MagicMifareGen3'
  | 'MagicMifareGen4GTU'
  | 'MagicMifareGen4GDM'
  | 'MagicUltralight'
  | 'IClassBlank';

export type RecoveryAction = 'Retry' | 'GoBack' | 'Reconnect' | 'Manual';

// Matches Rust ProcessPhase enum — autopwn attack phases
export type ProcessPhase = 'KeyCheck' | 'Darkside' | 'Nested' | 'Hardnested' | 'StaticNested' | 'Dumping';

// HF progress event payload emitted by Rust during autopwn/dump
export interface HfProgressPayload {
  phase: string;
  keys_found: number;
  keys_total: number;
  elapsed_secs: number;
}

export interface CardData {
  uid: string;
  raw: string;
  decoded: Record<string, string>;
}

export interface CardSummary {
  card_type: string;
  uid: string;
  display_name: string;
}

export interface CloneRecord {
  id: number | null;
  source_type: string;
  source_uid: string;
  target_type: string;
  target_uid: string;
  port: string;
  success: boolean;
  timestamp: string;
  notes: string | null;
}

// Device information returned on successful connection
export interface DeviceInfo {
  port: string;
  model: string;
  firmware: string;
}

// Card identification result from scanning
export interface CardIdentification {
  frequency: Frequency;
  card_type: CardType;
  card_data: CardData;
  cloneable: boolean;
  recommended_blank: BlankType;
}

// Write progress during clone operation
export interface WriteProgress {
  progress: number;
  current_block: number | null;
  total_blocks: number | null;
}

// Verification result after clone
export interface VerificationResult {
  success: boolean;
  mismatched_blocks: number[];
}

// Clone completion summary
export interface CloneCompletion {
  source: CardSummary;
  target: CardSummary;
  timestamp: string;
}

// Error details from any failed operation
export interface ErrorDetails {
  message: string;
  user_message: string;
  recoverable: boolean;
  recovery_action: RecoveryAction | null;
}

// Tagged union matching Rust's serde output for the full wizard state
export type WizardState =
  | { step: 'Idle' }
  | { step: 'DetectingDevice' }
  | { step: 'DeviceConnected'; data: DeviceInfo }
  | { step: 'ScanningCard' }
  | { step: 'CardIdentified'; data: CardIdentification }
  | { step: 'WaitingForBlank'; data: { expected_blank: BlankType } }
  | { step: 'BlankDetected'; data: { blank_type: BlankType; ready_to_write: boolean; existing_data_type: string | null } }
  | { step: 'Writing'; data: WriteProgress }
  | { step: 'HfProcessing'; data: { phase: string; keys_found: number; keys_total: number; elapsed_secs: number } }
  | { step: 'HfDumpReady'; data: { dump_info: string } }
  | { step: 'Verifying' }
  | { step: 'VerificationComplete'; data: VerificationResult }
  | { step: 'Complete'; data: CloneCompletion }
  | { step: 'Error'; data: ErrorDetails };

// Extract the step name as a string literal type
// Includes Rust-derived steps + frontend-only firmware states
export type WizardStepName = WizardState['step']
  | 'CheckingFirmware'
  | 'FirmwareOutdated'
  | 'UpdatingFirmware'
  | 'RedetectingDevice'
  | 'HfProcessing'
  | 'HfDumpReady';

// Helper: extract data type for a given step
export type StepData<S extends WizardStepName> = Extract<WizardState, { step: S }> extends { data: infer D } ? D : never;

// Firmware check result from Rust backend (camelCase from serde rename_all)
export interface FirmwareCheckResult {
  matched: boolean;
  clientVersion: string;
  deviceFirmwareVersion: string;
  hardwareVariant: 'rdv4' | 'rdv4-bt' | 'generic' | 'generic-256' | 'unknown';
  firmwarePathExists: boolean;
}

// Firmware flash progress event payload (emitted via Tauri events)
export interface FirmwareProgress {
  phase: 'connecting' | 'erasing' | 'writing' | 'done' | 'error';
  percent: number;
  message: string;
}

// A saved card record stored in the local database
export interface SavedCard {
  name: string;
  cardType: string;
  frequency: string;
  uid: string;
  raw: string;
  decoded: string;
  cloneable: boolean;
  recommendedBlank: string;
  createdAt: string;
}

// T5577 chip status for password detection and safety workflow
export interface T5577Status {
  detected: boolean;
  chip_type: string;
  password_set: boolean;
  block0: string | null;
  modulation: string | null;
}
