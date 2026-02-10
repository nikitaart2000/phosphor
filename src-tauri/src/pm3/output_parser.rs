use regex::Regex;
use std::collections::HashMap;
use std::sync::LazyLock;

use crate::cards::types::{CardData, CardType};

static ANSI_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\x1b\[[0-9;]*[mGKHJ]").expect("bad ansi regex"));

static EM4100_ID_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"EM 410x ID\s*[\-:]\s*([0-9A-Fa-f]{10})").expect("bad em regex"));

static HID_FC_CN_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)FC[:/\s]*(\d+)\s*[,;]?\s*CN[:/\s]*(\d+)").expect("bad hid regex")
});

static HID_RAW_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)RAW[:/\s]*([0-9A-Fa-f]+)").expect("bad hid raw regex"));

static INDALA_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)Indala.*?ID[:/\s]*([0-9A-Fa-f]+)").expect("bad indala regex"));

static IOPROX_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)IO\s*Prox.*?ID[:/\s]*([0-9A-Fa-f]+)").expect("bad ioprox regex"));

static AWID_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)AWID.*?FC[:/\s]*(\d+)\s*[,;]?\s*CN[:/\s]*(\d+)").expect("bad awid regex"));

static VALID_TAG_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\[\+\]\s*Valid\s+(\S+)\s+.*?found").expect("bad valid tag regex"));

pub fn strip_ansi(input: &str) -> String {
    ANSI_RE.replace_all(input, "").to_string()
}

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
        if let Some(caps) = INDALA_RE.captures(&clean) {
            let uid = caps[1].to_uppercase();
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

    // IO Prox
    if clean.contains("IO Prox") {
        if let Some(caps) = IOPROX_RE.captures(&clean) {
            let uid = caps[1].to_uppercase();
            let mut decoded = HashMap::new();
            decoded.insert("type".to_string(), "IOProx".to_string());
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
    }

    // AWID
    if clean.contains("AWID") {
        if let Some(caps) = AWID_RE.captures(&clean) {
            let fc = caps[1].to_string();
            let cn = caps[2].to_string();
            let uid = format!("FC{}:CN{}", fc, cn);
            let mut decoded = HashMap::new();
            decoded.insert("type".to_string(), "AWID".to_string());
            decoded.insert("facility_code".to_string(), fc);
            decoded.insert("card_number".to_string(), cn);
            return Some((
                CardType::AWID,
                CardData {
                    uid: uid.clone(),
                    raw: uid,
                    decoded,
                },
            ));
        }
    }

    // Generic fallback for other valid tags using [+] Valid <TYPE>
    if let Some(caps) = VALID_TAG_RE.captures(&clean) {
        let tag_name = caps[1].to_string();
        let card_type = match tag_name.to_lowercase().as_str() {
            "paradox" => CardType::Paradox,
            "viking" => CardType::Viking,
            "pyramid" => CardType::Pyramid,
            "keri" => CardType::Keri,
            "nexwatch" => CardType::NexWatch,
            "fdx-b" | "fdxb" => CardType::FDX_B,
            _ => return None,
        };
        let mut decoded = HashMap::new();
        decoded.insert("type".to_string(), tag_name.clone());
        // Try to grab any raw hex from the output
        let raw = extract_first_hex_block(&clean).unwrap_or_default();
        let uid = if raw.is_empty() {
            "unknown".to_string()
        } else {
            raw.clone()
        };
        return Some((
            card_type,
            CardData {
                uid,
                raw,
                decoded,
            },
        ));
    }

    None
}

fn parse_hid(clean: &str) -> Option<(CardType, CardData)> {
    let mut decoded = HashMap::new();
    decoded.insert("type".to_string(), "HID Prox".to_string());

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

fn extract_first_hex_block(s: &str) -> Option<String> {
    static HEX_BLOCK_RE: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"\b([0-9A-Fa-f]{8,})\b").expect("bad hex block regex"));

    HEX_BLOCK_RE
        .captures(s)
        .map(|c| c[1].to_uppercase())
}

/// Parse `lf search` output to check if a T5577 blank is present.
/// Returns true if T55xx chip is detected in the output.
pub fn is_t5577_detected(output: &str) -> bool {
    let clean = strip_ansi(output);
    clean.contains("T55xx") || clean.contains("T5577") || clean.contains("T5555")
}

/// Parse verification output: compare two UID strings.
pub fn verify_match(source_uid: &str, clone_output: &str) -> (bool, Vec<u16>) {
    let clean = strip_ansi(clone_output);
    // After cloning, `lf search` on the T5577 should return the original UID
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
