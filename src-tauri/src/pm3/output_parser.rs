use regex::Regex;
use std::collections::HashMap;
use std::sync::LazyLock;

use crate::cards::types::{AutopwnEvent, CardData, CardType, MagicGeneration, T5577Status};

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
    LazyLock::new(|| Regex::new(r"EM 410x ID\s*[\-:]?\s*([0-9A-Fa-f]{10})").expect("bad em regex"));

static HID_FC_CN_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)FC[:/\s]*(\d+)\s*[,;]?\s*CN[:/\s]*(\d+)").expect("bad hid regex")
});

static HID_RAW_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)(?:HID|Prox).*?RAW[:/\s]*([0-9A-Fa-f]+)").expect("bad hid raw regex"));

// PM3 outputs raw hex on a standalone line: "[=] raw: 200078BE5E1E"
// Marker can be [+] or [=] depending on context. No protocol prefix on this line.
static STANDALONE_RAW_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?im)^\s*\[[+=]\]\s*raw[:/\s]+([0-9A-Fa-f]+)").expect("bad standalone raw regex")
});

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

// XSF format from real PM3 output: "IO Prox - XSF(01)65:01337, Raw: 007859603059cdaf"
// Groups: (1)=VN decimal, (2)=FC hex, (3)=CN decimal
static IOPROX_XSF_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)IO\s*Prox.*?XSF\((\d+)\)([0-9A-Fa-f]+):(\d+)")
        .expect("bad ioprox xsf regex")
});

static IOPROX_RAW_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)IO\s*Prox.*?(?:ID|Raw)[:/\s]*([0-9A-Fa-f]+)").expect("bad ioprox regex")
});

static AWID_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)AWID.*?FC[:/\s]*(\d+).*?(?:CN|Card)[:/\s]*(\d+)").expect("bad awid regex")
});

static AWID_FMT_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)AWID\s*(?:-\s*)?(?:len[:/\s]*)?(\d+)(?:\s*bit)?").expect("bad awid fmt regex")
});

static FDXB_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)FDX-?B.*?Country[:/\s]*(\d+).*?(?:National|ID)[:/\s]*(\d+)")
        .expect("bad fdxb regex")
});

// Real PM3 multi-line output: "Animal ID......... 999-123456789012"
// Country and National on separate lines, but Animal ID has both on one line
static FDXB_ANIMAL_ID_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"Animal\s+ID[.\s]+(\d+)-(\d+)")
        .expect("bad fdxb animal id regex")
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

// Real PM3 output: "KERI - Internal ID: 12345, Raw: E000000080003039"
static KERI_INTERNAL_ID_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)Internal\s+ID[:/\s]*(\d+)").expect("bad keri internal id regex")
});

// Descrambled MS line: "Descrambled MS - FC: 1 Card: 12544"
static KERI_MS_FC_CN_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)(?:Descrambled\s+)?MS.*?FC[:/\s]*(\d+).*?Card[:/\s]*(\d+)")
        .expect("bad keri ms fc/cn regex")
});

// ---------------------------------------------------------------------------
// Regex patterns — 11 new types
// ---------------------------------------------------------------------------

static PRESCO_RE: LazyLock<Regex> = LazyLock::new(|| {
    // Real PM3 output uses "Full code:" not "Card:"
    Regex::new(r"(?i)Presco.*?(?:Card|Full\s*code)[:/\s]*([0-9A-Fa-f]+)").expect("bad presco regex")
});

static PRESCO_SC_UC_RE: LazyLock<Regex> = LazyLock::new(|| {
    // Real PM3 output: "Presco Site code: 0 User code: 57470"
    Regex::new(r"(?i)Presco.*?Site\s*(?:code)?[:/\s]*(\d+).*?User\s*(?:code)?[:/\s]*(\d+)")
        .expect("bad presco sc/uc regex")
});

static NEDAP_CARD_RE: LazyLock<Regex> = LazyLock::new(|| {
    // Real PM3 outputs "ID: 12345" not "Card: 12345"
    Regex::new(r"(?i)Nedap.*?(?:Card|ID)[:/\s]*(\d+)").expect("bad nedap card regex")
});

static NEDAP_SUB_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)Nedap.*?Sub(?:type)?[:/\s]*(\d+)").expect("bad nedap sub regex")
});

// Real PM3 output: "customer code: 101"
static NEDAP_CC_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)customer\s*code[:/\s]*(\d+)").expect("bad nedap cc regex")
});

// Real PM3 output: "G-Prox-II - Len: 26 FC: 123 Card: 1234 xor: 141, Raw: ..."
// Need FC + Card + xor + Len for clone command (--xor --fmt --fc --cn)
static GPROXII_FC_CN_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)G-?Prox.*?FC[:/\s]*(\d+).*?Card[:/\s]*(\d+)")
        .expect("bad gproxii fc/cn regex")
});
static GPROXII_XOR_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)xor[:/\s]*(\d+)").expect("bad gproxii xor regex")
});
static GPROXII_FMT_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)Len[:/\s]*(\d+)").expect("bad gproxii fmt regex")
});

static GALLAGHER_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?i)Gallagher.*?Region(?:\s+Code)?[:/\s]*(\d+).*?Facility(?:\s+Code)?[:/\s]*(\d+).*?Card\s+(?:Number|No\.?)[:/\s]*(\d+).*?Issue\s+Level[:/\s]*(\d+)",
    )
    .expect("bad gallagher regex")
});

// Per-field Gallagher regexes — fallback for multi-line PM3 output
static GALLAGHER_RC_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)Region(?:\s+Code)?[:/\s]*(\d+)").expect("bad gallagher rc regex")
});
static GALLAGHER_FC_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)Facility(?:\s+Code)?[:/\s]*(\d+)").expect("bad gallagher fc regex")
});
static GALLAGHER_CN_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)Card\s+(?:Number|No\.?)[:/\s]*(\d+)").expect("bad gallagher cn regex")
});
static GALLAGHER_IL_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)Issue\s+Level[:/\s]*(\d+)").expect("bad gallagher il regex")
});

static PAC_DETECT_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)\[\+\].*\b(?:PAC|Stanley)\b").expect("bad pac detect regex")
});

static PAC_CN_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)PAC(?:/Stanley)?.*?Card[:/\s]*([0-9A-Fa-f]+)").expect("bad pac cn regex")
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

// Real PM3 output: "88bit id : 521512301 (0x1f15a56d)"
static NEXWATCH_88BIT_ID_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)88bit\s+id\s*:\s*(\d+)").expect("bad nexwatch 88bit id regex")
});

static NEXWATCH_RAW_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)(?:NexWatch|NXT).*?Raw[:/\s]*([0-9A-Fa-f]+)")
        .expect("bad nexwatch raw regex")
});

static VIKING_ID_RE: LazyLock<Regex> = LazyLock::new(|| {
    // Real PM3 output: "Viking - Card 1A2B3C4D" (hex ID, just "Card" not "Card ID")
    Regex::new(r"(?i)Viking.*?(?:Card(?:\s*ID)?|ID)[:/\s]*([0-9A-Fa-f]+)")
        .expect("bad viking id regex")
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
// HF detection patterns (13.56 MHz)
// ---------------------------------------------------------------------------

// UID from hf search / hf 14a info: "UID: 01 02 03 04" or "UID: 04 11 22 33 44 55 66"
static HF_UID_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)UID\s*:\s*((?:[0-9A-Fa-f]{2}[\s:]*){4,10})")
        .expect("bad hf uid regex")
});

// ATQA from hf 14a info: "ATQA: 00 04"
static HF_ATQA_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)ATQA\s*:\s*([0-9A-Fa-f]{2}\s+[0-9A-Fa-f]{2})")
        .expect("bad hf atqa regex")
});

// SAK from hf 14a info: "SAK: 08 [2]" or "SAK: 08"
static HF_SAK_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)SAK\s*:\s*([0-9A-Fa-f]{2})").expect("bad hf sak regex")
});

// ATS: "ATS: 06 75 77 81 02 80"
static HF_ATS_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)ATS\s*:\s*((?:[0-9A-Fa-f]{2}\s*)+)").expect("bad hf ats regex")
});

// PRNG detection: "Prng detection: WEAK" or "Prng detection..... weak"
// PM3 uses dots or colons as separator depending on context
static HF_PRNG_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)Prng\s+detection[\s.:]+(WEAK|HARD|STATIC)")
        .expect("bad hf prng regex")
});

// Magic capabilities from hf mf info output
static HF_MAGIC_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)(?:Magic|Gen(?:eration)?)\s*(?:capabilities)?[\s.:]*(?::[\s.]*)?(Gen\s*1[ab]?|CUID|USCUID|Gen\s*2|Gen\s*3|APDU|UFUID|GDM|Gen\s*4\s*(?:GTU|GDM)?|[Uu]ltimate)")
        .expect("bad hf magic regex")
});

// iCLASS/Picopass detection with optional CSN
// Requires [+] marker to avoid matching "Searching for iCLASS / PicoPass tag..." lines
static HF_ICLASS_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)\[\+\].*(?:iCLASS|Picopass)")
        .expect("bad hf iclass regex")
});

// iCLASS CSN extraction
static HF_ICLASS_CSN_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)CSN\s*:\s*([0-9A-Fa-f\s]+)").expect("bad hf iclass csn regex")
});

// DESFire detection
static HF_DESFIRE_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)(?:MIFARE\s+)?DESFire(?:\s+(?:EV[123]|Light))?")
        .expect("bad hf desfire regex")
});

// NTAG type: "NTAG 213" / "NTAG 215" / "NTAG 216" / "NTAG213" etc.
static HF_NTAG_TYPE_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)NTAG\s*(\d{3})").expect("bad hf ntag type regex")
});

// Ultralight type: "Ultralight EV1" / "Ultralight C" / "Ultralight Nano"
static HF_MFU_TYPE_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)(?:MIFARE\s+)?Ultralight(?:\s+(EV1|C|Nano|AES))?")
        .expect("bad hf mfu type regex")
});

// ---------------------------------------------------------------------------
// Autopwn progress regexes (hf mf autopwn streaming output)
// ---------------------------------------------------------------------------

// "found 12/32 keys (D)" — dictionary phase progress
static AUTOPWN_KEYS_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"found\s+(\d+)\s*/\s*(\d+)\s+keys").expect("bad autopwn keys regex")
});

// "found valid key [ FFFFFFFFFFFF ]" — individual key found
static AUTOPWN_KEY_FOUND_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)found\s+valid\s+key\s*\[\s*([0-9A-Fa-f]{12})\s*\]")
        .expect("bad autopwn key found regex")
});

// "Succeeded in dumping all blocks" — full dump
static AUTOPWN_DUMP_OK_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)Succeeded\s+in\s+dumping\s+all\s+blocks")
        .expect("bad autopwn dump ok regex")
});

// "Dump file is PARTIAL complete" — partial dump
static AUTOPWN_DUMP_PARTIAL_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)Dump\s+file\s+is\s+PARTIAL").expect("bad autopwn dump partial regex")
});

// "saved 64 blocks to file hf-mf-01020304-dump.bin"
// or "saved to binary file `hf-mf-01020304-dump.bin`"
static AUTOPWN_DUMP_SAVED_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)saved\s+.*?(?:to\s+(?:binary\s+)?file\s+[`]?|file\s+)([^\s`]+\.(?:bin|json|eml))")
        .expect("bad autopwn dump saved regex")
});

// "all key recovery attempts failed" — total failure
static AUTOPWN_FAIL_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)all\s+key\s+recovery\s+attempts?\s+failed")
        .expect("bad autopwn fail regex")
});

// "autopwn execution time: 45 seconds"
static AUTOPWN_TIME_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)autopwn\s+execution\s+time\s*:\s*(\d+)\s*seconds?")
        .expect("bad autopwn time regex")
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
        let raw_hex = INDALA_RAW_RE
            .captures(&clean)
            .or_else(|| STANDALONE_RAW_RE.captures(&clean))
            .map(|c| c[1].to_uppercase());
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

    // Keri — improved with type detection (PM3 outputs "KERI" uppercase)
    if clean.contains("Keri") || clean.contains("KERI") {
        return parse_keri(&clean);
    }

    // Pyramid — dedicated parser for FC/CN extraction
    if clean.contains("Pyramid") {
        return parse_pyramid(&clean);
    }

    // --- New card types (check before generic fallback) ---

    // Gallagher
    if clean.contains("Gallagher") || clean.contains("GALLAGHER") {
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

    // GProxII (appears as "G-Prox-II", "Guardall", "G-Prox II" in PM3 output)
    if clean.contains("Guardall") || clean.contains("GProx") || clean.contains("G-Prox") {
        if let Some(caps) = GPROXII_FC_CN_RE.captures(&clean) {
            let fc = caps[1].to_string();
            let cn = caps[2].to_string();
            let xor = GPROXII_XOR_RE
                .captures(&clean)
                .map(|c| c[1].to_string())
                .unwrap_or_else(|| "0".to_string());
            let fmt = GPROXII_FMT_RE
                .captures(&clean)
                .map(|c| c[1].to_string())
                .unwrap_or_else(|| "26".to_string());
            let uid = format!("FC{}:CN{}", fc, cn);
            let mut decoded = HashMap::new();
            decoded.insert("type".to_string(), "GProxII".to_string());
            decoded.insert("facility_code".to_string(), fc);
            decoded.insert("card_number".to_string(), cn);
            decoded.insert("xor".to_string(), xor);
            decoded.insert("format".to_string(), fmt);
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
        let cc = NEDAP_CC_RE.captures(&clean).map(|c| c[1].to_string());
        if let Some(cn) = cn {
            let st = st.unwrap_or_else(|| "5".to_string()); // PM3 default subtype is 5
            let cc = cc.unwrap_or_else(|| "0".to_string());
            let uid = format!("ST{}:CC{}:ID{}", st, cc, cn);
            let mut decoded = HashMap::new();
            decoded.insert("type".to_string(), "Nedap".to_string());
            decoded.insert("subtype".to_string(), st);
            decoded.insert("customer_code".to_string(), cc);
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

        // Grab raw hex — try same-line regex first, then standalone multiline
        let raw_hex = NEXWATCH_RAW_RE
            .captures(&clean)
            .or_else(|| STANDALONE_RAW_RE.captures(&clean))
            .map(|c| c[1].to_uppercase());

        // Grab card ID — try original format, then real PM3 "88bit id" format
        let card_id = NEXWATCH_ID_RE
            .captures(&clean)
            .or_else(|| NEXWATCH_88BIT_ID_RE.captures(&clean))
            .map(|c| c[1].to_string());

        if let Some(ref id) = card_id {
            decoded.insert("card_id".to_string(), id.clone());
        }

        if let Some(raw) = raw_hex {
            decoded.insert("raw".to_string(), raw.clone());
            return Some((
                CardType::NexWatch,
                CardData { uid: raw.clone(), raw, decoded },
            ));
        }

        if let Some(id) = card_id {
            return Some((
                CardType::NexWatch,
                CardData { uid: id.clone(), raw: id, decoded },
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
// HF search parser
// ---------------------------------------------------------------------------

/// Parse `hf search` output (optionally enriched with `hf 14a info` / `hf mf info`).
/// Returns (CardType, CardData) for detected HF cards.
pub fn parse_hf_search(output: &str) -> Option<(CardType, CardData)> {
    let clean = strip_ansi(output);

    // No-card conditions
    if clean.contains("No known/supported 13.56 MHz tags found")
        || clean.contains("No data found")
        || clean.is_empty()
    {
        return None;
    }

    let mut decoded = HashMap::new();

    // --- iCLASS / Picopass (separate protocol, not ISO 14443-A) ---
    if HF_ICLASS_RE.is_match(&clean) {
        decoded.insert("type".to_string(), "IClass".to_string());
        let csn = HF_ICLASS_CSN_RE
            .captures(&clean)
            .map(|c| {
                c[1].split_whitespace()
                    .collect::<Vec<&str>>()
                    .join("")
                    .to_uppercase()
            })
            .unwrap_or_default();
        let uid = if csn.is_empty() {
            "iCLASS".to_string()
        } else {
            decoded.insert("uid".to_string(), csn.clone());
            csn
        };
        return Some((
            CardType::IClass,
            CardData {
                uid,
                raw: String::new(),
                decoded,
            },
        ));
    }

    // --- ISO 14443-A cards: extract UID, ATQA, SAK ---

    let uid = if let Some(caps) = HF_UID_RE.captures(&clean) {
        let raw_uid = caps[1].trim();
        // Normalize: remove spaces/colons, uppercase
        let uid_clean: String = raw_uid
            .chars()
            .filter(|c| c.is_ascii_hexdigit())
            .collect::<String>()
            .to_uppercase();
        let uid_size = uid_clean.len() / 2;
        decoded.insert("uid".to_string(), uid_clean.clone());
        decoded.insert("uid_size".to_string(), format!("{}B", uid_size));
        uid_clean
    } else {
        String::new()
    };

    // ATQA
    if let Some(caps) = HF_ATQA_RE.captures(&clean) {
        let atqa = caps[1]
            .split_whitespace()
            .map(|s| s.to_uppercase())
            .collect::<Vec<String>>()
            .join(" ");
        decoded.insert("atqa".to_string(), atqa);
    }

    // SAK
    let sak = if let Some(caps) = HF_SAK_RE.captures(&clean) {
        let sak_val = caps[1].to_uppercase();
        decoded.insert("sak".to_string(), sak_val.clone());
        u8::from_str_radix(&sak_val, 16).ok()
    } else {
        None
    };

    // ATS (optional, mainly DESFire)
    if let Some(caps) = HF_ATS_RE.captures(&clean) {
        decoded.insert("ats".to_string(), caps[1].trim().to_uppercase());
    }

    // PRNG detection (MIFARE Classic)
    if let Some(caps) = HF_PRNG_RE.captures(&clean) {
        decoded.insert("prng".to_string(), caps[1].to_uppercase());
    }

    // Magic card capabilities
    if let Some(caps) = HF_MAGIC_RE.captures(&clean) {
        decoded.insert("magic".to_string(), caps[1].to_string());
    }

    // --- DESFire (check before Classic: SAK 0x20 can be either) ---
    if HF_DESFIRE_RE.is_match(&clean) {
        decoded.insert("type".to_string(), "DESFire".to_string());
        return Some((
            CardType::DESFire,
            CardData {
                uid: uid.clone(),
                raw: String::new(),
                decoded,
            },
        ));
    }

    // --- NTAG (SAK 0x00, "NTAG" in output) ---
    if let Some(caps) = HF_NTAG_TYPE_RE.captures(&clean) {
        let ntag_type = format!("NTAG{}", &caps[1]);
        decoded.insert("type".to_string(), "NTAG".to_string());
        decoded.insert("ntag_type".to_string(), ntag_type);
        return Some((
            CardType::NTAG,
            CardData {
                uid: uid.clone(),
                raw: String::new(),
                decoded,
            },
        ));
    }

    // --- Ultralight (SAK 0x00, "Ultralight" in output, but not NTAG) ---
    if HF_MFU_TYPE_RE.is_match(&clean) && !HF_NTAG_TYPE_RE.is_match(&clean) {
        let ul_type = HF_MFU_TYPE_RE
            .captures(&clean)
            .and_then(|c| c.get(1))
            .map(|m| format!("Ultralight {}", m.as_str()))
            .unwrap_or_else(|| "Ultralight".to_string());
        decoded.insert("type".to_string(), "MifareUltralight".to_string());
        decoded.insert("ul_type".to_string(), ul_type);
        return Some((
            CardType::MifareUltralight,
            CardData {
                uid: uid.clone(),
                raw: String::new(),
                decoded,
            },
        ));
    }

    // --- SAK-based MIFARE Classic determination ---
    if let Some(sak_val) = sak {
        match sak_val {
            // Classic 1K: SAK 0x08, 0x88, 0x09, 0x89
            0x08 | 0x88 | 0x09 | 0x89 => {
                decoded.insert("type".to_string(), "MifareClassic1K".to_string());
                return Some((
                    CardType::MifareClassic1K,
                    CardData {
                        uid: uid.clone(),
                        raw: String::new(),
                        decoded,
                    },
                ));
            }
            // Classic 4K: SAK 0x18, 0x98, 0x19, 0x28, 0x38
            0x18 | 0x98 | 0x19 | 0x28 | 0x38 => {
                decoded.insert("type".to_string(), "MifareClassic4K".to_string());
                return Some((
                    CardType::MifareClassic4K,
                    CardData {
                        uid: uid.clone(),
                        raw: String::new(),
                        decoded,
                    },
                ));
            }
            // SAK 0x00 without NTAG/UL text: check ATQA
            0x00 => {
                if let Some(atqa) = decoded.get("atqa") {
                    if atqa == "00 44" {
                        decoded
                            .insert("type".to_string(), "MifareUltralight".to_string());
                        return Some((
                            CardType::MifareUltralight,
                            CardData {
                                uid: uid.clone(),
                                raw: String::new(),
                                decoded,
                            },
                        ));
                    }
                }
            }
            _ => {}
        }
    }

    // --- Text-based fallback for MIFARE Classic ---
    if clean.contains("MIFARE Classic 4K") || clean.contains("Classic 4K") {
        decoded.insert("type".to_string(), "MifareClassic4K".to_string());
        return Some((
            CardType::MifareClassic4K,
            CardData {
                uid: uid.clone(),
                raw: String::new(),
                decoded,
            },
        ));
    }
    if clean.contains("MIFARE Classic") || clean.contains("Classic 1K") {
        decoded.insert("type".to_string(), "MifareClassic1K".to_string());
        return Some((
            CardType::MifareClassic1K,
            CardData {
                uid: uid.clone(),
                raw: String::new(),
                decoded,
            },
        ));
    }

    None
}

// ---------------------------------------------------------------------------
// Autopwn line parser (streaming, called per-line during hf mf autopwn)
// ---------------------------------------------------------------------------

/// Parse a single line from `hf mf autopwn` streaming output.
/// Returns `Some(AutopwnEvent)` if the line contains a recognizable progress marker.
/// Called by the `on_line` callback in `run_command_streaming()`.
pub fn parse_autopwn_line(line: &str) -> Option<AutopwnEvent> {
    let clean = strip_ansi(line);
    let trimmed = clean.trim();

    if trimmed.is_empty() {
        return None;
    }

    // "all key recovery attempts failed" — total failure
    if AUTOPWN_FAIL_RE.is_match(trimmed) {
        return Some(AutopwnEvent::Failed {
            reason: "All key recovery attempts failed".to_string(),
        });
    }

    // "autopwn execution time: 45 seconds" — final marker
    if let Some(caps) = AUTOPWN_TIME_RE.captures(trimmed) {
        let secs: u32 = caps[1].parse().unwrap_or(0);
        return Some(AutopwnEvent::Finished { time_secs: secs });
    }

    // "Succeeded in dumping all blocks" — full dump, look for file path
    if AUTOPWN_DUMP_OK_RE.is_match(trimmed) {
        return Some(AutopwnEvent::DumpComplete {
            file_path: String::new(), // file path parsed from a later line
        });
    }

    // "Dump file is PARTIAL complete"
    if AUTOPWN_DUMP_PARTIAL_RE.is_match(trimmed) {
        return Some(AutopwnEvent::DumpPartial {
            file_path: String::new(),
        });
    }

    // "saved 64 blocks to file hf-mf-01020304-dump.bin"
    if let Some(caps) = AUTOPWN_DUMP_SAVED_RE.captures(trimmed) {
        let path = caps[1].to_string();
        return Some(AutopwnEvent::DumpComplete { file_path: path });
    }

    // "found valid key [ FFFFFFFFFFFF ]" — individual key
    if let Some(caps) = AUTOPWN_KEY_FOUND_RE.captures(trimmed) {
        return Some(AutopwnEvent::KeyFound {
            key: caps[1].to_uppercase(),
        });
    }

    // "found 12/32 keys (D)" — dictionary progress
    if let Some(caps) = AUTOPWN_KEYS_RE.captures(trimmed) {
        let found: u32 = caps[1].parse().unwrap_or(0);
        let total: u32 = caps[2].parse().unwrap_or(0);
        return Some(AutopwnEvent::DictionaryProgress { found, total });
    }

    // Attack phase detection
    if trimmed.contains("Darkside attack") || trimmed.contains("darkside") {
        return Some(AutopwnEvent::DarksideStarted);
    }
    if trimmed.contains("Hardnested attack") || trimmed.contains("hardnested") {
        return Some(AutopwnEvent::HardnestedStarted);
    }
    if trimmed.contains("Staticnested") || trimmed.contains("staticnested") || trimmed.contains("static nonce") {
        return Some(AutopwnEvent::StaticnestedStarted);
    }
    // Nested must come after Hardnested/Staticnested to avoid false matches
    if (trimmed.contains("Nested attack") || trimmed.contains("nested authentication"))
        && !trimmed.contains("Hardnested")
        && !trimmed.contains("hardnested")
        && !trimmed.contains("Staticnested")
        && !trimmed.contains("staticnested")
    {
        return Some(AutopwnEvent::NestedStarted);
    }

    None
}

// ---------------------------------------------------------------------------
// Magic card generation detection (from `hf mf info` output)
// ---------------------------------------------------------------------------

/// Parse `hf mf info` output to detect magic card generation.
/// Returns `Some(MagicGeneration)` if magic capabilities are found.
/// Used by blank detection to verify the correct magic card is on the reader.
pub fn parse_magic_detection(output: &str) -> Option<MagicGeneration> {
    let clean = strip_ansi(output);

    let caps = HF_MAGIC_RE.captures(&clean)?;
    let magic_str = caps[1].to_string();
    let lower = magic_str.to_lowercase();

    // Gen4 GDM / USCUID — must check before Gen4 GTU
    if lower.contains("gdm") || lower.contains("uscuid") {
        return Some(MagicGeneration::Gen4GDM);
    }
    // Gen4 GTU / Ultimate
    if lower.contains("gtu") || lower.contains("ultimate") || lower.contains("gen 4") || lower.contains("gen4") {
        return Some(MagicGeneration::Gen4GTU);
    }
    // Gen3 / APDU / UFUID
    if lower.contains("gen 3") || lower.contains("gen3") || lower.contains("apdu") || lower.contains("ufuid") {
        return Some(MagicGeneration::Gen3);
    }
    // Gen2 / CUID
    if lower.contains("gen 2") || lower.contains("gen2") || lower.contains("cuid") {
        return Some(MagicGeneration::Gen2);
    }
    // Gen1a / Gen1b
    if lower.contains("gen 1") || lower.contains("gen1") {
        return Some(MagicGeneration::Gen1a);
    }

    None
}

/// Check if `hf 14a info` output indicates an ISO 14443-A card is present.
/// Returns true if UID, ATQA, or SAK lines are found.
pub fn is_hf_card_present(output: &str) -> bool {
    let clean = strip_ansi(output);
    let lower = clean.to_lowercase();
    lower.contains("uid") && (lower.contains("atqa") || lower.contains("sak"))
}

/// Check if `hf mfu info` output indicates an Ultralight/NTAG magic card.
/// Magic UL/NTAG cards respond to RATS with ATS (genuine never does).
pub fn is_magic_ultralight(output: &str) -> bool {
    let clean = strip_ansi(output);
    let lower = clean.to_lowercase();
    // Magic UL detection: responds to RATS/ATS or identified as magic
    lower.contains("magic") || lower.contains("gen1a") || lower.contains("directwrite")
}

/// Check if `hf iclass info` output indicates an iCLASS/Picopass card is present.
pub fn is_iclass_present(output: &str) -> bool {
    let clean = strip_ansi(output);
    let lower = clean.to_lowercase();
    lower.contains("iclass") || lower.contains("picopass")
}

/// Extract dump file path from PM3 dump/autopwn output.
/// Works with `hf mf autopwn`, `hf mfu dump`, `hf iclass dump`, etc.
/// Matches patterns like "saved ... to file <path>" or "saved ... file <path>".
pub fn extract_dump_file_path(output: &str) -> Option<String> {
    let clean = strip_ansi(output);
    for line in clean.lines() {
        if let Some(caps) = AUTOPWN_DUMP_SAVED_RE.captures(line) {
            return Some(caps[1].to_string());
        }
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
    } else if let Some(caps) = STANDALONE_RAW_RE.captures(clean) {
        // PM3 outputs raw on standalone "[+] raw: <hex>" line without protocol prefix
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

    // Try XSF format first (real PM3 output): "XSF(01)65:01337"
    // VN=decimal, FC=hex (needs conversion to decimal), CN=decimal
    if let Some(caps) = IOPROX_XSF_RE.captures(clean) {
        let vn = caps[1].to_string();
        let fc_hex = &caps[2];
        let cn = caps[3].to_string();
        // Convert FC from hex to decimal for clone command
        let fc_dec = u32::from_str_radix(fc_hex, 16)
            .map(|n| n.to_string())
            .unwrap_or_else(|_| fc_hex.to_string());
        decoded.insert("version".to_string(), vn);
        decoded.insert("facility_code".to_string(), fc_dec.clone());
        decoded.insert("card_number".to_string(), cn.clone());
        // Also capture raw if present
        if let Some(raw_caps) = IOPROX_RAW_RE.captures(clean) {
            decoded.insert("raw".to_string(), raw_caps[1].to_uppercase());
        }
        let uid = format!("FC{}:CN{}", fc_dec, cn);
        return Some((
            CardType::IOProx,
            CardData {
                uid,
                raw: String::new(),
                decoded,
            },
        ));
    }

    // Try FC/CN/VN text format (alternate PM3 output with explicit labels)
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

    // Try single-line format first (Country and National on same line)
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

    // Real PM3 multi-line output: "Animal ID......... 999-123456789012"
    // Country/National on separate lines, but Animal ID has both combined
    if let Some(caps) = FDXB_ANIMAL_ID_RE.captures(clean) {
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
    let is_internal = clean.contains("Internal");
    if is_internal {
        decoded.insert("keri_type".to_string(), "i".to_string());
    } else if clean.contains("MS") {
        decoded.insert("keri_type".to_string(), "m".to_string());
    }

    // Try Internal ID first: "KERI - Internal ID: 12345"
    if let Some(caps) = KERI_INTERNAL_ID_RE.captures(clean) {
        let id = caps[1].to_string();
        decoded.insert("card_number".to_string(), id.clone());
        // Also grab raw if available
        if let Some(raw_caps) = KERI_RE.captures(clean) {
            decoded.insert("raw".to_string(), raw_caps[1].to_uppercase());
        }
        return Some((
            CardType::Keri,
            CardData {
                uid: id.clone(),
                raw: decoded.get("raw").cloned().unwrap_or_default(),
                decoded,
            },
        ));
    }

    // Try MS format: "Descrambled MS - FC: 1 Card: 12544"
    if let Some(caps) = KERI_MS_FC_CN_RE.captures(clean) {
        let fc = caps[1].to_string();
        let cn = caps[2].to_string();
        decoded.insert("facility_code".to_string(), fc.clone());
        decoded.insert("card_number".to_string(), cn.clone());
        decoded.insert("keri_type".to_string(), "m".to_string());
        return Some((
            CardType::Keri,
            CardData {
                uid: format!("FC{}:CN{}", fc, cn),
                raw: String::new(),
                decoded,
            },
        ));
    }

    // Fallback to raw hex
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
    // Note: no strip_ansi here — parse_lf_search already strips ANSI internally.
    if let Some((_, card_data)) = parse_lf_search(clone_output) {
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
    // Note: no strip_ansi here — parse_lf_search already strips ANSI internally.
    if let Some((detected_type, clone_data)) = parse_lf_search(clone_output) {
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

// ===========================================================================
// Unit Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cards::types::CardType;
    use crate::pm3::command_builder::build_clone_command;

    // -----------------------------------------------------------------------
    // Helper: build realistic PM3 `lf search` output
    // -----------------------------------------------------------------------

    /// Wraps a card-specific block with standard PM3 lf search framing.
    fn pm3_lf_search_output(body: &str) -> String {
        format!(
            "[=] NOTE: some demods output possible binary\n\
             [=] if it finds something that looks like a tag\n\
             [=] False Positives ARE possible\n\
             [=]\n\
             [=] Checking for known tags...\n\
             [=]\n\
             {}\n\
             \n\
             [+] Valid ID found!\n",
            body
        )
    }

    // =======================================================================
    // 1. EM4100
    // =======================================================================

    #[test]
    fn parse_em4100() {
        let output = pm3_lf_search_output(
            "[+] EM 410x ID 0F00112233\n\
             [+] EM410x ( RF/64 )\n\
             [=] EM 410x ID 0F00112233 (Full)\n\
             [=]     Possible de:tag ID: 4276803383"
        );
        let (card_type, data) = parse_lf_search(&output).expect("should parse EM4100");
        assert_eq!(card_type, CardType::EM4100);
        assert_eq!(data.uid, "0F00112233");
        assert_eq!(data.decoded.get("id").unwrap(), "0F00112233");
    }

    #[test]
    fn clone_em4100() {
        let output = pm3_lf_search_output(
            "[+] EM 410x ID 0F00112233"
        );
        let (_, data) = parse_lf_search(&output).unwrap();
        let cmd = build_clone_command(&CardType::EM4100, &data.uid, &data.decoded);
        assert_eq!(cmd.unwrap(), "lf em 410x clone --id 0F00112233");
    }

    // =======================================================================
    // 2. HID Prox
    // =======================================================================

    #[test]
    fn parse_hid_prox_fc_cn_raw() {
        let output = pm3_lf_search_output(
            "[+] [H10301] HID Prox H10301 26-bit;  FC: 65  CN: 29334\n\
             [+] raw: 200078BE5E1E"
        );
        let (card_type, data) = parse_lf_search(&output).expect("should parse HID");
        assert_eq!(card_type, CardType::HIDProx);
        assert_eq!(data.decoded.get("facility_code").unwrap(), "65");
        assert_eq!(data.decoded.get("card_number").unwrap(), "29334");
        assert_eq!(data.decoded.get("raw").unwrap(), "200078BE5E1E");
    }

    #[test]
    fn clone_hid_prox_prefers_raw() {
        let output = pm3_lf_search_output(
            "[+] [H10301] HID Prox H10301 26-bit;  FC: 65  CN: 29334\n\
             [+] raw: 200078BE5E1E"
        );
        let (_, data) = parse_lf_search(&output).unwrap();
        let cmd = build_clone_command(&CardType::HIDProx, &data.uid, &data.decoded);
        // Should prefer raw over structured
        assert_eq!(cmd.unwrap(), "lf hid clone -r 200078BE5E1E");
    }

    #[test]
    fn clone_hid_prox_structured_fallback() {
        // No raw available — falls back to structured
        let mut decoded = HashMap::new();
        decoded.insert("facility_code".to_string(), "65".to_string());
        decoded.insert("card_number".to_string(), "29334".to_string());
        decoded.insert("format".to_string(), "H10301".to_string());
        let cmd = build_clone_command(&CardType::HIDProx, "FC65:CN29334", &decoded);
        assert_eq!(cmd.unwrap(), "lf hid clone -w H10301 --fc 65 --cn 29334");
    }

    // =======================================================================
    // 3. Indala
    // =======================================================================

    #[test]
    fn parse_indala_raw() {
        let output = pm3_lf_search_output(
            "[+] Indala (len 64)  Raw: A0000000A0000000\n\
             [=] Indala ID: 12345678"
        );
        let (card_type, data) = parse_lf_search(&output).expect("should parse Indala");
        assert_eq!(card_type, CardType::Indala);
        assert_eq!(data.decoded.get("raw").unwrap(), "A0000000A0000000");
    }

    #[test]
    fn clone_indala() {
        let output = pm3_lf_search_output(
            "[+] Indala (len 64)  Raw: A0000000A0000000"
        );
        let (_, data) = parse_lf_search(&output).unwrap();
        let cmd = build_clone_command(&CardType::Indala, &data.uid, &data.decoded);
        assert_eq!(cmd.unwrap(), "lf indala clone --raw A0000000A0000000");
    }

    // =======================================================================
    // 4. AWID
    // =======================================================================

    #[test]
    fn parse_awid() {
        let output = pm3_lf_search_output(
            "[+] AWID 26 bit;  FC: 50  CN: 1234"
        );
        let (card_type, data) = parse_lf_search(&output).expect("should parse AWID");
        assert_eq!(card_type, CardType::AWID);
        assert_eq!(data.decoded.get("facility_code").unwrap(), "50");
        assert_eq!(data.decoded.get("card_number").unwrap(), "1234");
        assert_eq!(data.decoded.get("format").unwrap(), "26");
    }

    #[test]
    fn clone_awid() {
        let output = pm3_lf_search_output(
            "[+] AWID 26 bit;  FC: 50  CN: 1234"
        );
        let (_, data) = parse_lf_search(&output).unwrap();
        let cmd = build_clone_command(&CardType::AWID, &data.uid, &data.decoded);
        assert_eq!(cmd.unwrap(), "lf awid clone --fmt 26 --fc 50 --cn 1234");
    }

    #[test]
    fn parse_awid_real_pm3_output() {
        // Real PM3 output: "AWID - len: 26 FC: 50 Card: 1234 - Wiegand: 26409a4, Raw: 011db288..."
        let output = pm3_lf_search_output(
            "[+] AWID - len: 26 FC: 50 Card: 1234 - Wiegand: 26409a4, Raw: 011db2881474411111111111"
        );
        let (card_type, data) = parse_lf_search(&output).expect("should parse AWID real output");
        assert_eq!(card_type, CardType::AWID);
        assert_eq!(data.decoded.get("facility_code").unwrap(), "50");
        assert_eq!(data.decoded.get("card_number").unwrap(), "1234");
        assert_eq!(data.decoded.get("format").unwrap(), "26");
    }

    #[test]
    fn clone_awid_fails_without_fields() {
        // AWID has no raw fallback
        let decoded = HashMap::new();
        let cmd = build_clone_command(&CardType::AWID, "ABCD1234", &decoded);
        assert!(cmd.is_none(), "AWID without FC/CN should return None");
    }

    // =======================================================================
    // 5. IO Prox
    // =======================================================================

    #[test]
    fn parse_ioprox() {
        // Test FC/CN text format (without XSF prefix) — fallback path
        let output = pm3_lf_search_output(
            "[+] IO Prox  FC: 101  CN: 1337"
        );
        let (card_type, data) = parse_lf_search(&output).expect("should parse IOProx FC/CN");
        assert_eq!(card_type, CardType::IOProx);
        assert_eq!(data.decoded.get("facility_code").unwrap(), "101");
        assert_eq!(data.decoded.get("card_number").unwrap(), "1337");
    }

    #[test]
    fn clone_ioprox() {
        // Test FC/CN text format clone (fallback path)
        let output = pm3_lf_search_output(
            "[+] IO Prox  FC: 101  CN: 1337"
        );
        let (_, data) = parse_lf_search(&output).unwrap();
        let cmd = build_clone_command(&CardType::IOProx, &data.uid, &data.decoded);
        assert_eq!(cmd.unwrap(), "lf io clone --vn 0 --fc 101 --cn 1337");
    }

    #[test]
    fn parse_ioprox_real_pm3_output() {
        // Real PM3 output: "IO Prox - XSF(01)65:01337, Raw: 007859603059cdaf ( ok )"
        let output = pm3_lf_search_output(
            "[+] IO Prox - XSF(01)65:01337, Raw: 007859603059cdaf ( ok )"
        );
        let (card_type, data) = parse_lf_search(&output).expect("should parse IOProx XSF");
        assert_eq!(card_type, CardType::IOProx);
        // VN and CN captured as-is from XSF format (leading zeros preserved)
        assert_eq!(data.decoded.get("version").unwrap(), "01");
        // FC 0x65 = 101 decimal
        assert_eq!(data.decoded.get("facility_code").unwrap(), "101");
        // CN from XSF format preserves leading zero: "01337"
        assert_eq!(data.decoded.get("card_number").unwrap(), "01337");
    }

    #[test]
    fn clone_ioprox_real_pm3_output() {
        let output = pm3_lf_search_output(
            "[+] IO Prox - XSF(01)65:01337, Raw: 007859603059cdaf ( ok )"
        );
        let (_, data) = parse_lf_search(&output).unwrap();
        let cmd = build_clone_command(&CardType::IOProx, &data.uid, &data.decoded);
        assert_eq!(cmd.unwrap(), "lf io clone --vn 1 --fc 101 --cn 1337");
    }

    // =======================================================================
    // 6. FDX-B
    // =======================================================================

    #[test]
    fn parse_fdxb() {
        // FDXB_RE uses `.*?` which doesn't span newlines — all fields must be on same line
        let output = pm3_lf_search_output(
            "[+] FDX-B / ISO 11784/11785 - Animal  Country: 999  National ID: 123456789012"
        );
        let (card_type, data) = parse_lf_search(&output).expect("should parse FDX-B");
        assert_eq!(card_type, CardType::FDX_B);
        assert_eq!(data.decoded.get("country").unwrap(), "999");
        assert_eq!(data.decoded.get("national_id").unwrap(), "123456789012");
    }

    #[test]
    fn clone_fdxb() {
        let output = pm3_lf_search_output(
            "[+] FDX-B / ISO 11784/11785 - Animal  Country: 999  National ID: 123456789012"
        );
        let (_, data) = parse_lf_search(&output).unwrap();
        let cmd = build_clone_command(&CardType::FDX_B, &data.uid, &data.decoded);
        assert_eq!(
            cmd.unwrap(),
            "lf fdxb clone --country 999 --national 123456789012"
        );
    }

    #[test]
    fn parse_fdxb_real_pm3_output() {
        // Real PM3 output: multi-line, Country/National on separate lines,
        // but Animal ID has both on one line: "999-123456789012"
        let output = pm3_lf_search_output(
            "[+] FDX-B / ISO 11784/5 Animal\n\
             [+] Animal ID......... 999-123456789012\n\
             [+] National Code..... 123456789012 ( 0x1CBE991A14 )\n\
             [+] Country Code...... 999 - Test range\n\
             [+] Raw............... 28 58 99 7D 3B 9F 00 00 C0 CC 00 00 00"
        );
        let (card_type, data) = parse_lf_search(&output).expect("should parse FDX-B real output");
        assert_eq!(card_type, CardType::FDX_B);
        assert_eq!(data.decoded.get("country").unwrap(), "999");
        assert_eq!(data.decoded.get("national_id").unwrap(), "123456789012");
    }

    #[test]
    fn clone_fdxb_real_pm3_output() {
        let output = pm3_lf_search_output(
            "[+] FDX-B / ISO 11784/5 Animal\n\
             [+] Animal ID......... 999-123456789012\n\
             [+] National Code..... 123456789012 ( 0x1CBE991A14 )\n\
             [+] Country Code...... 999 - Test range"
        );
        let (_, data) = parse_lf_search(&output).unwrap();
        let cmd = build_clone_command(&CardType::FDX_B, &data.uid, &data.decoded);
        assert_eq!(
            cmd.unwrap(),
            "lf fdxb clone --country 999 --national 123456789012"
        );
    }

    // =======================================================================
    // 7. Paradox
    // =======================================================================

    #[test]
    fn parse_paradox_fc_cn() {
        let output = pm3_lf_search_output(
            "[+] Paradox - FC: 96  Card: 40426  Raw: 0F0A00009E3A"
        );
        let (card_type, data) = parse_lf_search(&output).expect("should parse Paradox");
        assert_eq!(card_type, CardType::Paradox);
        assert_eq!(data.decoded.get("facility_code").unwrap(), "96");
        assert_eq!(data.decoded.get("card_number").unwrap(), "40426");
        assert_eq!(data.decoded.get("raw").unwrap(), "0F0A00009E3A");
    }

    #[test]
    fn clone_paradox() {
        let output = pm3_lf_search_output(
            "[+] Paradox - FC: 96  Card: 40426  Raw: 0F0A00009E3A"
        );
        let (_, data) = parse_lf_search(&output).unwrap();
        let cmd = build_clone_command(&CardType::Paradox, &data.uid, &data.decoded);
        // Paradox prefers FC/CN
        assert_eq!(cmd.unwrap(), "lf paradox clone --fc 96 --cn 40426");
    }

    // =======================================================================
    // 8. Presco
    // =======================================================================

    #[test]
    fn parse_presco_hex() {
        let output = pm3_lf_search_output(
            "[+] Presco - Card: 001CA7E6A"
        );
        let (card_type, data) = parse_lf_search(&output).expect("should parse Presco");
        assert_eq!(card_type, CardType::Presco);
        assert_eq!(data.decoded.get("hex").unwrap(), "001CA7E6A");
    }

    #[test]
    fn parse_presco_site_user() {
        let output = pm3_lf_search_output(
            "[+] Presco - Site: 42  User: 1337  Card: 001CA7E6A"
        );
        let (card_type, data) = parse_lf_search(&output).expect("should parse Presco SC/UC");
        assert_eq!(card_type, CardType::Presco);
        assert_eq!(data.decoded.get("site_code").unwrap(), "42");
        assert_eq!(data.decoded.get("user_code").unwrap(), "1337");
    }

    #[test]
    fn clone_presco_hex() {
        let mut decoded = HashMap::new();
        decoded.insert("hex".to_string(), "001CA7E6A".to_string());
        let cmd = build_clone_command(&CardType::Presco, "001CA7E6A", &decoded);
        assert_eq!(cmd.unwrap(), "lf presco clone -d 001CA7E6A");
    }

    #[test]
    fn clone_presco_site_user() {
        let mut decoded = HashMap::new();
        decoded.insert("site_code".to_string(), "42".to_string());
        decoded.insert("user_code".to_string(), "1337".to_string());
        let cmd = build_clone_command(&CardType::Presco, "SC42:UC1337", &decoded);
        assert_eq!(cmd.unwrap(), "lf presco clone --sitecode 42 --usercode 1337");
    }

    #[test]
    fn parse_presco_real_pm3_output() {
        // Real PM3: "Presco Site code: 0 User code: 57470 Full code: 0031E07E Raw: ..."
        let output = pm3_lf_search_output(
            "[+] Presco Site code: 0 User code: 57470 Full code: 0031E07E Raw: 10D0000000000000000000000031E07E"
        );
        let (card_type, data) = parse_lf_search(&output).expect("should parse Presco real output");
        assert_eq!(card_type, CardType::Presco);
        assert_eq!(data.decoded.get("site_code").unwrap(), "0");
        assert_eq!(data.decoded.get("user_code").unwrap(), "57470");
    }

    #[test]
    fn clone_presco_real_pm3_output() {
        let output = pm3_lf_search_output(
            "[+] Presco Site code: 0 User code: 57470 Full code: 0031E07E Raw: 10D0000000000000000000000031E07E"
        );
        let (_, data) = parse_lf_search(&output).unwrap();
        let cmd = build_clone_command(&CardType::Presco, &data.uid, &data.decoded);
        assert_eq!(cmd.unwrap(), "lf presco clone --sitecode 0 --usercode 57470");
    }

    // =======================================================================
    // 9. Viking
    // =======================================================================

    #[test]
    fn parse_viking_raw() {
        let output = pm3_lf_search_output(
            "[+] Viking tag  Raw: 1A2B3C4D"
        );
        let (card_type, data) = parse_lf_search(&output).expect("should parse Viking");
        assert_eq!(card_type, CardType::Viking);
        assert_eq!(data.uid, "1A2B3C4D");
    }

    #[test]
    fn parse_viking_id() {
        let output = pm3_lf_search_output(
            "[+] Viking Card ID: 12345\n\
             [=] Viking Raw: 1A2B3C4D"
        );
        let (card_type, data) = parse_lf_search(&output).expect("should parse Viking ID");
        assert_eq!(card_type, CardType::Viking);
        assert_eq!(data.decoded.get("card_id").unwrap(), "12345");
        assert_eq!(data.decoded.get("raw").unwrap(), "1A2B3C4D");
    }

    #[test]
    fn clone_viking() {
        let cmd = build_clone_command(
            &CardType::Viking,
            "1A2B3C4D",
            &HashMap::new(),
        );
        assert_eq!(cmd.unwrap(), "lf viking clone --cn 1A2B3C4D");
    }

    #[test]
    fn parse_viking_real_pm3_output() {
        // Real PM3: "Viking - Card 1A2B3C4D, Raw: F200001A2B3C4D1A"
        let output = pm3_lf_search_output(
            "[+] Viking - Card 1A2B3C4D, Raw: F200001A2B3C4D1A"
        );
        let (card_type, data) = parse_lf_search(&output).expect("should parse Viking real output");
        assert_eq!(card_type, CardType::Viking);
        assert_eq!(data.decoded.get("card_id").unwrap(), "1A2B3C4D");
        assert_eq!(data.decoded.get("raw").unwrap(), "F200001A2B3C4D1A");
    }

    // =======================================================================
    // 10. Pyramid
    // =======================================================================

    #[test]
    fn parse_pyramid_fc_cn() {
        let output = pm3_lf_search_output(
            "[+] Pyramid - len: 26, FC: 123, Card: 4567, Raw: AABBCCDD"
        );
        let (card_type, data) = parse_lf_search(&output).expect("should parse Pyramid");
        assert_eq!(card_type, CardType::Pyramid);
        assert_eq!(data.decoded.get("facility_code").unwrap(), "123");
        assert_eq!(data.decoded.get("card_number").unwrap(), "4567");
        assert_eq!(data.decoded.get("raw").unwrap(), "AABBCCDD");
    }

    #[test]
    fn clone_pyramid() {
        let output = pm3_lf_search_output(
            "[+] Pyramid - len: 26, FC: 123, Card: 4567, Raw: AABBCCDD"
        );
        let (_, data) = parse_lf_search(&output).unwrap();
        let cmd = build_clone_command(&CardType::Pyramid, &data.uid, &data.decoded);
        assert_eq!(cmd.unwrap(), "lf pyramid clone --fc 123 --cn 4567");
    }

    // =======================================================================
    // 11. Nedap
    // =======================================================================

    #[test]
    fn parse_nedap() {
        let output = pm3_lf_search_output(
            "[+] Nedap - Card: 12345  Subtype: 1"
        );
        let (card_type, data) = parse_lf_search(&output).expect("should parse Nedap");
        assert_eq!(card_type, CardType::Nedap);
        assert_eq!(data.decoded.get("card_number").unwrap(), "12345");
        assert_eq!(data.decoded.get("subtype").unwrap(), "1");
    }

    #[test]
    fn parse_nedap_no_subtype() {
        let output = pm3_lf_search_output(
            "[+] NEDAP - Card: 99999"
        );
        let (card_type, data) = parse_lf_search(&output).expect("should parse Nedap no-sub");
        assert_eq!(card_type, CardType::Nedap);
        assert_eq!(data.decoded.get("card_number").unwrap(), "99999");
        assert_eq!(data.decoded.get("subtype").unwrap(), "5"); // PM3 default subtype is 5
    }

    #[test]
    fn parse_nedap_real_pm3_output() {
        // Real PM3: "NEDAP (64b) - ID: 12345 subtype: 1 customer code: 101 / 0x065 Raw: FF820CA58960F8F3"
        let output = pm3_lf_search_output(
            "[+] NEDAP (64b) - ID: 12345 subtype: 1 customer code: 101 / 0x065 Raw: FF820CA58960F8F3"
        );
        let (card_type, data) = parse_lf_search(&output).expect("should parse Nedap real output");
        assert_eq!(card_type, CardType::Nedap);
        assert_eq!(data.decoded.get("card_number").unwrap(), "12345");
        assert_eq!(data.decoded.get("subtype").unwrap(), "1");
        assert_eq!(data.decoded.get("customer_code").unwrap(), "101");
    }

    #[test]
    fn clone_nedap() {
        let mut decoded = HashMap::new();
        decoded.insert("subtype".to_string(), "1".to_string());
        decoded.insert("customer_code".to_string(), "101".to_string());
        decoded.insert("card_number".to_string(), "12345".to_string());
        let cmd = build_clone_command(&CardType::Nedap, "ST1:CC101:ID12345", &decoded);
        assert_eq!(cmd.unwrap(), "lf nedap clone --st 1 --cc 101 --id 12345");
    }

    #[test]
    fn clone_nedap_real_pm3_output() {
        let output = pm3_lf_search_output(
            "[+] NEDAP (64b) - ID: 12345 subtype: 1 customer code: 101 / 0x065 Raw: FF820CA58960F8F3"
        );
        let (_, data) = parse_lf_search(&output).unwrap();
        let cmd = build_clone_command(&CardType::Nedap, &data.uid, &data.decoded);
        assert_eq!(cmd.unwrap(), "lf nedap clone --st 1 --cc 101 --id 12345");
    }

    #[test]
    fn clone_nedap_fails_without_fields() {
        let decoded = HashMap::new();
        let cmd = build_clone_command(&CardType::Nedap, "ABCD1234", &decoded);
        assert!(cmd.is_none(), "Nedap without required fields should return None");
    }

    // =======================================================================
    // 12. GProxII
    // =======================================================================

    #[test]
    fn parse_gproxii() {
        // Synthetic: old-style format with FC/Card
        let output = pm3_lf_search_output(
            "[+] G-Prox-II - Len: 26 FC: 10 Card: 1234 xor: 0"
        );
        let (card_type, data) = parse_lf_search(&output).expect("should parse GProxII");
        assert_eq!(card_type, CardType::GProxII);
        assert_eq!(data.decoded.get("facility_code").unwrap(), "10");
        assert_eq!(data.decoded.get("card_number").unwrap(), "1234");
    }

    #[test]
    fn parse_gproxii_real_pm3_output() {
        // Real PM3: "G-Prox-II - Len: 26 FC: 123 Card: 1234 xor: 141, Raw: fac2a38c2b081af008eb0ac2"
        let output = pm3_lf_search_output(
            "[+] G-Prox-II - Len: 26 FC: 123 Card: 1234 xor: 141, Raw: fac2a38c2b081af008eb0ac2\n\
             \n\
             [+] Valid Guardall G-Prox II ID found!"
        );
        let (card_type, data) = parse_lf_search(&output).expect("should parse GProxII real output");
        assert_eq!(card_type, CardType::GProxII);
        assert_eq!(data.decoded.get("facility_code").unwrap(), "123");
        assert_eq!(data.decoded.get("card_number").unwrap(), "1234");
        assert_eq!(data.decoded.get("xor").unwrap(), "141");
        assert_eq!(data.decoded.get("format").unwrap(), "26");
    }

    #[test]
    fn clone_gproxii() {
        let mut decoded = HashMap::new();
        decoded.insert("facility_code".to_string(), "123".to_string());
        decoded.insert("card_number".to_string(), "1234".to_string());
        decoded.insert("xor".to_string(), "141".to_string());
        decoded.insert("format".to_string(), "26".to_string());
        let cmd = build_clone_command(&CardType::GProxII, "FC123:CN1234", &decoded);
        assert_eq!(cmd.unwrap(), "lf gproxii clone --xor 141 --fmt 26 --fc 123 --cn 1234");
    }

    #[test]
    fn clone_gproxii_real_pm3_output() {
        let output = pm3_lf_search_output(
            "[+] G-Prox-II - Len: 26 FC: 123 Card: 1234 xor: 141, Raw: fac2a38c2b081af008eb0ac2\n\
             \n\
             [+] Valid Guardall G-Prox II ID found!"
        );
        let (_, data) = parse_lf_search(&output).unwrap();
        let cmd = build_clone_command(&CardType::GProxII, &data.uid, &data.decoded);
        assert_eq!(cmd.unwrap(), "lf gproxii clone --xor 141 --fmt 26 --fc 123 --cn 1234");
    }

    #[test]
    fn clone_gproxii_fails_without_fields() {
        let decoded = HashMap::new();
        let cmd = build_clone_command(&CardType::GProxII, "ABCD1234", &decoded);
        assert!(cmd.is_none(), "GProxII without fc+card_number should return None");
    }

    // =======================================================================
    // 13. Keri
    // =======================================================================

    #[test]
    fn parse_keri_internal() {
        // Synthetic: old-style Keri raw format
        let output = pm3_lf_search_output(
            "[+] Keri - Internal Raw: 0000000012345"
        );
        let (card_type, data) = parse_lf_search(&output).expect("should parse Keri");
        assert_eq!(card_type, CardType::Keri);
        assert_eq!(data.decoded.get("keri_type").unwrap(), "i");
    }

    #[test]
    fn parse_keri_ms() {
        let output = pm3_lf_search_output(
            "[+] Keri - MS Raw: ABCDEF1234567"
        );
        let (card_type, data) = parse_lf_search(&output).expect("should parse Keri MS");
        assert_eq!(card_type, CardType::Keri);
        assert_eq!(data.decoded.get("keri_type").unwrap(), "m");
    }

    #[test]
    fn parse_keri_real_pm3_output() {
        // Real PM3: "KERI - Internal ID: 12345, Raw: E000000080003039"
        //           "Descrambled MS - FC: 1 Card: 12544"
        let output = pm3_lf_search_output(
            "[+] KERI - Internal ID: 12345, Raw: E000000080003039\n\
             [+] Descrambled MS - FC: 1 Card: 12544"
        );
        let (card_type, data) = parse_lf_search(&output).expect("should parse Keri real output");
        assert_eq!(card_type, CardType::Keri);
        assert_eq!(data.decoded.get("keri_type").unwrap(), "i");
        assert_eq!(data.decoded.get("card_number").unwrap(), "12345");
        assert_eq!(data.uid, "12345");
    }

    #[test]
    fn clone_keri_internal() {
        let mut decoded = HashMap::new();
        decoded.insert("keri_type".to_string(), "i".to_string());
        decoded.insert("card_number".to_string(), "12345".to_string());
        let cmd = build_clone_command(&CardType::Keri, "12345", &decoded);
        assert_eq!(cmd.unwrap(), "lf keri clone -t i --cn 12345");
    }

    #[test]
    fn clone_keri_real_pm3_output() {
        let output = pm3_lf_search_output(
            "[+] KERI - Internal ID: 12345, Raw: E000000080003039\n\
             [+] Descrambled MS - FC: 1 Card: 12544"
        );
        let (_, data) = parse_lf_search(&output).unwrap();
        let cmd = build_clone_command(&CardType::Keri, &data.uid, &data.decoded);
        assert_eq!(cmd.unwrap(), "lf keri clone -t i --cn 12345");
    }

    // =======================================================================
    // 14. Gallagher
    // =======================================================================

    #[test]
    fn parse_gallagher_single_line() {
        let output = pm3_lf_search_output(
            "[+] Gallagher - Region Code: 1  Facility Code: 22  Card Number: 3333  Issue Level: 1"
        );
        let (card_type, data) = parse_lf_search(&output).expect("should parse Gallagher");
        assert_eq!(card_type, CardType::Gallagher);
        assert_eq!(data.decoded.get("region_code").unwrap(), "1");
        assert_eq!(data.decoded.get("facility_code").unwrap(), "22");
        assert_eq!(data.decoded.get("card_number").unwrap(), "3333");
        assert_eq!(data.decoded.get("issue_level").unwrap(), "1");
    }

    #[test]
    fn parse_gallagher_multi_line() {
        let output = pm3_lf_search_output(
            "[+] Gallagher Tag Detected\n\
             [=]   Region Code: 5\n\
             [=]   Facility Code: 100\n\
             [=]   Card Number: 54321\n\
             [=]   Issue Level: 2"
        );
        let (card_type, data) = parse_lf_search(&output).expect("should parse Gallagher multi-line");
        assert_eq!(card_type, CardType::Gallagher);
        assert_eq!(data.decoded.get("region_code").unwrap(), "5");
        assert_eq!(data.decoded.get("facility_code").unwrap(), "100");
        assert_eq!(data.decoded.get("card_number").unwrap(), "54321");
        assert_eq!(data.decoded.get("issue_level").unwrap(), "2");
    }

    #[test]
    fn clone_gallagher() {
        let mut decoded = HashMap::new();
        decoded.insert("region_code".to_string(), "1".to_string());
        decoded.insert("facility_code".to_string(), "22".to_string());
        decoded.insert("card_number".to_string(), "3333".to_string());
        decoded.insert("issue_level".to_string(), "1".to_string());
        let cmd = build_clone_command(&CardType::Gallagher, "RC1:FC22:CN3333:IL1", &decoded);
        assert_eq!(
            cmd.unwrap(),
            "lf gallagher clone --rc 1 --fc 22 --cn 3333 --il 1"
        );
    }

    #[test]
    fn clone_gallagher_fails_without_fields() {
        let decoded = HashMap::new();
        let cmd = build_clone_command(&CardType::Gallagher, "ABCD1234", &decoded);
        assert!(cmd.is_none(), "Gallagher without all 4 fields should return None");
    }

    #[test]
    fn parse_gallagher_real_pm3_output() {
        // Real PM3 v4.20728 `lf search` output — note "Region:" (no "Code"),
        // "Facility:" (no "Code"), "Card No.:" (not "Card Number:")
        let output = pm3_lf_search_output(
            "[+] GALLAGHER - Region: 1 Facility: 22 Card No.: 3333 Issue Level: 1\n\
             [+]    Displayed: B22\n\
             [+]    Raw: 7FEAA35854B86B0D1A8CB120\n\
             [+]    CRC: 20 - 20 (ok)\n\
             [+] Valid GALLAGHER ID found!",
        );
        let (card_type, data) = parse_lf_search(&output).expect("should parse real GALLAGHER");
        assert_eq!(card_type, CardType::Gallagher);
        assert_eq!(data.decoded.get("region_code").unwrap(), "1");
        assert_eq!(data.decoded.get("facility_code").unwrap(), "22");
        assert_eq!(data.decoded.get("card_number").unwrap(), "3333");
        assert_eq!(data.decoded.get("issue_level").unwrap(), "1");
    }

    #[test]
    fn clone_gallagher_real_pm3_output() {
        // Round-trip: parse real PM3 output → build clone command
        let output = pm3_lf_search_output(
            "[+] GALLAGHER - Region: 1 Facility: 22 Card No.: 3333 Issue Level: 1",
        );
        let (card_type, data) = parse_lf_search(&output).expect("parse");
        let cmd = build_clone_command(&card_type, &data.uid, &data.decoded);
        assert_eq!(
            cmd.unwrap(),
            "lf gallagher clone --rc 1 --fc 22 --cn 3333 --il 1"
        );
    }

    // =======================================================================
    // 15. PAC/Stanley
    // =======================================================================

    #[test]
    fn parse_pac_card_number() {
        let output = pm3_lf_search_output(
            "[+] PAC/Stanley tag found\n\
             [=] PAC/Stanley Card: 16720198"
        );
        let (card_type, data) = parse_lf_search(&output).expect("should parse PAC");
        assert_eq!(card_type, CardType::PAC);
        assert_eq!(data.decoded.get("card_number").unwrap(), "16720198");
    }

    #[test]
    fn parse_pac_raw() {
        let output = pm3_lf_search_output(
            "[+] PAC/Stanley tag found\n\
             [=] PAC/Stanley Raw: FF2049AABBCCDD"
        );
        let (card_type, data) = parse_lf_search(&output).expect("should parse PAC raw");
        assert_eq!(card_type, CardType::PAC);
        assert_eq!(data.decoded.get("raw").unwrap(), "FF2049AABBCCDD");
    }

    #[test]
    fn clone_pac_with_raw() {
        let mut decoded = HashMap::new();
        decoded.insert("raw".to_string(), "FF2049AABBCCDD".to_string());
        let cmd = build_clone_command(&CardType::PAC, "FF2049", &decoded);
        assert_eq!(cmd.unwrap(), "lf pac clone --raw FF2049AABBCCDD");
    }

    #[test]
    fn parse_pac_real_pm3_output() {
        // Real PM3 v4.20728 `lf search` output — Card ID is hex, not decimal
        let output = pm3_lf_search_output(
            "[+] PAC/Stanley - Card: CD4F5552, Raw: FF2049906D8511C593155B56D5B2649F",
        );
        let (card_type, data) = parse_lf_search(&output).expect("should parse real PAC");
        assert_eq!(card_type, CardType::PAC);
        assert_eq!(data.decoded.get("card_number").unwrap(), "CD4F5552");
        assert_eq!(
            data.decoded.get("raw").unwrap(),
            "FF2049906D8511C593155B56D5B2649F"
        );
    }

    #[test]
    fn clone_pac_real_pm3_output() {
        // Round-trip: parse → clone. Builder prefers raw when available
        let output = pm3_lf_search_output(
            "[+] PAC/Stanley - Card: CD4F5552, Raw: FF2049906D8511C593155B56D5B2649F",
        );
        let (card_type, data) = parse_lf_search(&output).expect("parse");
        let cmd = build_clone_command(&card_type, &data.uid, &data.decoded);
        assert_eq!(
            cmd.unwrap(),
            "lf pac clone --raw FF2049906D8511C593155B56D5B2649F"
        );
    }

    // =======================================================================
    // 16. Noralsy
    // =======================================================================

    #[test]
    fn parse_noralsy_card_raw() {
        let output = pm3_lf_search_output(
            "[+] Noralsy - Card: 112233  Year: 2023\n\
             [=] Noralsy Raw: 002C180000000000"
        );
        let (card_type, data) = parse_lf_search(&output).expect("should parse Noralsy");
        assert_eq!(card_type, CardType::Noralsy);
        assert_eq!(data.decoded.get("card_number").unwrap(), "112233");
        assert_eq!(data.decoded.get("year").unwrap(), "2023");
        assert_eq!(data.decoded.get("raw").unwrap(), "002C180000000000");
    }

    #[test]
    fn clone_noralsy() {
        let mut decoded = HashMap::new();
        decoded.insert("card_number".to_string(), "112233".to_string());
        decoded.insert("year".to_string(), "2023".to_string());
        let cmd = build_clone_command(&CardType::Noralsy, "112233", &decoded);
        assert_eq!(cmd.unwrap(), "lf noralsy clone --cn 112233 -y 2023");
    }

    #[test]
    fn parse_noralsy_real_pm3_output() {
        // Real PM3 v4.20728 `lf search` output
        let output = pm3_lf_search_output(
            "[+] Noralsy - Card: 112233, Year: 2000, Raw: BB0214FF0110002233070000",
        );
        let (card_type, data) = parse_lf_search(&output).expect("should parse real Noralsy");
        assert_eq!(card_type, CardType::Noralsy);
        assert_eq!(data.decoded.get("card_number").unwrap(), "112233");
        assert_eq!(data.decoded.get("year").unwrap(), "2000");
        assert_eq!(
            data.decoded.get("raw").unwrap(),
            "BB0214FF0110002233070000"
        );
    }

    #[test]
    fn clone_noralsy_real_pm3_output() {
        // Round-trip: parse → clone — uses --cn (not --raw, which doesn't exist)
        let output = pm3_lf_search_output(
            "[+] Noralsy - Card: 112233, Year: 2000, Raw: BB0214FF0110002233070000",
        );
        let (card_type, data) = parse_lf_search(&output).expect("parse");
        let cmd = build_clone_command(&card_type, &data.uid, &data.decoded);
        assert_eq!(cmd.unwrap(), "lf noralsy clone --cn 112233 -y 2000");
    }

    // =======================================================================
    // 17. Jablotron
    // =======================================================================

    #[test]
    fn parse_jablotron() {
        let output = pm3_lf_search_output(
            "[+] Jablotron - Card: 112233"
        );
        let (card_type, data) = parse_lf_search(&output).expect("should parse Jablotron");
        assert_eq!(card_type, CardType::Jablotron);
        assert_eq!(data.decoded.get("card_number").unwrap(), "112233");
    }

    #[test]
    fn clone_jablotron() {
        let mut decoded = HashMap::new();
        decoded.insert("card_number".to_string(), "112233".to_string());
        let cmd = build_clone_command(&CardType::Jablotron, "112233", &decoded);
        assert_eq!(cmd.unwrap(), "lf jablotron clone --cn 112233");
    }

    #[test]
    fn parse_jablotron_real_pm3_output() {
        // Real PM3 v4.20728 `lf search` output — Card is internal ID (not FullCode)
        let output = pm3_lf_search_output(
            "[+] Jablotron - Card: 1b669, Raw: FFFF00001122335C\n\
             [+] Printed: 1410-00-0011-2233",
        );
        let (card_type, data) = parse_lf_search(&output).expect("should parse real Jablotron");
        assert_eq!(card_type, CardType::Jablotron);
        assert_eq!(data.decoded.get("card_number").unwrap(), "1B669");
    }

    #[test]
    fn clone_jablotron_real_pm3_output() {
        // Round-trip: parse → clone
        let output = pm3_lf_search_output(
            "[+] Jablotron - Card: 1b669, Raw: FFFF00001122335C",
        );
        let (card_type, data) = parse_lf_search(&output).expect("parse");
        let cmd = build_clone_command(&card_type, &data.uid, &data.decoded);
        assert_eq!(cmd.unwrap(), "lf jablotron clone --cn 1B669");
    }

    // =======================================================================
    // 18. SecuraKey
    // =======================================================================

    #[test]
    fn parse_securakey() {
        let output = pm3_lf_search_output(
            "[+] SecuraKey tag found\n\
             [=] Securakey Raw: 7FCB400001ADEA5344300000"
        );
        let (card_type, data) = parse_lf_search(&output).expect("should parse SecuraKey");
        assert_eq!(card_type, CardType::SecuraKey);
        assert_eq!(data.decoded.get("raw").unwrap(), "7FCB400001ADEA5344300000");
    }

    #[test]
    fn clone_securakey() {
        let mut decoded = HashMap::new();
        decoded.insert("raw".to_string(), "7FCB400001ADEA5344300000".to_string());
        let cmd = build_clone_command(&CardType::SecuraKey, "7FCB400001ADEA5344300000", &decoded);
        assert_eq!(cmd.unwrap(), "lf securakey clone --raw 7FCB400001ADEA5344300000");
    }

    #[test]
    fn parse_securakey_real_pm3_output() {
        // Real PM3 v4.20728 `lf search` output — includes FC/Card/len before Raw
        let output = pm3_lf_search_output(
            "[+] Securakey - len: 26 FC: 0x35 Card: 64169, Raw: 7FCB400001ADEA5344300000\n\
             [+] Wiegand: 006BF553 parity ( ok )",
        );
        let (card_type, data) = parse_lf_search(&output).expect("should parse real Securakey");
        assert_eq!(card_type, CardType::SecuraKey);
        assert_eq!(
            data.decoded.get("raw").unwrap(),
            "7FCB400001ADEA5344300000"
        );
    }

    #[test]
    fn clone_securakey_real_pm3_output() {
        // Round-trip: parse → clone
        let output = pm3_lf_search_output(
            "[+] Securakey - len: 26 FC: 0x35 Card: 64169, Raw: 7FCB400001ADEA5344300000",
        );
        let (card_type, data) = parse_lf_search(&output).expect("parse");
        let cmd = build_clone_command(&card_type, &data.uid, &data.decoded);
        assert_eq!(
            cmd.unwrap(),
            "lf securakey clone --raw 7FCB400001ADEA5344300000"
        );
    }

    // =======================================================================
    // 19. Visa2000
    // =======================================================================

    #[test]
    fn parse_visa2000() {
        let output = pm3_lf_search_output(
            "[+] Visa2000 - Card: 112233"
        );
        let (card_type, data) = parse_lf_search(&output).expect("should parse Visa2000");
        assert_eq!(card_type, CardType::Visa2000);
        assert_eq!(data.decoded.get("card_number").unwrap(), "112233");
    }

    #[test]
    fn clone_visa2000() {
        let mut decoded = HashMap::new();
        decoded.insert("card_number".to_string(), "112233".to_string());
        let cmd = build_clone_command(&CardType::Visa2000, "112233", &decoded);
        assert_eq!(cmd.unwrap(), "lf visa2000 clone --cn 112233");
    }

    #[test]
    fn clone_visa2000_fails_without_card_number() {
        let decoded = HashMap::new();
        let cmd = build_clone_command(&CardType::Visa2000, "ABCD1234", &decoded);
        assert!(cmd.is_none(), "Visa2000 without numeric card_number should return None");
    }

    #[test]
    fn parse_visa2000_real_pm3_output() {
        // Real PM3 v4.20728 `lf search` output
        let output = pm3_lf_search_output(
            "[+] Visa2000 - Card 112233, Raw: 564953320001B66900000183",
        );
        let (card_type, data) = parse_lf_search(&output).expect("should parse real Visa2000");
        assert_eq!(card_type, CardType::Visa2000);
        assert_eq!(data.decoded.get("card_number").unwrap(), "112233");
    }

    #[test]
    fn clone_visa2000_real_pm3_output() {
        // Round-trip: parse → clone
        let output = pm3_lf_search_output(
            "[+] Visa2000 - Card 112233, Raw: 564953320001B66900000183",
        );
        let (card_type, data) = parse_lf_search(&output).expect("parse");
        let cmd = build_clone_command(&card_type, &data.uid, &data.decoded);
        assert_eq!(cmd.unwrap(), "lf visa2000 clone --cn 112233");
    }

    // =======================================================================
    // 20. Motorola
    // =======================================================================

    #[test]
    fn parse_motorola() {
        let output = pm3_lf_search_output(
            "[+] Motorola tag found\n\
             [=] Motorola Raw: 0000000100000000"
        );
        let (card_type, data) = parse_lf_search(&output).expect("should parse Motorola");
        assert_eq!(card_type, CardType::Motorola);
        assert_eq!(data.decoded.get("raw").unwrap(), "0000000100000000");
    }

    #[test]
    fn clone_motorola() {
        let mut decoded = HashMap::new();
        decoded.insert("raw".to_string(), "0000000100000000".to_string());
        let cmd = build_clone_command(&CardType::Motorola, "0000000100000000", &decoded);
        assert_eq!(cmd.unwrap(), "lf motorola clone --raw 0000000100000000");
    }

    // =======================================================================
    // 21. IDTECK
    // =======================================================================

    #[test]
    fn parse_idteck() {
        let output = pm3_lf_search_output(
            "[+] IDTECK tag found\n\
             [=] IDTECK Raw: 4944544B351FBE4B"
        );
        let (card_type, data) = parse_lf_search(&output).expect("should parse IDTECK");
        assert_eq!(card_type, CardType::IDTECK);
        assert_eq!(data.decoded.get("raw").unwrap(), "4944544B351FBE4B");
    }

    #[test]
    fn clone_idteck() {
        let mut decoded = HashMap::new();
        decoded.insert("raw".to_string(), "4944544B351FBE4B".to_string());
        let cmd = build_clone_command(&CardType::IDTECK, "4944544B351FBE4B", &decoded);
        assert_eq!(cmd.unwrap(), "lf idteck clone --raw 4944544B351FBE4B");
    }

    #[test]
    fn parse_idteck_real_pm3_output() {
        // Real PM3 v4.20728 `lf search` output — includes Card ID decimal + hex + Raw
        let output = pm3_lf_search_output(
            "[+] IDTECK Tag Found: Card ID 4963871 ( 0x4BBE1F ) Raw: 4944544B351FBE4B  chksum 0x35 ( fail )\n\
             [+] [H10301  ] HID H10301 26-bit                FC: 37  CN: 57103  parity ( ok )",
        );
        let (card_type, data) = parse_lf_search(&output).expect("should parse real IDTECK");
        assert_eq!(card_type, CardType::IDTECK);
        assert_eq!(
            data.decoded.get("raw").unwrap(),
            "4944544B351FBE4B"
        );
    }

    #[test]
    fn clone_idteck_real_pm3_output() {
        // Round-trip: parse → clone
        let output = pm3_lf_search_output(
            "[+] IDTECK Tag Found: Card ID 4963871 ( 0x4BBE1F ) Raw: 4944544B351FBE4B  chksum 0x35 ( fail )",
        );
        let (card_type, data) = parse_lf_search(&output).expect("parse");
        let cmd = build_clone_command(&card_type, &data.uid, &data.decoded);
        assert_eq!(
            cmd.unwrap(),
            "lf idteck clone --raw 4944544B351FBE4B"
        );
    }

    // =======================================================================
    // 22. NexWatch
    // =======================================================================

    #[test]
    fn parse_nexwatch_raw() {
        let output = pm3_lf_search_output(
            "[+] NexWatch tag found\n\
             [=] NexWatch Raw: 5600000000213C9F8F150000"
        );
        let (card_type, data) = parse_lf_search(&output).expect("should parse NexWatch");
        assert_eq!(card_type, CardType::NexWatch);
        assert_eq!(data.decoded.get("raw").unwrap(), "5600000000213C9F8F150000");
    }

    #[test]
    fn parse_nexwatch_id_and_raw() {
        let output = pm3_lf_search_output(
            "[+] NexWatch tag found\n\
             [=] NXT ID: 31337\n\
             [=] NexWatch Raw: 5600000000213C9F8F150000"
        );
        let (card_type, data) = parse_lf_search(&output).expect("should parse NexWatch ID+raw");
        assert_eq!(card_type, CardType::NexWatch);
        assert_eq!(data.decoded.get("card_id").unwrap(), "31337");
        assert_eq!(data.decoded.get("raw").unwrap(), "5600000000213C9F8F150000");
    }

    #[test]
    fn clone_nexwatch() {
        let cmd = build_clone_command(
            &CardType::NexWatch,
            "5600000000213C9F8F150000",
            &HashMap::new(),
        );
        assert_eq!(
            cmd.unwrap(),
            "lf nexwatch clone --raw 5600000000213C9F8F150000"
        );
    }

    #[test]
    fn parse_nexwatch_real_pm3_output() {
        // Real PM3 v4.20728 `lf search` output — multi-line, 88bit id, standalone Raw line
        let output = pm3_lf_search_output(
            "[+]  NexWatch raw id : 0x40c00080\n\
             [+]      fingerprint : Quadrakey\n\
             [+]         88bit id : 521512301 (0x1f15a56d)\n\
             [+]             mode : 1\n\
             [=]  Raw : 5600000000213C9F8F150C00",
        );
        let (card_type, data) = parse_lf_search(&output).expect("should parse real NexWatch");
        assert_eq!(card_type, CardType::NexWatch);
        assert_eq!(data.decoded.get("card_id").unwrap(), "521512301");
        assert_eq!(
            data.decoded.get("raw").unwrap(),
            "5600000000213C9F8F150C00"
        );
    }

    #[test]
    fn clone_nexwatch_real_pm3_output() {
        // Round-trip: parse → clone
        let output = pm3_lf_search_output(
            "[+]  NexWatch raw id : 0x40c00080\n\
             [+]         88bit id : 521512301 (0x1f15a56d)\n\
             [=]  Raw : 5600000000213C9F8F150C00",
        );
        let (card_type, data) = parse_lf_search(&output).expect("parse");
        let cmd = build_clone_command(&card_type, &data.uid, &data.decoded);
        assert_eq!(
            cmd.unwrap(),
            "lf nexwatch clone --raw 5600000000213C9F8F150C00"
        );
    }

    // =======================================================================
    // Non-cloneable LF types (detection only)
    // =======================================================================

    #[test]
    fn parse_cotag() {
        let output = pm3_lf_search_output("[+] COTAG Found");
        let (card_type, _) = parse_lf_search(&output).expect("should detect COTAG");
        assert_eq!(card_type, CardType::COTAG);
        assert!(!card_type.is_cloneable());
    }

    #[test]
    fn parse_em4x50() {
        let output = pm3_lf_search_output("[+] EM4x50 chip detected");
        let (card_type, _) = parse_lf_search(&output).expect("should detect EM4x50");
        assert_eq!(card_type, CardType::EM4x50);
        assert!(!card_type.is_cloneable());
    }

    #[test]
    fn parse_hitag() {
        let output = pm3_lf_search_output("[+] Hitag 2 detected");
        let (card_type, _) = parse_lf_search(&output).expect("should detect Hitag");
        assert_eq!(card_type, CardType::Hitag);
        assert!(!card_type.is_cloneable());
    }

    #[test]
    fn clone_non_cloneable_returns_none() {
        let decoded = HashMap::new();
        assert!(build_clone_command(&CardType::COTAG, "COTAG", &decoded).is_none());
        assert!(build_clone_command(&CardType::EM4x50, "EM4x50", &decoded).is_none());
        assert!(build_clone_command(&CardType::Hitag, "Hitag", &decoded).is_none());
    }

    // =======================================================================
    // Edge cases
    // =======================================================================

    #[test]
    fn parse_no_tag_found() {
        let output = "[=] No known 125/134 kHz tags found!";
        assert!(parse_lf_search(output).is_none());
    }

    #[test]
    fn parse_empty_input() {
        assert!(parse_lf_search("").is_none());
    }

    #[test]
    fn parse_ansi_stripped() {
        let output = "\x1b[33m[+] EM 410x ID 0F00112233\x1b[0m";
        let (card_type, data) = parse_lf_search(output).expect("should strip ANSI");
        assert_eq!(card_type, CardType::EM4100);
        assert_eq!(data.uid, "0F00112233");
    }

    #[test]
    fn clone_rejects_invalid_uid() {
        let decoded = HashMap::new();
        // Semicolons, spaces — injection attempts
        assert!(build_clone_command(&CardType::EM4100, "0F00; rm -rf /", &decoded).is_none());
        assert!(build_clone_command(&CardType::EM4100, "", &decoded).is_none());
    }

    // =======================================================================
    // T5577 detection
    // =======================================================================

    #[test]
    fn parse_t5577_detect_basic() {
        let output = "\
            [=] Chip type......... T55x7\n\
            [=] Modulation........ ASK/Manchester\n\
            [=] Bit Rate.......... RF/64\n\
            [=] Block0............ 00148040\n\
            [=] Password set...... No";
        let status = parse_t5577_detect(output);
        assert!(status.detected);
        assert_eq!(status.chip_type, "T55x7");
        assert!(!status.password_set);
        assert_eq!(status.block0.unwrap(), "00148040");
        assert_eq!(status.modulation.unwrap(), "ASK/Manchester");
    }

    #[test]
    fn parse_t5577_detect_password() {
        let output = "\
            [=] Chip type......... T5577\n\
            [=] Password set...... Yes";
        let status = parse_t5577_detect(output);
        assert!(status.detected);
        assert!(status.password_set);
    }

    #[test]
    fn parse_t5577_chk_found() {
        let output = "[+] Found valid password: 51243648";
        assert_eq!(parse_t5577_chk(output).unwrap(), "51243648");
    }

    #[test]
    fn parse_t5577_chk_not_found() {
        let output = "[=] Checking passwords...\n[-] No valid password found.";
        assert!(parse_t5577_chk(output).is_none());
    }

    // =======================================================================
    // EM4305 detection
    // =======================================================================

    #[test]
    fn parse_em4305_info_detected() {
        assert!(parse_em4305_info("[+] EM4x05/EM4x69 chip found"));
        assert!(parse_em4305_info("Chip type: EM4305"));
    }

    #[test]
    fn parse_em4305_info_not_detected() {
        assert!(!parse_em4305_info("[!!] No compatible chip detected"));
    }

    #[test]
    fn parse_em4305_word0_value() {
        let output = "[+] Word 00 : 00000000";
        assert_eq!(parse_em4305_word0(output).unwrap(), "00000000");
    }

    // =======================================================================
    // Verification
    // =======================================================================

    #[test]
    fn verify_match_em4100() {
        let clone_output = pm3_lf_search_output("[+] EM 410x ID 0F00112233");
        let (matched, mismatched) = verify_match("0F00112233", &clone_output);
        assert!(matched);
        assert!(mismatched.is_empty());
    }

    #[test]
    fn verify_match_mismatch() {
        let clone_output = pm3_lf_search_output("[+] EM 410x ID AAAAAAAAAA");
        let (matched, _) = verify_match("0F00112233", &clone_output);
        assert!(!matched);
    }

    #[test]
    fn verify_detailed_hid() {
        let clone_output = pm3_lf_search_output(
            "[+] [H10301] HID Prox H10301 26-bit;  FC: 65  CN: 29334\n\
             [+] raw: 200078BE5E1E"
        );
        let mut source_decoded = HashMap::new();
        source_decoded.insert("facility_code".to_string(), "65".to_string());
        source_decoded.insert("card_number".to_string(), "29334".to_string());
        source_decoded.insert("raw".to_string(), "200078BE5E1E".to_string());
        let (matched, mismatched) =
            verify_match_detailed(&CardType::HIDProx, &source_decoded, &clone_output);
        assert!(matched);
        assert!(mismatched.is_empty());
    }

    // =======================================================================
    // HF: parse_hf_search() tests
    // =======================================================================

    #[test]
    fn hf_parse_classic_1k_4byte_uid() {
        let output = "\
            [+] UID: 01 02 03 04\n\
            [+] ATQA: 00 04\n\
            [+] SAK: 08 [2]\n\
            [+] MIFARE Classic 1K card\n\
            [+] Prng detection: WEAK";
        let (card_type, data) = parse_hf_search(output).expect("should parse Classic 1K");
        assert_eq!(card_type, CardType::MifareClassic1K);
        assert_eq!(data.decoded.get("uid").unwrap(), "01020304");
        assert_eq!(data.decoded.get("uid_size").unwrap(), "4B");
        assert_eq!(data.decoded.get("atqa").unwrap(), "00 04");
        assert_eq!(data.decoded.get("sak").unwrap(), "08");
        assert_eq!(data.decoded.get("prng").unwrap(), "WEAK");
    }

    #[test]
    fn hf_parse_classic_4k_sak18() {
        let output = "\
            [+] UID: AA BB CC DD\n\
            [+] ATQA: 00 02\n\
            [+] SAK: 18 [2]\n\
            [+] MIFARE Classic 4K card";
        let (card_type, data) = parse_hf_search(output).expect("should parse Classic 4K");
        assert_eq!(card_type, CardType::MifareClassic4K);
        assert_eq!(data.decoded.get("uid").unwrap(), "AABBCCDD");
        assert_eq!(data.decoded.get("sak").unwrap(), "18");
    }

    #[test]
    fn hf_parse_classic_7byte_uid() {
        let output = "\
            [+] UID: 04 11 22 33 44 55 66\n\
            [+] ATQA: 00 44\n\
            [+] SAK: 08 [2]\n\
            [+] MIFARE Classic 1K";
        let (card_type, data) = parse_hf_search(output).expect("should parse 7B UID Classic");
        assert_eq!(card_type, CardType::MifareClassic1K);
        assert_eq!(data.decoded.get("uid").unwrap(), "04112233445566");
        assert_eq!(data.decoded.get("uid_size").unwrap(), "7B");
    }

    #[test]
    fn hf_parse_classic_sak88() {
        // SAK 0x88 = Classic 1K with UID not complete (cascaded)
        let output = "\
            [+] UID: DE AD BE EF\n\
            [+] ATQA: 00 04\n\
            [+] SAK: 88";
        let (card_type, _) = parse_hf_search(output).expect("should parse SAK 0x88");
        assert_eq!(card_type, CardType::MifareClassic1K);
    }

    #[test]
    fn hf_parse_classic_with_magic() {
        let output = "\
            [+] UID: 01 02 03 04\n\
            [+] ATQA: 00 04\n\
            [+] SAK: 08\n\
            [+] Magic capabilities: Gen 1a";
        let (card_type, data) = parse_hf_search(output).expect("should parse magic Classic");
        assert_eq!(card_type, CardType::MifareClassic1K);
        assert_eq!(data.decoded.get("magic").unwrap(), "Gen 1a");
    }

    #[test]
    fn hf_parse_classic_text_fallback_4k() {
        // No SAK in output, but text says "MIFARE Classic 4K"
        let output = "\
            [+] UID: 11 22 33 44\n\
            [+] MIFARE Classic 4K detected";
        let (card_type, _) = parse_hf_search(output).expect("should parse text fallback 4K");
        assert_eq!(card_type, CardType::MifareClassic4K);
    }

    #[test]
    fn hf_parse_ntag215() {
        let output = "\
            [+] UID: 04 AA BB CC DD EE FF\n\
            [+] ATQA: 00 44\n\
            [+] SAK: 00 [2]\n\
            [+] NTAG 215";
        let (card_type, data) = parse_hf_search(output).expect("should parse NTAG215");
        assert_eq!(card_type, CardType::NTAG);
        assert_eq!(data.decoded.get("ntag_type").unwrap(), "NTAG215");
        assert_eq!(data.decoded.get("uid_size").unwrap(), "7B");
    }

    #[test]
    fn hf_parse_ntag213() {
        let output = "\
            [+] UID: 04 01 02 03 04 05 06\n\
            [+] ATQA: 00 44\n\
            [+] SAK: 00\n\
            [+] NTAG213";
        let (card_type, data) = parse_hf_search(output).expect("should parse NTAG213");
        assert_eq!(card_type, CardType::NTAG);
        assert_eq!(data.decoded.get("ntag_type").unwrap(), "NTAG213");
    }

    #[test]
    fn hf_parse_ultralight_ev1() {
        let output = "\
            [+] UID: 04 11 22 33 44 55 66\n\
            [+] ATQA: 00 44\n\
            [+] SAK: 00\n\
            [+] MIFARE Ultralight EV1";
        let (card_type, data) = parse_hf_search(output).expect("should parse UL EV1");
        assert_eq!(card_type, CardType::MifareUltralight);
        assert_eq!(data.decoded.get("ul_type").unwrap(), "Ultralight EV1");
    }

    #[test]
    fn hf_parse_ultralight_plain() {
        let output = "\
            [+] UID: 04 AA BB CC DD EE FF\n\
            [+] ATQA: 00 44\n\
            [+] SAK: 00\n\
            [+] MIFARE Ultralight";
        let (card_type, data) = parse_hf_search(output).expect("should parse UL plain");
        assert_eq!(card_type, CardType::MifareUltralight);
        assert_eq!(data.decoded.get("ul_type").unwrap(), "Ultralight");
    }

    #[test]
    fn hf_parse_ultralight_sak00_atqa0044_no_text() {
        // SAK 0x00 + ATQA 00 44 without explicit "Ultralight" text
        let output = "\
            [+] UID: 04 11 22 33 44 55 66\n\
            [+] ATQA: 00 44\n\
            [+] SAK: 00";
        let (card_type, _) = parse_hf_search(output).expect("should parse SAK00/ATQA0044");
        assert_eq!(card_type, CardType::MifareUltralight);
    }

    #[test]
    fn hf_parse_desfire_ev1() {
        let output = "\
            [+] UID: 04 AA BB CC DD EE FF\n\
            [+] ATQA: 03 44\n\
            [+] SAK: 20 [2]\n\
            [+] ATS: 06 75 77 81 02 80\n\
            [+] MIFARE DESFire EV1";
        let (card_type, data) = parse_hf_search(output).expect("should parse DESFire EV1");
        assert_eq!(card_type, CardType::DESFire);
        assert_eq!(data.decoded.get("sak").unwrap(), "20");
        assert!(data.decoded.get("ats").is_some());
        assert!(!card_type.is_cloneable());
    }

    #[test]
    fn hf_parse_desfire_plain() {
        let output = "\
            [+] UID: 01 02 03 04 05 06 07\n\
            [+] ATQA: 03 44\n\
            [+] SAK: 20\n\
            [+] DESFire";
        let (card_type, _) = parse_hf_search(output).expect("should parse DESFire plain");
        assert_eq!(card_type, CardType::DESFire);
    }

    #[test]
    fn hf_parse_iclass_with_csn() {
        let output = "\
            [+] iCLASS / Picopass card found\n\
            [+] CSN: 00 0B 0F FF F7 FF 12 E0";
        let (card_type, data) = parse_hf_search(output).expect("should parse iCLASS");
        assert_eq!(card_type, CardType::IClass);
        assert_eq!(data.decoded.get("uid").unwrap(), "000B0FFFF7FF12E0");
    }

    #[test]
    fn hf_parse_iclass_no_csn() {
        let output = "[+] iCLASS card detected";
        let (card_type, data) = parse_hf_search(output).expect("should parse iCLASS no CSN");
        assert_eq!(card_type, CardType::IClass);
        assert_eq!(data.uid, "iCLASS");
    }

    #[test]
    fn hf_parse_no_card() {
        let output = "[!] No known/supported 13.56 MHz tags found";
        assert!(parse_hf_search(output).is_none());
    }

    #[test]
    fn hf_parse_no_data() {
        let output = "[!] No data found";
        assert!(parse_hf_search(output).is_none());
    }

    #[test]
    fn hf_parse_empty() {
        assert!(parse_hf_search("").is_none());
    }

    #[test]
    fn hf_parse_ansi_stripped() {
        let output = "\x1b[32m[+] UID: 01 02 03 04\x1b[0m\n\
            \x1b[32m[+] SAK: 08\x1b[0m\n\
            \x1b[32m[+] ATQA: 00 04\x1b[0m";
        let (card_type, data) = parse_hf_search(output).expect("should strip ANSI");
        assert_eq!(card_type, CardType::MifareClassic1K);
        assert_eq!(data.decoded.get("uid").unwrap(), "01020304");
    }

    #[test]
    fn hf_parse_prng_hard() {
        let output = "\
            [+] UID: 01 02 03 04\n\
            [+] ATQA: 00 04\n\
            [+] SAK: 08\n\
            [+] Prng detection: HARD";
        let (_, data) = parse_hf_search(output).expect("should parse PRNG HARD");
        assert_eq!(data.decoded.get("prng").unwrap(), "HARD");
    }

    #[test]
    fn hf_parse_prng_static() {
        let output = "\
            [+] UID: 01 02 03 04\n\
            [+] ATQA: 00 04\n\
            [+] SAK: 08\n\
            [+] Prng detection: STATIC";
        let (_, data) = parse_hf_search(output).expect("should parse PRNG STATIC");
        assert_eq!(data.decoded.get("prng").unwrap(), "STATIC");
    }

    // -----------------------------------------------------------------------
    // Autopwn parser tests
    // -----------------------------------------------------------------------

    #[test]
    fn autopwn_dictionary_progress() {
        let line = "[=] found 12/32 keys (D)";
        let event = parse_autopwn_line(line).expect("should parse dict progress");
        assert_eq!(
            event,
            AutopwnEvent::DictionaryProgress {
                found: 12,
                total: 32
            }
        );
    }

    #[test]
    fn autopwn_dictionary_all_keys() {
        let line = "[+] found 32/32 keys (D)";
        let event = parse_autopwn_line(line).expect("should parse all keys found");
        assert_eq!(
            event,
            AutopwnEvent::DictionaryProgress {
                found: 32,
                total: 32
            }
        );
    }

    #[test]
    fn autopwn_key_found() {
        let line = "[+] found valid key [ FFFFFFFFFFFF ]";
        let event = parse_autopwn_line(line).expect("should parse key found");
        assert_eq!(
            event,
            AutopwnEvent::KeyFound {
                key: "FFFFFFFFFFFF".to_string()
            }
        );
    }

    #[test]
    fn autopwn_key_found_lowercase() {
        let line = "[+] found valid key [ a0a1a2a3a4a5 ]";
        let event = parse_autopwn_line(line).expect("should parse lowercase key");
        assert_eq!(
            event,
            AutopwnEvent::KeyFound {
                key: "A0A1A2A3A4A5".to_string()
            }
        );
    }

    #[test]
    fn autopwn_darkside_started() {
        let line = "[!] Darkside attack starting...";
        let event = parse_autopwn_line(line).expect("should parse darkside start");
        assert_eq!(event, AutopwnEvent::DarksideStarted);
    }

    #[test]
    fn autopwn_nested_started() {
        let line = "[=] Nested attack starting...";
        let event = parse_autopwn_line(line).expect("should parse nested start");
        assert_eq!(event, AutopwnEvent::NestedStarted);
    }

    #[test]
    fn autopwn_hardnested_started() {
        let line = "[=] Hardnested attack starting...";
        let event = parse_autopwn_line(line).expect("should parse hardnested start");
        assert_eq!(event, AutopwnEvent::HardnestedStarted);
    }

    #[test]
    fn autopwn_staticnested_started() {
        let line = "[=] Staticnested attack starting...";
        let event = parse_autopwn_line(line).expect("should parse staticnested start");
        assert_eq!(event, AutopwnEvent::StaticnestedStarted);
    }

    #[test]
    fn autopwn_dump_complete() {
        let line = "[+] Succeeded in dumping all blocks";
        let event = parse_autopwn_line(line).expect("should parse dump complete");
        assert_eq!(
            event,
            AutopwnEvent::DumpComplete {
                file_path: String::new()
            }
        );
    }

    #[test]
    fn autopwn_dump_partial() {
        let line = "[!] Dump file is PARTIAL complete";
        let event = parse_autopwn_line(line).expect("should parse dump partial");
        assert_eq!(
            event,
            AutopwnEvent::DumpPartial {
                file_path: String::new()
            }
        );
    }

    #[test]
    fn autopwn_dump_saved_bin() {
        let line = "[+] saved 64 blocks to file hf-mf-01020304-dump.bin";
        let event = parse_autopwn_line(line).expect("should parse dump saved");
        assert_eq!(
            event,
            AutopwnEvent::DumpComplete {
                file_path: "hf-mf-01020304-dump.bin".to_string()
            }
        );
    }

    #[test]
    fn autopwn_dump_saved_json() {
        let line = "[+] saved to binary file `hf-mf-AABBCCDD-dump.json`";
        let event = parse_autopwn_line(line).expect("should parse json dump saved");
        assert_eq!(
            event,
            AutopwnEvent::DumpComplete {
                file_path: "hf-mf-AABBCCDD-dump.json".to_string()
            }
        );
    }

    #[test]
    fn autopwn_failed() {
        let line = "[!] all key recovery attempts failed";
        let event = parse_autopwn_line(line).expect("should parse failure");
        assert_eq!(
            event,
            AutopwnEvent::Failed {
                reason: "All key recovery attempts failed".to_string()
            }
        );
    }

    #[test]
    fn autopwn_finished() {
        let line = "[=] autopwn execution time: 45 seconds";
        let event = parse_autopwn_line(line).expect("should parse finish time");
        assert_eq!(event, AutopwnEvent::Finished { time_secs: 45 });
    }

    #[test]
    fn autopwn_empty_line() {
        assert!(parse_autopwn_line("").is_none());
        assert!(parse_autopwn_line("   ").is_none());
    }

    #[test]
    fn autopwn_irrelevant_line() {
        assert!(parse_autopwn_line("[=] Using key FFFFFFFFFFFF for sector 0").is_none());
        assert!(parse_autopwn_line("[+] UID: 01 02 03 04").is_none());
    }

    // HF-4: parse_magic_detection() tests
    // -----------------------------------------------------------------------

    #[test]
    fn magic_detect_gen1a() {
        let output = "[+] Magic capabilities... Gen 1a";
        assert_eq!(parse_magic_detection(output), Some(MagicGeneration::Gen1a));
    }

    #[test]
    fn magic_detect_gen1b() {
        let output = "[+] Magic capabilities : Gen 1b";
        assert_eq!(parse_magic_detection(output), Some(MagicGeneration::Gen1a));
    }

    #[test]
    fn magic_detect_gen2_cuid() {
        let output = "[+] Magic capabilities : CUID";
        assert_eq!(parse_magic_detection(output), Some(MagicGeneration::Gen2));
    }

    #[test]
    fn magic_detect_gen2_text() {
        let output = "[+] Generation: Gen 2";
        assert_eq!(parse_magic_detection(output), Some(MagicGeneration::Gen2));
    }

    #[test]
    fn magic_detect_gen3_apdu() {
        let output = "[+] Magic capabilities : APDU";
        assert_eq!(parse_magic_detection(output), Some(MagicGeneration::Gen3));
    }

    #[test]
    fn magic_detect_gen3_text() {
        let output = "[+] Magic capabilities : Gen 3";
        assert_eq!(parse_magic_detection(output), Some(MagicGeneration::Gen3));
    }

    #[test]
    fn magic_detect_gen4_gtu() {
        let output = "[+] Magic capabilities... Gen 4 GTU";
        assert_eq!(parse_magic_detection(output), Some(MagicGeneration::Gen4GTU));
    }

    #[test]
    fn magic_detect_gen4_ultimate() {
        let output = "[+] Magic: ultimate";
        assert_eq!(parse_magic_detection(output), Some(MagicGeneration::Gen4GTU));
    }

    #[test]
    fn magic_detect_gen4_gdm() {
        let output = "[+] Magic capabilities : GDM";
        assert_eq!(parse_magic_detection(output), Some(MagicGeneration::Gen4GDM));
    }

    #[test]
    fn magic_detect_none() {
        let output = "[+] UID: 01 02 03 04\n[+] ATQA: 00 04\n[+] SAK: 08";
        assert_eq!(parse_magic_detection(output), None);
    }

    #[test]
    fn magic_detect_empty() {
        assert_eq!(parse_magic_detection(""), None);
    }

    #[test]
    fn magic_detect_ufuid() {
        let output = "[+] Magic capabilities : UFUID";
        assert_eq!(parse_magic_detection(output), Some(MagicGeneration::Gen3));
    }

    // is_hf_card_present() tests

    #[test]
    fn hf_card_present_yes() {
        let output = "[+] UID: 01 02 03 04\n[+] ATQA: 00 04\n[+] SAK: 08";
        assert!(is_hf_card_present(output));
    }

    #[test]
    fn hf_card_present_no() {
        let output = "[!] No known 13.56 MHz tag found";
        assert!(!is_hf_card_present(output));
    }

    // is_iclass_present() tests

    #[test]
    fn iclass_present_yes() {
        let output = "[+] iCLASS / Picopass detected\n[+] CSN: 00 01 02 03 04 05 06 07";
        assert!(is_iclass_present(output));
    }

    #[test]
    fn iclass_present_no() {
        let output = "[!] No known tag found";
        assert!(!is_iclass_present(output));
    }

    // -----------------------------------------------------------------------
    // extract_dump_file_path tests
    // -----------------------------------------------------------------------

    #[test]
    fn extract_dump_path_bin() {
        let output = "[+] saved 64 blocks to file hf-mf-01020304-dump.bin\n[+] autopwn finished";
        let path = extract_dump_file_path(output).expect("should extract bin path");
        assert_eq!(path, "hf-mf-01020304-dump.bin");
    }

    #[test]
    fn extract_dump_path_json() {
        let output = "[+] saved to binary file `hf-mf-AABBCCDD-dump.json`\n[+] done";
        let path = extract_dump_file_path(output).expect("should extract json path");
        assert_eq!(path, "hf-mf-AABBCCDD-dump.json");
    }

    #[test]
    fn extract_dump_path_eml() {
        let output = "[+] saved 64 blocks to file hf-mf-01020304-dump.eml";
        let path = extract_dump_file_path(output).expect("should extract eml path");
        assert_eq!(path, "hf-mf-01020304-dump.eml");
    }

    #[test]
    fn extract_dump_path_none() {
        let output = "[+] autopwn finished\n[+] no dump saved";
        assert!(extract_dump_file_path(output).is_none());
    }

    #[test]
    fn extract_dump_path_multiline_finds_first() {
        let output = "[+] some preamble\n[+] saved 64 blocks to file first-dump.bin\n[+] saved 64 blocks to file second-dump.bin";
        let path = extract_dump_file_path(output).expect("should extract first path");
        assert_eq!(path, "first-dump.bin");
    }

    // -----------------------------------------------------------------------
    // parse_hf_search edge case tests
    // -----------------------------------------------------------------------

    #[test]
    fn hf_parse_classic_1k_sak98() {
        // SAK 0x98 = Classic 4K with cascaded UID (7-byte)
        let output = "\
[+] UID: 04 11 22 33 44 55 66
[+] ATQA: 00 02
[+] SAK: 98 [2]";
        let (ct, cd) = parse_hf_search(output).expect("should parse SAK 98");
        assert_eq!(ct, CardType::MifareClassic4K);
        assert_eq!(cd.decoded.get("sak").unwrap(), "98");
    }

    #[test]
    fn hf_parse_classic_with_prng_weak() {
        let output = "\
[+] UID: 01 02 03 04
[+] ATQA: 00 04
[+] SAK: 08 [2]
[=] Prng detection: WEAK";
        let (ct, cd) = parse_hf_search(output).expect("should parse");
        assert_eq!(ct, CardType::MifareClassic1K);
        assert_eq!(cd.decoded.get("prng").unwrap(), "WEAK");
    }

    #[test]
    fn hf_parse_ntag216() {
        let output = "\
[+] UID: 04 AA BB CC DD EE FF
[+] ATQA: 00 44
[+] SAK: 00 [2]
[=] NTAG 216";
        let (ct, cd) = parse_hf_search(output).expect("should parse NTAG 216");
        assert_eq!(ct, CardType::NTAG);
        // Parser stores full match including "NTAG" prefix
        assert!(cd.decoded.get("ntag_type").unwrap().contains("216"));
    }

    #[test]
    fn hf_parse_desfire_with_ats() {
        let output = "\
[+] UID: 04 11 22 33 44 55 66
[+] ATQA: 03 44
[+] SAK: 20 [2]
[+] ATS: 75 77 80 02 80
[=] MIFARE DESFire EV1";
        let (ct, cd) = parse_hf_search(output).expect("should parse DESFire with ATS");
        assert_eq!(ct, CardType::DESFire);
        assert_eq!(cd.decoded.get("ats").unwrap(), "75 77 80 02 80");
    }

    #[test]
    fn hf_parse_ultralight_c() {
        let output = "\
[+] UID: 04 11 22 33 44 55 66
[+] ATQA: 00 44
[+] SAK: 00 [2]
[=] Ultralight C";
        let (ct, cd) = parse_hf_search(output).expect("should parse UL C");
        assert_eq!(ct, CardType::MifareUltralight);
        assert_eq!(cd.decoded.get("ul_type").unwrap(), "Ultralight C");
    }

    #[test]
    fn hf_parse_iclass_picopass() {
        // Parser strips spaces from CSN/UID
        let output = "[+] Picopass / iCLASS LEGACY detected\n[+] CSN: AA BB CC DD EE FF 00 11";
        let (ct, cd) = parse_hf_search(output).expect("should parse Picopass");
        assert_eq!(ct, CardType::IClass);
        assert_eq!(cd.uid, "AABBCCDDEEFF0011");
    }

    // -----------------------------------------------------------------------
    // parse_autopwn_line edge case tests
    // -----------------------------------------------------------------------

    #[test]
    fn autopwn_key_found_method_n() {
        // Nested attack key
        let line = "[=] found valid key [ A0A1A2A3A4A5 ] (N)";
        let event = parse_autopwn_line(line).expect("should parse nested key");
        assert_eq!(
            event,
            AutopwnEvent::KeyFound {
                key: "A0A1A2A3A4A5".to_string(),
            }
        );
    }

    #[test]
    fn autopwn_dictionary_partial() {
        let line = "[=] found 5/32 keys (D)";
        let event = parse_autopwn_line(line).expect("should parse partial keys");
        assert_eq!(
            event,
            AutopwnEvent::DictionaryProgress {
                found: 5,
                total: 32,
            }
        );
    }

    #[test]
    fn autopwn_4k_keys() {
        // 4K cards have 80 sectors = 160 keys (A + B per sector)
        let line = "[=] found 80/80 keys (D)";
        let event = parse_autopwn_line(line).expect("should parse 4K keys");
        assert_eq!(
            event,
            AutopwnEvent::DictionaryProgress {
                found: 80,
                total: 80,
            }
        );
    }

    #[test]
    fn autopwn_finished_long_time() {
        let line = "[+] autopwn execution time: 3742 seconds";
        let event = parse_autopwn_line(line).expect("should parse long time");
        assert_eq!(event, AutopwnEvent::Finished { time_secs: 3742 });
    }

    #[test]
    fn autopwn_dump_saved_with_path() {
        let line = "[+] saved 256 blocks to file hf-mf-DEADBEEF-dump.bin";
        let event = parse_autopwn_line(line).expect("should parse 4K dump");
        assert_eq!(
            event,
            AutopwnEvent::DumpComplete {
                file_path: "hf-mf-DEADBEEF-dump.bin".to_string(),
            }
        );
    }

    // -----------------------------------------------------------------------
    // parse_magic_detection edge case tests
    // -----------------------------------------------------------------------

    #[test]
    fn magic_detect_gen1a_dots_separator() {
        // PM3 uses dots in output: "Magic capabilities... Gen 1a"
        let output = "[=] Magic capabilities... Gen 1a";
        let gen = parse_magic_detection(output).expect("should parse with dots");
        assert_eq!(gen, MagicGeneration::Gen1a);
    }

    #[test]
    fn magic_detect_case_insensitive() {
        let output = "[=] Magic capabilities: gen 2 / CUID";
        let gen = parse_magic_detection(output).expect("should parse case-insensitive");
        assert_eq!(gen, MagicGeneration::Gen2);
    }

    #[test]
    fn magic_detect_gen4_gtu_keyword_ultimate() {
        let output = "[=] Magic capabilities: Gen 4 GTU / ultimate magic card";
        let gen = parse_magic_detection(output).expect("should parse GTU");
        assert_eq!(gen, MagicGeneration::Gen4GTU);
    }

    #[test]
    fn magic_detect_uscuid() {
        // USCUID is a Gen4 GDM variant — real PM3 v4.20728 output
        let output = "[+] Magic capabilities... Gen 4 GDM / USCUID ( ZUID Gen1 Magic Wakeup )";
        let gen = parse_magic_detection(output).expect("should parse USCUID as Gen4GDM");
        assert_eq!(gen, MagicGeneration::Gen4GDM);
    }

    #[test]
    fn magic_detect_uscuid_standalone() {
        // USCUID without GDM prefix
        let output = "[+] Magic capabilities... USCUID";
        let gen = parse_magic_detection(output).expect("should parse standalone USCUID");
        assert_eq!(gen, MagicGeneration::Gen4GDM);
    }

    // -----------------------------------------------------------------------
    // Real PM3 output regression tests
    // -----------------------------------------------------------------------

    #[test]
    fn hf_parse_real_pm3_classic_1k_magic() {
        // Real output from PM3 v4.20728 with Gen1a+Gen4GDM/USCUID dual magic card
        let output = "\
[-] Searching for ISO14443-A tag...\n\
[=] ---------- ISO14443-A Information ----------\n\
[+]  UID: 7D E9 25 4E   ( ONUID, re-used )\n\
[+] ATQA: 00 04\n\
[+]  SAK: 08 [2]\n\
[+] Possible types:\n\
[+]    MIFARE Classic 1K\n\
[=] \n\
[=] Proprietary non iso14443-4 card found\n\
[=] RATS not supported\n\
\n\
[+] Magic capabilities... Gen 1a\n\
[+] Magic capabilities... Gen 4 GDM / USCUID ( ZUID Gen1 Magic Wakeup )\n\
[+] Prng detection..... weak\n\
\n\
[?] Hint: Use `hf mf c*` magic commands\n\
[?] Hint: Use `hf mf gdm* --gen1a` magic commands\n\
[?] Hint: Try `hf mf info`\n\
\n\
\n\
[+] Valid ISO 14443-A tag found\n\
\n\
[-] Searching for iCLASS / PicoPass tag...\n\
[-] Searching for FeliCa tag...";

        let (card_type, data) = parse_hf_search(output).expect("should parse real PM3 Classic 1K");
        assert_eq!(card_type, CardType::MifareClassic1K);
        assert_eq!(data.uid, "7DE9254E");
        assert_eq!(data.decoded.get("sak").unwrap(), "08");
        assert_eq!(data.decoded.get("atqa").unwrap(), "00 04");
        assert_eq!(data.decoded.get("prng").unwrap(), "WEAK");
        assert_eq!(data.decoded.get("magic").unwrap(), "Gen 1a");
        assert_eq!(data.decoded.get("uid_size").unwrap(), "4B");
    }

    #[test]
    fn hf_parse_iclass_not_false_positive() {
        // "Searching for iCLASS" should NOT trigger iCLASS detection
        let output = "\
[+]  UID: AA BB CC DD\n\
[+] ATQA: 00 04\n\
[+]  SAK: 08 [2]\n\
[-] Searching for iCLASS / PicoPass tag...";

        let (card_type, _) = parse_hf_search(output).expect("should parse as Classic, not iCLASS");
        assert_eq!(card_type, CardType::MifareClassic1K);
    }

    #[test]
    fn hf_parse_prng_dots_format() {
        // PM3 uses dots between "detection" and value
        let output = "\
[+]  UID: 11 22 33 44\n\
[+] ATQA: 00 04\n\
[+]  SAK: 08 [2]\n\
[+] Prng detection..... weak";

        let (_, data) = parse_hf_search(output).expect("should parse PRNG with dots");
        assert_eq!(data.decoded.get("prng").unwrap(), "WEAK");
    }

    #[test]
    fn hf_parse_prng_dots_hard() {
        let output = "\
[+]  UID: 11 22 33 44\n\
[+] ATQA: 00 04\n\
[+]  SAK: 08 [2]\n\
[+] Prng detection..... HARD";

        let (_, data) = parse_hf_search(output).expect("should parse PRNG HARD with dots");
        assert_eq!(data.decoded.get("prng").unwrap(), "HARD");
    }
}
