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
    // LF types
    EM4100,
    HIDProx,
    Indala,
    IOProx,
    AWID,
    #[serde(rename = "FDX-B")]
    FDX_B,
    Paradox,
    Viking,
    Pyramid,
    Keri,
    NexWatch,
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
            | CardType::NexWatch => Frequency::LF,

            CardType::MifareClassic1K
            | CardType::MifareClassic4K
            | CardType::MifareUltralight
            | CardType::NTAG
            | CardType::DESFire
            | CardType::IClass => Frequency::HF,
        }
    }

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
            _ => true,
        }
    }

    pub fn recommended_blank(&self) -> BlankType {
        match self {
            CardType::EM4100 => BlankType::T5577,
            CardType::HIDProx => BlankType::T5577,
            CardType::Indala => BlankType::T5577,
            CardType::IOProx => BlankType::T5577,
            CardType::AWID => BlankType::T5577,
            CardType::FDX_B => BlankType::T5577,
            CardType::Paradox => BlankType::T5577,
            CardType::Viking => BlankType::T5577,
            CardType::Pyramid => BlankType::T5577,
            CardType::Keri => BlankType::T5577,
            CardType::NexWatch => BlankType::T5577,
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

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum ProcessPhase {
    KeyCheck,
    Darkside,
    Nested,
    Hardnested,
    StaticNested,
    Dumping,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum MagicGeneration {
    Gen1a,
    Gen2,
    Gen3,
    Gen4GTU,
    Gen4GDM,
}
