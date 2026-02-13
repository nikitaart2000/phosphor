use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum Frequency {
    LF,
    HF,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[allow(non_camel_case_types)]
pub enum CardType {
    // LF cloneable types (22 total)
    EM4100,
    HIDProx,
    Indala,
    IOProx,
    AWID,
    FDX_B,
    Paradox,
    Viking,
    Pyramid,
    Keri,
    NexWatch,
    Presco,
    Nedap,
    GProxII,
    Gallagher,
    PAC,
    Noralsy,
    Jablotron,
    SecuraKey,
    Visa2000,
    Motorola,
    IDTECK,
    // LF non-cloneable types (3)
    COTAG,
    EM4x50,
    Hitag,
    // HF types
    MifareClassic1K,
    MifareClassic4K,
    MifareUltralight,
    NTAG,
    DESFire,
    IClass,
}

impl CardType {
    pub fn frequency(&self) -> Frequency {
        match self {
            CardType::EM4100
            | CardType::HIDProx
            | CardType::Indala
            | CardType::IOProx
            | CardType::AWID
            | CardType::FDX_B
            | CardType::Paradox
            | CardType::Viking
            | CardType::Pyramid
            | CardType::Keri
            | CardType::NexWatch
            | CardType::Presco
            | CardType::Nedap
            | CardType::GProxII
            | CardType::Gallagher
            | CardType::PAC
            | CardType::Noralsy
            | CardType::Jablotron
            | CardType::SecuraKey
            | CardType::Visa2000
            | CardType::Motorola
            | CardType::IDTECK
            | CardType::COTAG
            | CardType::EM4x50
            | CardType::Hitag => Frequency::LF,

            CardType::MifareClassic1K
            | CardType::MifareClassic4K
            | CardType::MifareUltralight
            | CardType::NTAG
            | CardType::DESFire
            | CardType::IClass => Frequency::HF,
        }
    }

    /// Human-readable name for UI display. Used by frontend via serde serialization.
    #[allow(dead_code)]
    pub fn display_name(&self) -> &str {
        match self {
            CardType::EM4100 => "EM4100",
            CardType::HIDProx => "HID Prox",
            CardType::Indala => "Indala",
            CardType::IOProx => "IO Prox",
            CardType::AWID => "AWID",
            CardType::FDX_B => "FDX-B",
            CardType::Paradox => "Paradox",
            CardType::Viking => "Viking",
            CardType::Pyramid => "Pyramid",
            CardType::Keri => "Keri",
            CardType::NexWatch => "NexWatch",
            CardType::Presco => "Presco",
            CardType::Nedap => "Nedap",
            CardType::GProxII => "GProx II",
            CardType::Gallagher => "Gallagher",
            CardType::PAC => "PAC/Stanley",
            CardType::Noralsy => "Noralsy",
            CardType::Jablotron => "Jablotron",
            CardType::SecuraKey => "SecuraKey",
            CardType::Visa2000 => "Visa2000",
            CardType::Motorola => "Motorola",
            CardType::IDTECK => "IDTECK",
            CardType::COTAG => "COTAG",
            CardType::EM4x50 => "EM4x50",
            CardType::Hitag => "Hitag",
            CardType::MifareClassic1K => "MIFARE Classic 1K",
            CardType::MifareClassic4K => "MIFARE Classic 4K",
            CardType::MifareUltralight => "MIFARE Ultralight",
            CardType::NTAG => "NTAG",
            CardType::DESFire => "DESFire",
            CardType::IClass => "iCLASS",
        }
    }

    pub fn is_cloneable(&self) -> bool {
        match self {
            CardType::DESFire => false,
            CardType::COTAG => false,
            CardType::EM4x50 => false,
            CardType::Hitag => false,
            _ => true,
        }
    }

    /// Reason why a card type cannot be cloned, if applicable.
    /// Provided for frontend display on non-cloneable card identification.
    #[allow(dead_code)]
    pub fn non_cloneable_reason(&self) -> Option<&str> {
        match self {
            CardType::DESFire => Some("DESFire uses AES encryption; cloning not supported"),
            CardType::COTAG => Some("Read-only, no clone commands available"),
            CardType::EM4x50 => Some("Requires native EM4x50 blank, not T5577-compatible"),
            CardType::Hitag => Some("Requires native Hitag chip, not T5577-compatible"),
            _ => None,
        }
    }

    /// Whether this card type supports cloning to EM4305 blanks (via `--em` flag).
    /// Only the original 11 LF types support this flag in Proxmark3 Iceman firmware.
    /// The 11 newer types (Presco, Nedap, GProxII, Gallagher, PAC, Noralsy,
    /// Jablotron, SecuraKey, Visa2000, Motorola, IDTECK) do not support `--em`
    /// and will fail silently or error if the flag is passed.
    pub fn supports_em4305(&self) -> bool {
        matches!(
            self,
            CardType::EM4100
                | CardType::HIDProx
                | CardType::Indala
                | CardType::IOProx
                | CardType::AWID
                | CardType::FDX_B
                | CardType::Paradox
                | CardType::Viking
                | CardType::Pyramid
                | CardType::Keri
                | CardType::NexWatch
        )
    }

    pub fn recommended_blank(&self) -> BlankType {
        match self {
            // All LF cloneable types use T5577 by default
            CardType::EM4100
            | CardType::HIDProx
            | CardType::Indala
            | CardType::IOProx
            | CardType::AWID
            | CardType::FDX_B
            | CardType::Paradox
            | CardType::Viking
            | CardType::Pyramid
            | CardType::Keri
            | CardType::NexWatch
            | CardType::Presco
            | CardType::Nedap
            | CardType::GProxII
            | CardType::Gallagher
            | CardType::PAC
            | CardType::Noralsy
            | CardType::Jablotron
            | CardType::SecuraKey
            | CardType::Visa2000
            | CardType::Motorola
            | CardType::IDTECK => BlankType::T5577,
            // Non-cloneable LF: return T5577 as placeholder (won't actually be used)
            CardType::COTAG | CardType::EM4x50 | CardType::Hitag => BlankType::T5577,
            // HF types
            CardType::MifareClassic1K | CardType::MifareClassic4K => BlankType::MagicMifareGen1a,
            CardType::MifareUltralight => BlankType::MagicUltralight,
            CardType::NTAG => BlankType::MagicUltralight,
            CardType::DESFire => BlankType::MagicMifareGen4GTU,
            CardType::IClass => BlankType::IClassBlank,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum BlankType {
    T5577,
    EM4305,
    MagicMifareGen1a,
    MagicMifareGen2,
    MagicMifareGen3,
    MagicMifareGen4GTU,
    MagicMifareGen4GDM,
    MagicUltralight,
    IClassBlank,
}

impl BlankType {
    /// Human-readable name for UI display. Used by frontend via serde serialization.
    #[allow(dead_code)]
    pub fn display_name(&self) -> &str {
        match self {
            BlankType::T5577 => "T5577",
            BlankType::EM4305 => "EM4305",
            BlankType::MagicMifareGen1a => "Magic MIFARE Gen1a",
            BlankType::MagicMifareGen2 => "Magic MIFARE Gen2 (CUID)",
            BlankType::MagicMifareGen3 => "Magic MIFARE Gen3 (UFUID)",
            BlankType::MagicMifareGen4GTU => "Magic MIFARE Gen4 GTU",
            BlankType::MagicMifareGen4GDM => "Magic MIFARE Gen4 GDM",
            BlankType::MagicUltralight => "Magic Ultralight",
            BlankType::IClassBlank => "iCLASS Blank",
        }
    }
}

/// T5577 chip detection result from `lf t55xx detect`.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct T5577Status {
    pub detected: bool,
    pub chip_type: String,
    pub password_set: bool,
    pub block0: Option<String>,
    pub modulation: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CardData {
    pub uid: String,
    pub raw: String,
    pub decoded: HashMap<String, String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CardSummary {
    pub card_type: String,
    pub uid: String,
    pub display_name: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum RecoveryAction {
    Retry,
    GoBack,
    Reconnect,
    Manual,
}

/// HF card processing phases for autopwn progress tracking.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum ProcessPhase {
    KeyCheck,
    Darkside,
    Nested,
    Hardnested,
    StaticNested,
    Dumping,
}

/// Events parsed line-by-line from `hf mf autopwn` streaming output.
#[derive(Clone, Debug, PartialEq)]
pub enum AutopwnEvent {
    /// Dictionary attack progress: found N of M keys, method char (D=dict)
    DictionaryProgress { found: u32, total: u32 },
    /// Individual key recovered
    KeyFound { key: String },
    /// Darkside attack started (0 known keys + WEAK PRNG)
    DarksideStarted,
    /// Nested attack started (1+ keys + WEAK PRNG)
    NestedStarted,
    /// Hardnested attack started (1+ keys + HARD PRNG)
    HardnestedStarted,
    /// Staticnested attack started (1+ keys + STATIC PRNG)
    StaticnestedStarted,
    /// All blocks dumped successfully
    DumpComplete { file_path: String },
    /// Partial dump (some sectors unreadable)
    DumpPartial { file_path: String },
    /// All key recovery attempts failed
    Failed { reason: String },
    /// Autopwn finished with execution time
    Finished { time_secs: u32 },
}

/// Magic card generation identifiers (reserved for Phase 3: HF card support).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[allow(dead_code)]
pub enum MagicGeneration {
    Gen1a,
    Gen2,
    Gen3,
    Gen4GTU,
    Gen4GDM,
}
