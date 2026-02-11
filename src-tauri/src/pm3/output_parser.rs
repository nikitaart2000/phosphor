use regex::Regex;
use std::collections::HashMap;
use std::sync::LazyLock;

use crate::cards::types::{CardData, CardType, T5577Status};

// ---------------------------------------------------------------------------
// ANSI stripping
// ---------------------------------------------------------------------------

static ANSI_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\x1b\[[0-9;]*[mGKHJ]").expect("bad ansi regex"));

pub fn strip_ansi(input: &str) -> String {
    ANSI_RE.replace_all(input, "").to_string()
}

// ---------------------------------------------------------------------------
// Regex patterns — original types (improved)
// ---------------------------------------------------------------------------

static EM4100_ID_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"EM 410x ID\s*[\-:]\s*([0-9A-Fa-f]{10})").expect("bad em regex"));

static HID_FC_CN_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)FC[:/\s]*(\d+)\s*[,;]?\s*CN[:/\s]*(\d+)").expect("bad hid regex")
});

static HID_RAW_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)(?:HID|Prox).*?RAW[:/\s]*([0-9A-Fa-f]+)").expect("bad hid raw regex"));

static HID_FORMAT_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)(?:H10301|H10302|H10304|Corp\s*1000|26[- ]?bit|34[- ]?bit|35[- ]?bit|37[- ]?bit)")
        .expect("bad hid format regex")
});

static INDALA_RAW_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)Indala.*?Raw[:/\s]*([0-9A-Fa-f]+)").expect("bad indala raw regex")
});

static INDALA_UID_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)Indala.*?ID[:/\s]*([0-9A-Fa-f]+)").expect("bad indala uid regex")
});

static IOPROX_FC_CN_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)IO\s*Prox.*?(?:VN[:/\s]*(\d+))?.*?FC[:/\s]*(\d+).*?CN[:/\s]*(\d+)")
        .expect("bad ioprox fc/cn regex")
});

static IOPROX_RAW_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)IO\s*Prox.*?(?:ID|Raw)[:/\s]*([0-9A-Fa-f]+)").expect("bad ioprox regex")
});

static AWID_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)AWID.*?FC[:/\s]*(\d+)\s*[,;]?\s*CN[:/\s]*(\d+)").expect("bad awid regex")
});

static AWID_FMT_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)AWID.*?(\d+)\s*bit").expect("bad awid fmt regex")
});

static FDXB_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)FDX-?B.*?Country[:/\s]*(\d+).*?(?:National|ID)[:/\s]*(\d+)")
        .expect("bad fdxb regex")
});

static PYRAMID_FC_CN_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)Pyramid.*?FC[:/\s]*(\d+).*?Card[:/\s]*(\d+)")
        .expect("bad pyramid fc/cn regex")
});

static PYRAMID_RAW_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)Pyramid.*?Raw[:/\s]*([0-9A-Fa-f]+)").expect("bad pyramid raw regex")
});

static PARADOX_FC_CN_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)Paradox.*?FC[:/\s]*(\d+).*?(?:Card|CN)[:/\s]*(\d+)")
        .expect("bad paradox fc/cn regex")
});

static PARADOX_RAW_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)Paradox.*?Raw[:/\s]*([0-9A-Fa-f]+)").expect("bad paradox raw regex")
});

static KERI_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)Keri.*?(?:Internal|MS|Raw)[:/\s]*([0-9A-Fa-f]+)")
        .expect("bad keri regex")
});

static KERI_TYPE_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)Keri.*?(Internal|MS)").expect("bad keri type regex")
});

// ---------------------------------------------------------------------------
// Regex patterns — 11 new types
// ---------------------------------------------------------------------------

static PRESCO_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)Presco.*?Card[:/\s]*([0-9A-Fa-f]+)").expect("bad presco regex")
});

static PRESCO_SC_UC_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)Presco.*?Site[:/\s]*(\d+).*?User[:/\s]*(\d+)")
        .expect("bad presco sc/uc regex")
});

static NEDAP_CARD_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)Nedap.*?Card[:/\s]*(\d+)").expect("bad nedap card regex")
});

static NEDAP_SUB_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)Nedap.*?Sub(?:type)?[:/\s]*(\d+)").expect("bad nedap sub regex")
});

static GPROXII_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)(?:Guardall|GProx\s*II).*?(?:XSF|FC)[:/\s]*(\d+).*?(?:Card|CN)[:/\s]*(\d+)")
        .expect("bad gproxii regex")
});

static GALLAGHER_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?i)Gallagher.*?Region\s+Code[:/\s]*(\d+).*?Facility\s+Code[:/\s]*(\d+).*?Card\s+Number[:/\s]*(\d+).*?Issue\s+Level[:/\s]*(\d+)",
    )
    .expect("bad gallagher regex")
});

// Per-field Gallagher regexes — fallback for multi-line PM3 output
static GALLAGHER_RC_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)Region\s+Code[:/\s]*(\d+)").expect("bad gallagher rc regex")
});
static GALLAGHER_FC_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)Facility\s+Code[:/\s]*(\d+)").expect("bad gallagher fc regex")
});
static GALLAGHER_CN_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)Card\s+Number[:/\s]*(\d+)").expect("bad gallagher cn regex")
});
static GALLAGHER_IL_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)Issue\s+Level[:/\s]*(\d+)").expect("bad gallagher il regex")
});

static PAC_DETECT_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)\[\+\].*\b(?:PAC|Stanley)\b").expect("bad pac detect regex")
});

static PAC_CN_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)PAC(?:/Stanley)?.*?Card[:/\s]*(\d+)").expect("bad pac cn regex")
});

static PAC_RAW_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)PAC(?:/Stanley)?.*?Raw[:/\s]*([0-9A-Fa-f]+)").expect("bad pac raw regex")
});

static NORALSY_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)Noralsy.*?Card[:/\s]*(\d+)(?:.*?Year[:/\s]*(\d+))?")
        .expect("bad noralsy regex")
});

static NORALSY_RAW_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)Noralsy.*?Raw[:/\s]*([0-9A-Fa-f]+)").expect("bad noralsy raw regex")
});

static JABLOTRON_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)Jablotron.*?Card[:/\s]*([0-9A-Fa-f]+)").expect("bad jablotron regex")
});

static SECURAKEY_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)Secura\s*[Kk]ey.*?Raw[:/\s]+([0-9A-Fa-f]+)").expect("bad securakey regex")
});

static VISA2000_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)Visa2000.*?Card[:/\s]*(\d+)").expect("bad visa2000 regex")
});

static MOTOROLA_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)Motorola.*?Raw[:/\s]*([0-9A-Fa-f]+)").expect("bad motorola regex")
});

static IDTECK_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)IDTECK.*?Raw[:/\s]*([0-9A-Fa-f]+)").expect("bad idteck regex")
});

static NEXWATCH_ID_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)(?:NexWatch|NXT)\s*ID[:/\s]*(\d+)").expect("bad nexwatch id regex")
});

static NEXWATCH_RAW_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)(?:NexWatch|NXT).*?Raw[:/\s]*([0-9A-Fa-f]+)")
        .expect("bad nexwatch raw regex")
});

static VIKING_ID_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)Viking\s*(?:ID|Card\s*ID)[:/\s]*(\d+)").expect("bad viking id regex")
});

static VIKING_RAW_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)Viking.*?Raw[:/\s]*([0-9A-Fa-f]+)").expect("bad viking raw regex")
});

// ---------------------------------------------------------------------------
// Non-cloneable LF detection patterns
// ---------------------------------------------------------------------------

static COTAG_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)\[\+\].*COTAG").expect("bad cotag regex")
});

static EM4X50_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)\[\+\].*EM4x50").expect("bad em4x50 regex")
});

static HITAG_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)\[\+\].*Hitag").expect("bad hitag regex")
});

// ---------------------------------------------------------------------------
// Valid tag fallback
// ---------------------------------------------------------------------------

static VALID_TAG_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\[\+\]\s*Valid\s+(\S+)\s+.*?found").expect("bad valid tag regex")
});

// ---------------------------------------------------------------------------
// T5577 detect patterns
// ---------------------------------------------------------------------------

static T5577_CHIP_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)Chip\s*(?:type)?\.+\s*(T55x7|T5555|T5577)").expect("bad t5577 chip regex")
});

static T5577_PASSWORD_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)Password\s*(?:set)?\.+\s*(Yes|No)").expect("bad t5577 password regex")
});

static T5577_BLOCK0_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)Block0\.+\s*([0-9A-Fa-f]{8})").expect("bad t5577 block0 regex")
});

static T5577_MOD_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)Modulation\.+\s*(.+)").expect("bad t5577 modulation regex")
});

static T5577_PASSWORD_FOUND_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)\[\+\]\s*(?:Found valid )?[Pp]assword[:\s]+([0-9A-Fa-f]{8})")
        .expect("bad t5577 password found regex")
});

// ---------------------------------------------------------------------------
// Main parse_lf_search
// ---------------------------------------------------------------------------

pub fn parse_lf_search(output: &str) -> Option<(CardType, CardData)> {
    let clean = strip_ansi(output);

    // Check for no-card condition first
    if clean.contains("No known 125/134 kHz tags found") {
        return None;
    }

    // EM4100
    if clean.contains("EM410x") || clean.contains("EM 410x") {
        if let Some(caps) = EM4100_ID_RE.captures(&clean) {
            let uid = caps[1].to_uppercase();
            let mut decoded = HashMap::new();
            decoded.insert("type".to_string(), "EM4100".to_string());
            decoded.insert("id".to_string(), uid.clone());
            return Some((
                CardType::EM4100,
                CardData {
                    uid: uid.clone(),
                    raw: uid,
                    decoded,
                },
            ));
        }
    }

    // HID Prox
    if clean.contains("HID Prox") || clean.contains("HID Corporate") {
        return parse_hid(&clean);
    }

    // Indala
    if clean.contains("Indala") {
        let raw_hex = INDALA_RAW_RE.captures(&clean).map(|c| c[1].to_uppercase());
        let uid_val = INDALA_UID_RE.captures(&clean).map(|c| c[1].to_uppercase());

        if let Some(ref raw) = raw_hex {
            let uid = uid_val.as_deref().unwrap_or(raw).to_string();
            let mut decoded = HashMap::new();
            decoded.insert("type".to_string(), "Indala".to_string());
            decoded.insert("raw".to_string(), raw.clone());
            decoded.insert("id".to_string(), uid.clone());
            return Some((
                CardType::Indala,
                CardData {
                    uid,
                    raw: raw.clone(),
                    decoded,
                },
            ));
        } else if let Some(uid) = uid_val {
            // No raw available — use UID as fallback (may be hex ID)
            let mut decoded = HashMap::new();
            decoded.insert("type".to_string(), "Indala".to_string());
            decoded.insert("id".to_string(), uid.clone());
            return Some((
                CardType::Indala,
                CardData {
                    uid: uid.clone(),
                    raw: uid,
                    decoded,
                },
            ));
        }
    }

    // IO Prox — improved with FC/CN/VN parsing
    if clean.contains("IO Prox") {
        return parse_ioprox(&clean);
    }

    // AWID — improved with format parsing
    if clean.contains("AWID") {
        return parse_awid(&clean);
    }

    // FDX-B — improved with country/national ID
    if clean.contains("FDX-B") || clean.contains("FDX B") || clean.contains("FDXB") {
        return parse_fdxb(&clean);
    }

    // Paradox — improved with FC/CN
    if clean.contains("Paradox") {
        return parse_paradox(&clean);
    }

    // Keri — improved with type detection
    if clean.contains("Keri") {
        return parse_keri(&clean);
    }

    // Pyramid — dedicated parser for FC/CN extraction
    if clean.contains("Pyramid") {
        return parse_pyramid(&clean);
    }

    // --- New card types (check before generic fallback) ---

    // Gallagher
    if clean.contains("Gallagher") {
        // Fast path: single-line regex with all 4 fields
        if let Some(caps) = GALLAGHER_RE.captures(&clean) {
            let rc = caps[1].to_string();
            let fc = caps[2].to_string();
            let cn = caps[3].to_string();
            let il = caps[4].to_string();
            let uid = format!("RC{}:FC{}:CN{}:IL{}", rc, fc, cn, il);
            let mut decoded = HashMap::new();
            decoded.insert("type".to_string(), "Gallagher".to_string());
            decoded.insert("region_code".to_string(), rc);
            decoded.insert("facility_code".to_string(), fc);
            decoded.insert("card_number".to_string(), cn);
            decoded.insert("issue_level".to_string(), il);
            return Some((
                CardType::Gallagher,
                CardData {
                    uid,
                    raw: String::new(),
                    decoded,
                },
            ));
        }
        // Fallback: per-field regexes for multi-line PM3 output (order-independent)
        let rc = GALLAGHER_RC_RE.captures(&clean).map(|c| c[1].to_string());
        let fc = GALLAGHER_FC_RE.captures(&clean).map(|c| c[1].to_string());
        let cn = GALLAGHER_CN_RE.captures(&clean).map(|c| c[1].to_string());
        let il = GALLAGHER_IL_RE.captures(&clean).map(|c| c[1].to_string());
        if let (Some(rc), Some(fc), Some(cn), Some(il)) = (rc, fc, cn, il) {
            let uid = format!("RC{}:FC{}:CN{}:IL{}", rc, fc, cn, il);
            let mut decoded = HashMap::new();
            decoded.insert("type".to_string(), "Gallagher".to_string());
            decoded.insert("region_code".to_string(), rc);
            decoded.insert("facility_code".to_string(), fc);
            decoded.insert("card_number".to_string(), cn);
            decoded.insert("issue_level".to_string(), il);
            return Some((
                CardType::Gallagher,
                CardData {
                    uid,
                    raw: String::new(),
                    decoded,
                },
            ));
        }
        // Raw hex fallback — card detected but regex didn't match firmware output format.
        // Without structured fields, command_builder cannot build a clone command.
        if let Some(hex) = extract_first_hex_block(&clean) {
            let mut decoded = HashMap::new();
            decoded.insert("type".to_string(), "Gallagher".to_string());
            decoded.insert("raw_fallback".to_string(), "true".to_string());
            return Some((
                CardType::Gallagher,
                CardData {
                    uid: hex.clone(),
                    raw: hex,
                    decoded,
                },
            ));
        }
    }

    // GProxII (appears as "Guardall" or "GProx II" in PM3 output)
    if clean.contains("Guardall") || clean.contains("GProx") {
        if let Some(caps) = GPROXII_RE.captures(&clean) {
            let xsf = caps[1].to_string();
            let cn = caps[2].to_string();
            let uid = format!("XSF{}:CN{}", xsf, cn);
            let mut decoded = HashMap::new();
            decoded.insert("type".to_string(), "GProxII".to_string());
            decoded.insert("xsf".to_string(), xsf);
            decoded.insert("card_number".to_string(), cn);
            return Some((
                CardType::GProxII,
                CardData {
                    uid,
                    raw: String::new(),
                    decoded,
                },
            ));
        }
        // Raw hex fallback — card detected but regex didn't match firmware output format.
        // Without structured fields, command_builder cannot build a clone command.
        if let Some(hex) = extract_first_hex_block(&clean) {
            let mut decoded = HashMap::new();
            decoded.insert("type".to_string(), "GProxII".to_string());
            decoded.insert("raw_fallback".to_string(), "true".to_string());
            return Some((
                CardType::GProxII,
                CardData {
                    uid: hex.clone(),
                    raw: hex,
                    decoded,
                },
            ));
        }
    }

    // Nedap
    if clean.contains("Nedap") || clean.contains("NEDAP") {
        let cn = NEDAP_CARD_RE.captures(&clean).map(|c| c[1].to_string());
        let st = NEDAP_SUB_RE.captures(&clean).map(|c| c[1].to_string());
        if let Some(cn) = cn {
            let st = st.unwrap_or_else(|| "0".to_string());
            let uid = format!("ST{}:CN{}", st, cn);
            let mut decoded = HashMap::new();
            decoded.insert("type".to_string(), "Nedap".to_string());
            decoded.insert("subtype".to_string(), st);
            decoded.insert("card_number".to_string(), cn);
            return Some((
                CardType::Nedap,
                CardData {
                    uid,
                    raw: String::new(),
                    decoded,
                },
            ));
        }
        // Raw hex fallback — card detected but regex didn't match firmware output format.
        // Without structured fields, command_builder cannot build a clone command.
        if let Some(hex) = extract_first_hex_block(&clean) {
            let mut decoded = HashMap::new();
            decoded.insert("type".to_string(), "Nedap".to_string());
            decoded.insert("raw_fallback".to_string(), "true".to_string());
            return Some((
                CardType::Nedap,
                CardData {
                    uid: hex.clone(),
                    raw: hex,
                    decoded,
                },
            ));
        }
    }

    // Presco
    if clean.contains("Presco") {
        return parse_presco(&clean);
    }

    // PAC/Stanley
    if PAC_DETECT_RE.is_match(&clean) {
        return parse_pac(&clean);
    }

    // Noralsy
    if clean.contains("Noralsy") {
        return parse_noralsy(&clean);
    }

    // Jablotron
    if clean.contains("Jablotron") {
        if let Some(caps) = JABLOTRON_RE.captures(&clean) {
            let cn = caps[1].to_uppercase();
            let mut decoded = HashMap::new();
            decoded.insert("type".to_string(), "Jablotron".to_string());
            decoded.insert("card_number".to_string(), cn.clone());
            return Some((
                CardType::Jablotron,
                CardData {
                    uid: cn.clone(),
                    raw: cn,
                    decoded,
                },
            ));
        }
    }

    // SecuraKey
    if clean.contains("Securakey") || clean.contains("SecuraKey") || clean.contains("SECURAKEY") {
        if let Some(caps) = SECURAKEY_RE.captures(&clean) {
            let raw = caps[1].to_uppercase();
            let mut decoded = HashMap::new();
            decoded.insert("type".to_string(), "SecuraKey".to_string());
            decoded.insert("raw".to_string(), raw.clone());
            return Some((
                CardType::SecuraKey,
                CardData {
                    uid: raw.clone(),
                    raw,
                    decoded,
                },
            ));
        }
    }

    // Visa2000
    if clean.contains("Visa2000") {
        if let Some(caps) = VISA2000_RE.captures(&clean) {
            let cn = caps[1].to_string();
            let mut decoded = HashMap::new();
            decoded.insert("type".to_string(), "Visa2000".to_string());
            decoded.insert("card_number".to_string(), cn.clone());
            return Some((
                CardType::Visa2000,
                CardData {
                    uid: cn.clone(),
                    raw: String::new(),
                    decoded,
                },
            ));
        }
    }

    // Motorola
    if clean.contains("Motorola") {
        if let Some(caps) = MOTOROLA_RE.captures(&clean) {
            let raw = caps[1].to_uppercase();
            let mut decoded = HashMap::new();
            decoded.insert("type".to_string(), "Motorola".to_string());
            decoded.insert("raw".to_string(), raw.clone());
            return Some((
                CardType::Motorola,
                CardData {
                    uid: raw.clone(),
                    raw,
                    decoded,
                },
            ));
        }
    }

    // IDTECK
    if clean.contains("IDTECK") || clean.contains("Idteck") {
        if let Some(caps) = IDTECK_RE.captures(&clean) {
            let raw = caps[1].to_uppercase();
            let mut decoded = HashMap::new();
            decoded.insert("type".to_string(), "IDTECK".to_string());
            decoded.insert("raw".to_string(), raw.clone());
            return Some((
                CardType::IDTECK,
                CardData {
                    uid: raw.clone(),
                    raw,
                    decoded,
                },
            ));
        }
    }

    // --- Non-cloneable LF types (detect but mark as non-cloneable) ---

    if COTAG_RE.is_match(&clean) {
        let mut decoded = HashMap::new();
        decoded.insert("type".to_string(), "COTAG".to_string());
        return Some((
            CardType::COTAG,
            CardData {
                uid: "COTAG".to_string(),
                raw: String::new(),
                decoded,
            },
        ));
    }

    if EM4X50_RE.is_match(&clean) {
        let mut decoded = HashMap::new();
        decoded.insert("type".to_string(), "EM4x50".to_string());
        return Some((
            CardType::EM4x50,
            CardData {
                uid: "EM4x50".to_string(),
                raw: String::new(),
                decoded,
            },
        ));
    }

    if HITAG_RE.is_match(&clean) {
        let mut decoded = HashMap::new();
        decoded.insert("type".to_string(), "Hitag".to_string());
        return Some((
            CardType::Hitag,
            CardData {
                uid: "Hitag".to_string(),
                raw: String::new(),
                decoded,
            },
        ));
    }

    // NexWatch — dedicated parsing before generic fallback
    if clean.contains("NexWatch") || clean.contains("NXT") {
        let mut decoded = HashMap::new();
        decoded.insert("type".to_string(), "NexWatch".to_string());
        // Try dedicated ID pattern first
        if let Some(caps) = NEXWATCH_ID_RE.captures(&clean) {
            let id = caps[1].to_string();
            decoded.insert("card_id".to_string(), id.clone());
            // Also grab raw if available
            if let Some(raw_caps) = NEXWATCH_RAW_RE.captures(&clean) {
                let raw = raw_caps[1].to_uppercase();
                decoded.insert("raw".to_string(), raw.clone());
                return Some((
                    CardType::NexWatch,
                    CardData { uid: raw.clone(), raw, decoded },
                ));
            }
            return Some((
                CardType::NexWatch,
                CardData { uid: id.clone(), raw: id, decoded },
            ));
        }
        // Try raw hex pattern
        if let Some(caps) = NEXWATCH_RAW_RE.captures(&clean) {
            let raw = caps[1].to_uppercase();
            decoded.insert("raw".to_string(), raw.clone());
            return Some((
                CardType::NexWatch,
                CardData { uid: raw.clone(), raw, decoded },
            ));
        }
        // Last resort: generic hex block
        if let Some(hex) = extract_first_hex_block(&clean) {
            return Some((
                CardType::NexWatch,
                CardData { uid: hex.clone(), raw: hex, decoded },
            ));
        }
    }

    // Viking — dedicated parsing before generic fallback
    if clean.contains("Viking") || clean.contains("viking") {
        let mut decoded = HashMap::new();
        decoded.insert("type".to_string(), "Viking".to_string());
        // Try dedicated ID pattern first
        if let Some(caps) = VIKING_ID_RE.captures(&clean) {
            let id = caps[1].to_string();
            decoded.insert("card_id".to_string(), id.clone());
            // Also grab raw if available
            if let Some(raw_caps) = VIKING_RAW_RE.captures(&clean) {
                let raw = raw_caps[1].to_uppercase();
                decoded.insert("raw".to_string(), raw.clone());
                return Some((
                    CardType::Viking,
                    CardData { uid: raw.clone(), raw, decoded },
                ));
            }
            return Some((
                CardType::Viking,
                CardData { uid: id.clone(), raw: id, decoded },
            ));
        }
        // Try raw hex pattern
        if let Some(caps) = VIKING_RAW_RE.captures(&clean) {
            let raw = caps[1].to_uppercase();
            decoded.insert("raw".to_string(), raw.clone());
            return Some((
                CardType::Viking,
                CardData { uid: raw.clone(), raw, decoded },
            ));
        }
        // Last resort: generic hex block
        if let Some(hex) = extract_first_hex_block(&clean) {
            return Some((
                CardType::Viking,
                CardData { uid: hex.clone(), raw: hex, decoded },
            ));
        }
    }

    // Generic fallback for valid tags using [+] Valid <TYPE>
    if let Some(caps) = VALID_TAG_RE.captures(&clean) {
        let tag_name = caps[1].to_string();
        let card_type = match tag_name.to_lowercase().as_str() {
            "viking" => CardType::Viking,
            "nexwatch" => CardType::NexWatch,
            _ => return None,
        };
        let mut decoded = HashMap::new();
        decoded.insert("type".to_string(), tag_name.clone());
        let raw = extract_first_hex_block(&clean).unwrap_or_default();
        let uid = if raw.is_empty() {
            "unknown".to_string()
        } else {
            raw.clone()
        };
        return Some((
            card_type,
            CardData { uid, raw, decoded },
        ));
    }

    None
}

// ---------------------------------------------------------------------------
// Dedicated sub-parsers for types with complex fields
// ---------------------------------------------------------------------------

fn parse_hid(clean: &str) -> Option<(CardType, CardData)> {
    let mut decoded = HashMap::new();
    decoded.insert("type".to_string(), "HID Prox".to_string());

    // Detect HID format (H10301 etc.)
    if let Some(fmt_caps) = HID_FORMAT_RE.captures(clean) {
        decoded.insert("format".to_string(), fmt_caps[0].to_string());
    }

    let (fc, cn) = if let Some(caps) = HID_FC_CN_RE.captures(clean) {
        let fc = caps[1].to_string();
        let cn = caps[2].to_string();
        decoded.insert("facility_code".to_string(), fc.clone());
        decoded.insert("card_number".to_string(), cn.clone());
        (fc, cn)
    } else {
        (String::new(), String::new())
    };

    let raw = if let Some(caps) = HID_RAW_RE.captures(clean) {
        caps[1].to_uppercase()
    } else {
        String::new()
    };

    let uid = if !fc.is_empty() && !cn.is_empty() {
        format!("FC{}:CN{}", fc, cn)
    } else if !raw.is_empty() {
        raw.clone()
    } else {
        return None;
    };

    if !raw.is_empty() {
        decoded.insert("raw".to_string(), raw.clone());
    }

    Some((CardType::HIDProx, CardData { uid, raw, decoded }))
}

fn parse_ioprox(clean: &str) -> Option<(CardType, CardData)> {
    let mut decoded = HashMap::new();
    decoded.insert("type".to_string(), "IOProx".to_string());

    // Try FC/CN/VN first
    if let Some(caps) = IOPROX_FC_CN_RE.captures(clean) {
        let vn = caps.get(1)
            .map(|m| m.as_str().to_string())
            .unwrap_or_else(|| "0".to_string());
        decoded.insert("version".to_string(), vn);
        let fc = caps[2].to_string();
        let cn = caps[3].to_string();
        decoded.insert("facility_code".to_string(), fc.clone());
        decoded.insert("card_number".to_string(), cn.clone());
        let uid = format!("FC{}:CN{}", fc, cn);
        return Some((
            CardType::IOProx,
            CardData {
                uid,
                raw: String::new(),
                decoded,
            },
        ));
    }

    // Fallback to raw
    if let Some(caps) = IOPROX_RAW_RE.captures(clean) {
        let uid = caps[1].to_uppercase();
        decoded.insert("id".to_string(), uid.clone());
        return Some((
            CardType::IOProx,
            CardData {
                uid: uid.clone(),
                raw: uid,
                decoded,
            },
        ));
    }

    None
}

fn parse_awid(clean: &str) -> Option<(CardType, CardData)> {
    let mut decoded = HashMap::new();
    decoded.insert("type".to_string(), "AWID".to_string());

    // Detect bit format (26/34/37/50)
    if let Some(fmt_caps) = AWID_FMT_RE.captures(clean) {
        decoded.insert("format".to_string(), fmt_caps[1].to_string());
    }

    if let Some(caps) = AWID_RE.captures(clean) {
        let fc = caps[1].to_string();
        let cn = caps[2].to_string();
        decoded.insert("facility_code".to_string(), fc.clone());
        decoded.insert("card_number".to_string(), cn.clone());
        let uid = format!("FC{}:CN{}", fc, cn);
        return Some((
            CardType::AWID,
            CardData {
                uid: uid.clone(),
                raw: uid,
                decoded,
            },
        ));
    }

    None
}

fn parse_fdxb(clean: &str) -> Option<(CardType, CardData)> {
    let mut decoded = HashMap::new();
    decoded.insert("type".to_string(), "FDX-B".to_string());

    if let Some(caps) = FDXB_RE.captures(clean) {
        let country = caps[1].to_string();
        let national = caps[2].to_string();
        decoded.insert("country".to_string(), country.clone());
        decoded.insert("national_id".to_string(), national.clone());
        let uid = format!("{}:{}", country, national);
        return Some((
            CardType::FDX_B,
            CardData {
                uid,
                raw: String::new(),
                decoded,
            },
        ));
    }

    // Fallback: try to get any raw hex
    let raw = extract_first_hex_block(clean).unwrap_or_default();
    if !raw.is_empty() {
        decoded.insert("raw".to_string(), raw.clone());
        return Some((
            CardType::FDX_B,
            CardData {
                uid: raw.clone(),
                raw,
                decoded,
            },
        ));
    }

    None
}

fn parse_paradox(clean: &str) -> Option<(CardType, CardData)> {
    let mut decoded = HashMap::new();
    decoded.insert("type".to_string(), "Paradox".to_string());

    // Try FC/CN first
    if let Some(caps) = PARADOX_FC_CN_RE.captures(clean) {
        let fc = caps[1].to_string();
        let cn = caps[2].to_string();
        decoded.insert("facility_code".to_string(), fc.clone());
        decoded.insert("card_number".to_string(), cn.clone());
        let uid = format!("FC{}:CN{}", fc, cn);
        // Also grab raw if available
        if let Some(raw_caps) = PARADOX_RAW_RE.captures(clean) {
            decoded.insert("raw".to_string(), raw_caps[1].to_uppercase());
        }
        return Some((
            CardType::Paradox,
            CardData {
                uid,
                raw: decoded.get("raw").cloned().unwrap_or_default(),
                decoded,
            },
        ));
    }

    // Fallback to raw
    if let Some(caps) = PARADOX_RAW_RE.captures(clean) {
        let raw = caps[1].to_uppercase();
        decoded.insert("raw".to_string(), raw.clone());
        return Some((
            CardType::Paradox,
            CardData {
                uid: raw.clone(),
                raw,
                decoded,
            },
        ));
    }

    // Last resort: hex block
    let raw = extract_first_hex_block(clean).unwrap_or_default();
    if !raw.is_empty() {
        return Some((
            CardType::Paradox,
            CardData {
                uid: raw.clone(),
                raw,
                decoded,
            },
        ));
    }

    None
}

fn parse_keri(clean: &str) -> Option<(CardType, CardData)> {
    let mut decoded = HashMap::new();
    decoded.insert("type".to_string(), "Keri".to_string());

    // Detect type: Internal or MS
    if let Some(type_caps) = KERI_TYPE_RE.captures(clean) {
        let ktype = type_caps[1].to_lowercase();
        let keri_type = if ktype.contains("internal") { "i" } else { "m" };
        decoded.insert("keri_type".to_string(), keri_type.to_string());
    }

    if let Some(caps) = KERI_RE.captures(clean) {
        let raw = caps[1].to_uppercase();
        decoded.insert("raw".to_string(), raw.clone());
        return Some((
            CardType::Keri,
            CardData {
                uid: raw.clone(),
                raw,
                decoded,
            },
        ));
    }

    // No fallback — extract_first_hex_block is unreliable for Keri and may grab
    // the wrong hex value. command_builder needs keri_type + valid raw for --type
    // flag, so it's better to report "card not readable" than clone with wrong data.
    None
}

fn parse_pyramid(clean: &str) -> Option<(CardType, CardData)> {
    let mut decoded = HashMap::new();
    decoded.insert("type".to_string(), "Pyramid".to_string());

    // Try FC/Card first (e.g. "[+] Pyramid - len: 26, FC: 123, Card: 456, Raw: AABBCCDD")
    if let Some(caps) = PYRAMID_FC_CN_RE.captures(clean) {
        let fc = caps[1].to_string();
        let cn = caps[2].to_string();
        decoded.insert("facility_code".to_string(), fc.clone());
        decoded.insert("card_number".to_string(), cn.clone());
        let uid = format!("FC{}:CN{}", fc, cn);
        // Also grab raw if available
        if let Some(raw_caps) = PYRAMID_RAW_RE.captures(clean) {
            decoded.insert("raw".to_string(), raw_caps[1].to_uppercase());
        }
        return Some((
            CardType::Pyramid,
            CardData {
                uid,
                raw: decoded.get("raw").cloned().unwrap_or_default(),
                decoded,
            },
        ));
    }

    // Fallback to raw hex
    if let Some(caps) = PYRAMID_RAW_RE.captures(clean) {
        let raw = caps[1].to_uppercase();
        decoded.insert("raw".to_string(), raw.clone());
        return Some((
            CardType::Pyramid,
            CardData {
                uid: raw.clone(),
                raw,
                decoded,
            },
        ));
    }

    // Last resort: any hex block
    let raw = extract_first_hex_block(clean).unwrap_or_default();
    if !raw.is_empty() {
        return Some((
            CardType::Pyramid,
            CardData {
                uid: raw.clone(),
                raw,
                decoded,
            },
        ));
    }

    None
}

fn parse_presco(clean: &str) -> Option<(CardType, CardData)> {
    let mut decoded = HashMap::new();
    decoded.insert("type".to_string(), "Presco".to_string());

    // Try site code + user code
    if let Some(caps) = PRESCO_SC_UC_RE.captures(clean) {
        let sc = caps[1].to_string();
        let uc = caps[2].to_string();
        decoded.insert("site_code".to_string(), sc.clone());
        decoded.insert("user_code".to_string(), uc.clone());
        let uid = format!("SC{}:UC{}", sc, uc);
        return Some((
            CardType::Presco,
            CardData {
                uid,
                raw: String::new(),
                decoded,
            },
        ));
    }

    // Fallback to hex card ID
    if let Some(caps) = PRESCO_RE.captures(clean) {
        let hex = caps[1].to_uppercase();
        decoded.insert("hex".to_string(), hex.clone());
        return Some((
            CardType::Presco,
            CardData {
                uid: hex.clone(),
                raw: hex,
                decoded,
            },
        ));
    }

    None
}

fn parse_pac(clean: &str) -> Option<(CardType, CardData)> {
    let mut decoded = HashMap::new();
    decoded.insert("type".to_string(), "PAC".to_string());

    // Try card number
    if let Some(caps) = PAC_CN_RE.captures(clean) {
        let cn = caps[1].to_string();
        decoded.insert("card_number".to_string(), cn.clone());
        // Also grab raw if available
        if let Some(raw_caps) = PAC_RAW_RE.captures(clean) {
            decoded.insert("raw".to_string(), raw_caps[1].to_uppercase());
        }
        return Some((
            CardType::PAC,
            CardData {
                uid: cn,
                raw: decoded.get("raw").cloned().unwrap_or_default(),
                decoded,
            },
        ));
    }

    // Fallback to raw
    if let Some(caps) = PAC_RAW_RE.captures(clean) {
        let raw = caps[1].to_uppercase();
        decoded.insert("raw".to_string(), raw.clone());
        return Some((
            CardType::PAC,
            CardData {
                uid: raw.clone(),
                raw,
                decoded,
            },
        ));
    }

    None
}

fn parse_noralsy(clean: &str) -> Option<(CardType, CardData)> {
    let mut decoded = HashMap::new();
    decoded.insert("type".to_string(), "Noralsy".to_string());

    // Try card number + year
    if let Some(caps) = NORALSY_RE.captures(clean) {
        let cn = caps[1].to_string();
        decoded.insert("card_number".to_string(), cn.clone());
        if let Some(year) = caps.get(2) {
            decoded.insert("year".to_string(), year.as_str().to_string());
        }
        // Also grab raw
        if let Some(raw_caps) = NORALSY_RAW_RE.captures(clean) {
            decoded.insert("raw".to_string(), raw_caps[1].to_uppercase());
        }
        let raw = decoded.get("raw").cloned().unwrap_or_default();
        return Some((
            CardType::Noralsy,
            CardData {
                uid: cn,
                raw,
                decoded,
            },
        ));
    }

    // Raw fallback
    if let Some(caps) = NORALSY_RAW_RE.captures(clean) {
        let raw = caps[1].to_uppercase();
        decoded.insert("raw".to_string(), raw.clone());
        return Some((
            CardType::Noralsy,
            CardData {
                uid: raw.clone(),
                raw,
                decoded,
            },
        ));
    }

    None
}

// ---------------------------------------------------------------------------
// T5577 detection
// ---------------------------------------------------------------------------

/// Parse `lf t55xx detect` output for password status and chip info.
pub fn parse_t5577_detect(output: &str) -> T5577Status {
    let clean = strip_ansi(output);

    // Check if detected at all
    let detected = clean.contains("T55xx")
        || clean.contains("T5577")
        || clean.contains("T5555")
        || clean.contains("Chip type");

    let chip_type = T5577_CHIP_RE
        .captures(&clean)
        .map(|c| c[1].to_string())
        .unwrap_or_else(|| {
            if detected {
                "T55x7".to_string()
            } else {
                String::new()
            }
        });

    let password_set = T5577_PASSWORD_RE
        .captures(&clean)
        .map(|c| c[1].eq_ignore_ascii_case("Yes"))
        .unwrap_or(false);

    let block0 = T5577_BLOCK0_RE
        .captures(&clean)
        .map(|c| c[1].to_uppercase());

    let modulation = T5577_MOD_RE
        .captures(&clean)
        .map(|c| c[1].trim().to_string());

    T5577Status {
        detected,
        chip_type,
        password_set,
        block0,
        modulation,
    }
}

/// Parse `lf t55xx chk` output for a found password.
/// Returns the password hex string if found (e.g. "51243648").
pub fn parse_t5577_chk(output: &str) -> Option<String> {
    let clean = strip_ansi(output);
    T5577_PASSWORD_FOUND_RE
        .captures(&clean)
        .map(|c| c[1].to_uppercase())
}

// ---------------------------------------------------------------------------
// Verification
// ---------------------------------------------------------------------------

/// Parse verification output: compare two UID strings.
pub fn verify_match(source_uid: &str, clone_output: &str) -> (bool, Vec<u16>) {
    let clean = strip_ansi(clone_output);
    if let Some((_, card_data)) = parse_lf_search(&clean) {
        let matches = card_data.uid.eq_ignore_ascii_case(source_uid);
        if matches {
            (true, vec![])
        } else {
            (false, vec![0]) // block 0 mismatch sentinel
        }
    } else {
        (false, vec![0])
    }
}

/// Enhanced verification: compare decoded fields instead of just UID string.
/// For FC/CN-based types, compare the individual fields for more robust matching.
pub fn verify_match_detailed(
    source_type: &CardType,
    source_decoded: &HashMap<String, String>,
    clone_output: &str,
) -> (bool, Vec<u16>) {
    let clean = strip_ansi(clone_output);

    if let Some((detected_type, clone_data)) = parse_lf_search(&clean) {
        // Type must match
        if *source_type != detected_type {
            return (false, vec![0]);
        }

        // For FC/CN types, compare fields individually
        let fc_match = match (
            source_decoded.get("facility_code"),
            clone_data.decoded.get("facility_code"),
        ) {
            (Some(src), Some(dst)) => src == dst,
            (None, None) => true,
            _ => false,
        };

        let cn_match = match (
            source_decoded.get("card_number"),
            clone_data.decoded.get("card_number"),
        ) {
            (Some(src), Some(dst)) => src == dst,
            (None, None) => true,
            _ => false,
        };

        // For raw-based types, compare raw hex
        let raw_match = match (source_decoded.get("raw"), clone_data.decoded.get("raw")) {
            (Some(src), Some(dst)) => src.eq_ignore_ascii_case(dst),
            _ => true, // If either doesn't have raw, skip raw comparison
        };

        // For id-based types (e.g. EM4100), compare the id field
        let id_match = match (source_decoded.get("id"), clone_data.decoded.get("id")) {
            (Some(src), Some(dst)) => src.eq_ignore_ascii_case(dst),
            (None, None) => true,
            _ => false,
        };

        if fc_match && cn_match && raw_match && id_match {
            (true, vec![])
        } else {
            let mut mismatched = vec![];
            if !fc_match {
                mismatched.push(1);
            }
            if !cn_match {
                mismatched.push(2);
            }
            if !raw_match {
                mismatched.push(0);
            }
            if !id_match {
                mismatched.push(3);
            }
            (false, mismatched)
        }
    } else {
        (false, vec![0])
    }
}

// ---------------------------------------------------------------------------
// EM4305 detection and verification
// ---------------------------------------------------------------------------

/// Parse `lf em 4x05 info` output. Returns true if an EM4305 chip was detected.
/// The PM3 output contains "[+] EM4x05/EM4x69" or similar chip identification lines.
pub fn parse_em4305_info(output: &str) -> bool {
    let clean = strip_ansi(output);
    // PM3 Iceman fork outputs "[+] EM4x05/EM4x69" or "Chip type: EM4x05" on success
    clean.contains("EM4x05")
        || clean.contains("EM4x69")
        || clean.contains("EM4305")
        || clean.contains("EM4469")
}

/// Parse `lf em 4x05 read -a 0` output to extract word 0 hex value.
/// Returns the hex string of word 0 (e.g., "00000000") or None if parse failed.
/// Used to verify wipe succeeded: word 0 should be all zeros after a successful wipe.
pub fn parse_em4305_word0(output: &str) -> Option<String> {
    static EM4305_WORD_RE: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"(?i)(?:Word|Address)\s*0+\s*[:|]\s*([0-9A-Fa-f]{8})")
            .expect("bad em4305 word regex")
    });

    let clean = strip_ansi(output);
    EM4305_WORD_RE
        .captures(&clean)
        .map(|c| c[1].to_uppercase())
}

// ---------------------------------------------------------------------------
// Utility
// ---------------------------------------------------------------------------

fn extract_first_hex_block(s: &str) -> Option<String> {
    static HEX_BLOCK_RE: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"(?:\b0[xX])?([0-9A-Fa-f]{8,})\b").expect("bad hex block regex"));

    HEX_BLOCK_RE.captures(s).map(|c| c[1].to_uppercase())
}
