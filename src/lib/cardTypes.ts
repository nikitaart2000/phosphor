// Display metadata for each supported card type.
// Maps internal CardType identifiers to human-readable names, descriptions,
// frequency bands, and recommended blank card types.

import type { CardType, BlankType, Frequency } from '../machines/types';

export interface CardDisplayInfo {
  displayName: string;
  frequency: Frequency;
  blankType: BlankType;
  description: string;
}

export const CARD_DISPLAY: Record<CardType, CardDisplayInfo> = {
  // -- LF cards (125 kHz) --
  EM4100: {
    displayName: 'EM4100 Access Card',
    frequency: 'LF',
    blankType: 'T5577',
    description: 'Standard 125 kHz proximity card, read-only, widely used in access control',
  },
  HIDProx: {
    displayName: 'HID Proximity Card',
    frequency: 'LF',
    blankType: 'T5577',
    description: '125 kHz HID proximity format, common in corporate access systems',
  },
  Indala: {
    displayName: 'Indala Proximity Card',
    frequency: 'LF',
    blankType: 'T5577',
    description: '125 kHz Indala/Motorola format, used in government and corporate facilities',
  },
  IOProx: {
    displayName: 'IO Prox Card',
    frequency: 'LF',
    blankType: 'T5577',
    description: '125 kHz Kantech IO Prox format',
  },
  AWID: {
    displayName: 'AWID Proximity Card',
    frequency: 'LF',
    blankType: 'T5577',
    description: '125 kHz AWID format, used in building access control',
  },
  FDX_B: {
    displayName: 'FDX-B Animal Tag',
    frequency: 'LF',
    blankType: 'T5577',
    description: '134.2 kHz FDX-B ISO 11784/11785 animal identification transponder',
  },
  Paradox: {
    displayName: 'Paradox Access Card',
    frequency: 'LF',
    blankType: 'T5577',
    description: '125 kHz Paradox format, used in security systems',
  },
  Viking: {
    displayName: 'Viking Access Card',
    frequency: 'LF',
    blankType: 'T5577',
    description: '125 kHz Viking format proximity card',
  },
  Pyramid: {
    displayName: 'Farpointe Pyramid Card',
    frequency: 'LF',
    blankType: 'T5577',
    description: '125 kHz Farpointe Pyramid format access card',
  },
  Keri: {
    displayName: 'Keri Access Card',
    frequency: 'LF',
    blankType: 'T5577',
    description: '125 kHz Keri Systems format proximity card',
  },
  NexWatch: {
    displayName: 'NexWatch Access Card',
    frequency: 'LF',
    blankType: 'T5577',
    description: '125 kHz Honeywell NexWatch format card',
  },

  // -- HF cards (13.56 MHz) --
  MifareClassic1K: {
    displayName: 'MIFARE Classic 1K',
    frequency: 'HF',
    blankType: 'MagicMifareGen1a',
    description: '13.56 MHz MIFARE Classic with 1 KB memory, 16 sectors',
  },
  MifareClassic4K: {
    displayName: 'MIFARE Classic 4K',
    frequency: 'HF',
    blankType: 'MagicMifareGen1a',
    description: '13.56 MHz MIFARE Classic with 4 KB memory, 40 sectors',
  },
  MifareUltralight: {
    displayName: 'MIFARE Ultralight',
    frequency: 'HF',
    blankType: 'MagicUltralight',
    description: '13.56 MHz MIFARE Ultralight, low-cost NFC tag',
  },
  NTAG: {
    displayName: 'NTAG NFC Tag',
    frequency: 'HF',
    blankType: 'MagicUltralight',
    description: '13.56 MHz NXP NTAG series (213/215/216), common in NFC applications',
  },
  DESFire: {
    displayName: 'MIFARE DESFire',
    frequency: 'HF',
    blankType: 'MagicMifareGen4GDM',
    description: '13.56 MHz MIFARE DESFire, AES/3DES encrypted, high-security applications',
  },
  IClass: {
    displayName: 'HID iCLASS',
    frequency: 'HF',
    blankType: 'IClassBlank',
    description: '13.56 MHz HID iCLASS/iCLASS SE smart card',
  },
};

// Blank card display info for the "place blank" step
export interface BlankDisplayInfo {
  displayName: string;
  frequency: Frequency;
  description: string;
  compatibleWith: CardType[];
}

export const BLANK_DISPLAY: Record<BlankType, BlankDisplayInfo> = {
  T5577: {
    displayName: 'T5577 Blank',
    frequency: 'LF',
    description: 'Writable 125 kHz transponder, emulates most LF card formats',
    compatibleWith: ['EM4100', 'HIDProx', 'Indala', 'IOProx', 'AWID', 'FDX_B', 'Paradox', 'Viking', 'Pyramid', 'Keri', 'NexWatch'],
  },
  EM4305: {
    displayName: 'EM4305 Blank',
    frequency: 'LF',
    description: 'Alternative 125 kHz writable transponder, use with --em flag',
    compatibleWith: ['EM4100', 'FDX_B'],
  },
  MagicMifareGen1a: {
    displayName: 'Magic MIFARE Gen1a',
    frequency: 'HF',
    description: 'UID-writable MIFARE Classic blank, backdoor commands supported',
    compatibleWith: ['MifareClassic1K', 'MifareClassic4K'],
  },
  MagicMifareGen2: {
    displayName: 'Magic MIFARE Gen2 (CUID)',
    frequency: 'HF',
    description: 'Direct-write MIFARE blank, no backdoor command needed',
    compatibleWith: ['MifareClassic1K', 'MifareClassic4K'],
  },
  MagicMifareGen3: {
    displayName: 'Magic MIFARE Gen3 (APDU)',
    frequency: 'HF',
    description: 'APDU-locked UID change, anti-detection features',
    compatibleWith: ['MifareClassic1K', 'MifareClassic4K'],
  },
  MagicMifareGen4GTU: {
    displayName: 'Magic MIFARE Gen4 GTU',
    frequency: 'HF',
    description: 'Ultimate magic card with password-protected configuration',
    compatibleWith: ['MifareClassic1K', 'MifareClassic4K'],
  },
  MagicMifareGen4GDM: {
    displayName: 'Magic MIFARE Gen4 GDM',
    frequency: 'HF',
    description: 'Gen4 with GDM write mode, supports DESFire emulation',
    compatibleWith: ['MifareClassic1K', 'MifareClassic4K', 'DESFire'],
  },
  MagicUltralight: {
    displayName: 'Magic Ultralight Blank',
    frequency: 'HF',
    description: 'UID-writable Ultralight/NTAG compatible blank',
    compatibleWith: ['MifareUltralight', 'NTAG'],
  },
  IClassBlank: {
    displayName: 'iCLASS Blank',
    frequency: 'HF',
    description: 'Writable iCLASS compatible credential',
    compatibleWith: ['IClass'],
  },
};

/**
 * Get display info for a card type, with fallback for unknown types.
 */
export function getCardDisplay(cardType: string): CardDisplayInfo {
  if (cardType in CARD_DISPLAY) {
    return CARD_DISPLAY[cardType as CardType];
  }
  return {
    displayName: cardType,
    frequency: 'LF',
    blankType: 'T5577',
    description: 'Unknown card type',
  };
}

/**
 * Get display info for a blank type, with fallback for unknown types.
 */
export function getBlankDisplay(blankType: string): BlankDisplayInfo {
  if (blankType in BLANK_DISPLAY) {
    return BLANK_DISPLAY[blankType as BlankType];
  }
  return {
    displayName: blankType,
    frequency: 'LF',
    description: 'Unknown blank type',
    compatibleWith: [],
  };
}
