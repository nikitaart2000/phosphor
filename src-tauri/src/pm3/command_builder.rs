/// PM3 CLI command strings.
/// All commands assume the Iceman fork with `-f` flag for subprocess piping.

use crate::cards::types::{BlankType, CardType};

// ---------------------------------------------------------------------------
// Device / search commands
// ---------------------------------------------------------------------------

pub fn build_detect_command() -> &'static str {
    "hw version"
}

pub fn build_lf_search() -> &'static str {
    "lf search"
}

pub fn build_hf_search() -> &'static str {
    "hf search"
}

// ---------------------------------------------------------------------------
// T5577 blank management
// ---------------------------------------------------------------------------

pub fn build_t5577_detect() -> &'static str {
    "lf t55xx detect"
}

pub fn build_t5577_chk() -> &'static str {
    "lf t55xx chk"
}

pub fn build_t5577_wipe() -> &'static str {
    "lf t55xx wipe"
}

/// Wipe a T5577 that has a known password.
pub fn build_t5577_wipe_with_password(password: &str) -> String {
    format!("lf t55xx wipe -p {}", password)
}

/// Detect a T5577 that might have a known password.
pub fn build_t5577_detect_with_password(password: &str) -> String {
    format!("lf t55xx detect -p {}", password)
}

// ---------------------------------------------------------------------------
// EM4305 blank management
// ---------------------------------------------------------------------------

pub fn build_em4305_wipe() -> &'static str {
    "lf em 4x05 wipe"
}

/// Append `--em` flag to a base clone command for EM4305 blanks.
pub fn build_clone_for_em4305(base_cmd: &str) -> String {
    format!("{} --em", base_cmd)
}

/// Append `-p {password}` to a base clone command for password-protected T5577.
pub fn build_clone_with_password(base_cmd: &str, password: &str) -> String {
    format!("{} -p {}", base_cmd, password)
}

// ---------------------------------------------------------------------------
// LF clone commands — original 11 types (improved)
// ---------------------------------------------------------------------------

pub fn build_em4100_clone(id: &str) -> String {
    format!("lf em 410x clone --id {}", id)
}

/// HID clone using detected Wiegand format (defaults to H10301 / 26-bit).
pub fn build_hid_clone(fc: u32, cn: u32, format: Option<&str>) -> String {
    let wiegand = format.unwrap_or("H10301");
    format!("lf hid clone -w {} --fc {} --cn {}", wiegand, fc, cn)
}

pub fn build_hid_clone_raw(raw: &str) -> String {
    format!("lf hid clone -r {}", raw)
}

pub fn build_indala_clone(raw: &str) -> String {
    format!("lf indala clone --raw {}", raw)
}

/// IO Prox clone with version number support.
pub fn build_ioprox_clone(fc: u32, cn: u32, vn: u32) -> String {
    format!("lf io clone --vn {} --fc {} --cn {}", vn, fc, cn)
}

pub fn build_ioprox_clone_raw(raw: &str) -> String {
    format!("lf io clone --raw {}", raw)
}

/// AWID clone with format support (26/34/37/50 bit).
pub fn build_awid_clone(fc: u32, cn: u32, fmt: Option<u32>) -> String {
    match fmt {
        Some(f) => format!("lf awid clone --fmt {} --fc {} --cn {}", f, fc, cn),
        None => format!("lf awid clone --fc {} --cn {}", fc, cn),
    }
}

/// FDX-B clone with country code + national ID.
pub fn build_fdxb_clone(country: u32, national_id: u64) -> String {
    format!(
        "lf fdxb clone --country {} --national {}",
        country, national_id
    )
}

pub fn build_fdxb_clone_raw(raw: &str) -> String {
    format!("lf fdxb clone --raw {}", raw)
}

/// Paradox clone with FC/CN (preferred over raw).
pub fn build_paradox_clone(fc: u32, cn: u32) -> String {
    format!("lf paradox clone --fc {} --cn {}", fc, cn)
}

pub fn build_paradox_clone_raw(raw: &str) -> String {
    format!("lf paradox clone --raw {}", raw)
}

pub fn build_viking_clone(raw: &str) -> String {
    format!("lf viking clone --raw {}", raw)
}

pub fn build_pyramid_clone(fc: u32, cn: u32) -> String {
    format!("lf pyramid clone --fc {} --cn {}", fc, cn)
}

/// Keri clone with type support: "i" for internal, "m" for MS format.
pub fn build_keri_clone(raw: &str, keri_type: Option<&str>) -> String {
    match keri_type {
        Some(t) => format!("lf keri clone -t {} --raw {}", t, raw),
        None => format!("lf keri clone --raw {}", raw),
    }
}

pub fn build_nexwatch_clone(raw: &str) -> String {
    format!("lf nexwatch clone --raw {}", raw)
}

// ---------------------------------------------------------------------------
// LF clone commands — 11 new types
// ---------------------------------------------------------------------------

/// Presco clone with hex data.
pub fn build_presco_clone_hex(hex: &str) -> String {
    format!("lf presco clone -d {}", hex)
}

/// Presco clone with site code + user code.
pub fn build_presco_clone(site_code: u32, user_code: u32) -> String {
    format!(
        "lf presco clone --sitecode {} --usercode {}",
        site_code, user_code
    )
}

/// Nedap clone with subtype + card number.
pub fn build_nedap_clone(subtype: u32, card_number: u32) -> String {
    format!("lf nedap clone --st {} --cn {}", subtype, card_number)
}

/// GProxII clone with XSF (installation code) + card number. XSF is CRITICAL.
pub fn build_gproxii_clone(xsf: u32, cn: u32) -> String {
    format!("lf gproxii clone --xsf {} --cn {}", xsf, cn)
}

/// Gallagher clone with region, facility, card number, issue level.
pub fn build_gallagher_clone(rc: u32, fc: u32, cn: u32, il: u32) -> String {
    format!(
        "lf gallagher clone --rc {} --fc {} --cn {} --il {}",
        rc, fc, cn, il
    )
}

/// PAC/Stanley clone with card number.
pub fn build_pac_clone(cn: &str) -> String {
    format!("lf pac clone --cn {}", cn)
}

pub fn build_pac_clone_raw(raw: &str) -> String {
    format!("lf pac clone --raw {}", raw)
}

/// Noralsy clone (raw only).
pub fn build_noralsy_clone(raw: &str) -> String {
    format!("lf noralsy clone --raw {}", raw)
}

/// Jablotron clone with hex card number.
pub fn build_jablotron_clone(cn: &str) -> String {
    format!("lf jablotron clone --cn {}", cn)
}

/// SecuraKey clone (raw only).
pub fn build_securakey_clone(raw: &str) -> String {
    format!("lf securakey clone --raw {}", raw)
}

/// Visa2000 clone with card number.
pub fn build_visa2000_clone(cn: u32) -> String {
    format!("lf visa2000 clone --cn {}", cn)
}

/// Motorola clone (raw only).
pub fn build_motorola_clone(raw: &str) -> String {
    format!("lf motorola clone --raw {}", raw)
}

/// IDTECK clone (raw only).
pub fn build_idteck_clone(raw: &str) -> String {
    format!("lf idteck clone --raw {}", raw)
}

// ---------------------------------------------------------------------------
// Type-specific reader commands for verification
// ---------------------------------------------------------------------------

/// Get the type-specific reader command for verification after cloning.
/// Uses a dedicated reader where available, falling back to `lf search`.
pub fn build_verify_command(card_type: &CardType) -> &'static str {
    match card_type {
        CardType::EM4100 => "lf em 410x reader",
        CardType::HIDProx => "lf hid reader",
        CardType::Indala => "lf indala reader",
        CardType::IOProx => "lf io reader",
        CardType::AWID => "lf awid reader",
        CardType::FDX_B => "lf fdxb reader",
        CardType::Paradox => "lf paradox reader",
        CardType::Viking => "lf viking reader",
        CardType::Pyramid => "lf pyramid reader",
        CardType::Keri => "lf keri reader",
        CardType::NexWatch => "lf nexwatch reader",
        CardType::Presco => "lf presco reader",
        CardType::Nedap => "lf nedap reader",
        CardType::GProxII => "lf gproxii reader",
        CardType::Gallagher => "lf gallagher reader",
        CardType::PAC => "lf pac reader",
        CardType::Noralsy => "lf noralsy reader",
        CardType::Jablotron => "lf jablotron reader",
        CardType::SecuraKey => "lf securakey reader",
        CardType::Visa2000 => "lf visa2000 reader",
        CardType::Motorola => "lf motorola reader",
        CardType::IDTECK => "lf idteck reader",
        // Non-cloneable LF or HF — fall back to search
        _ => "lf search",
    }
}

// ---------------------------------------------------------------------------
// Build clone command dispatcher
// ---------------------------------------------------------------------------

/// Build the appropriate clone command for a given card type + data.
/// Returns None if clone is not supported for this type.
pub fn build_clone_command(
    card_type: &CardType,
    uid: &str,
    decoded: &std::collections::HashMap<String, String>,
) -> Option<String> {
    match card_type {
        CardType::EM4100 => Some(build_em4100_clone(uid)),

        CardType::HIDProx => {
            if let (Some(fc), Some(cn)) =
                (decoded.get("facility_code"), decoded.get("card_number"))
            {
                if let (Ok(fc_n), Ok(cn_n)) = (fc.parse::<u32>(), cn.parse::<u32>()) {
                    let fmt = decoded.get("format").map(|s| s.as_str());
                    return Some(build_hid_clone(fc_n, cn_n, fmt));
                }
            }
            decoded.get("raw").map(|raw| build_hid_clone_raw(raw))
        }

        CardType::Indala => Some(build_indala_clone(uid)),

        CardType::IOProx => {
            if let (Some(fc), Some(cn)) =
                (decoded.get("facility_code"), decoded.get("card_number"))
            {
                let vn = decoded
                    .get("version")
                    .and_then(|v| v.parse::<u32>().ok())
                    .unwrap_or(1);
                if let (Ok(fc_n), Ok(cn_n)) = (fc.parse::<u32>(), cn.parse::<u32>()) {
                    return Some(build_ioprox_clone(fc_n, cn_n, vn));
                }
            }
            Some(build_ioprox_clone_raw(uid))
        }

        CardType::AWID => {
            if let (Some(fc), Some(cn)) =
                (decoded.get("facility_code"), decoded.get("card_number"))
            {
                if let (Ok(fc_n), Ok(cn_n)) = (fc.parse::<u32>(), cn.parse::<u32>()) {
                    let fmt = decoded.get("format").and_then(|f| f.parse::<u32>().ok());
                    return Some(build_awid_clone(fc_n, cn_n, fmt));
                }
            }
            None
        }

        CardType::FDX_B => {
            if let (Some(country), Some(national)) =
                (decoded.get("country"), decoded.get("national_id"))
            {
                if let (Ok(cc), Ok(nid)) = (country.parse::<u32>(), national.parse::<u64>()) {
                    return Some(build_fdxb_clone(cc, nid));
                }
            }
            // Fallback to raw
            if let Some(raw) = decoded.get("raw") {
                Some(build_fdxb_clone_raw(raw))
            } else {
                Some(build_fdxb_clone_raw(uid))
            }
        }

        CardType::Paradox => {
            if let (Some(fc), Some(cn)) =
                (decoded.get("facility_code"), decoded.get("card_number"))
            {
                if let (Ok(fc_n), Ok(cn_n)) = (fc.parse::<u32>(), cn.parse::<u32>()) {
                    return Some(build_paradox_clone(fc_n, cn_n));
                }
            }
            Some(build_paradox_clone_raw(uid))
        }

        CardType::Viking => Some(build_viking_clone(uid)),

        CardType::Pyramid => {
            if let (Some(fc), Some(cn)) =
                (decoded.get("facility_code"), decoded.get("card_number"))
            {
                if let (Ok(fc_n), Ok(cn_n)) = (fc.parse::<u32>(), cn.parse::<u32>()) {
                    return Some(build_pyramid_clone(fc_n, cn_n));
                }
            }
            None
        }

        CardType::Keri => {
            let keri_type = decoded.get("keri_type").map(|s| s.as_str());
            Some(build_keri_clone(uid, keri_type))
        }

        CardType::NexWatch => Some(build_nexwatch_clone(uid)),

        // --- New 11 types ---

        CardType::Presco => {
            if let (Some(sc), Some(uc)) =
                (decoded.get("site_code"), decoded.get("user_code"))
            {
                if let (Ok(sc_n), Ok(uc_n)) = (sc.parse::<u32>(), uc.parse::<u32>()) {
                    return Some(build_presco_clone(sc_n, uc_n));
                }
            }
            Some(build_presco_clone_hex(uid))
        }

        CardType::Nedap => {
            if let (Some(st), Some(cn)) =
                (decoded.get("subtype"), decoded.get("card_number"))
            {
                if let (Ok(st_n), Ok(cn_n)) = (st.parse::<u32>(), cn.parse::<u32>()) {
                    return Some(build_nedap_clone(st_n, cn_n));
                }
            }
            None
        }

        CardType::GProxII => {
            if let (Some(xsf), Some(cn)) =
                (decoded.get("xsf"), decoded.get("card_number"))
            {
                if let (Ok(xsf_n), Ok(cn_n)) = (xsf.parse::<u32>(), cn.parse::<u32>()) {
                    return Some(build_gproxii_clone(xsf_n, cn_n));
                }
            }
            None
        }

        CardType::Gallagher => {
            if let (Some(rc), Some(fc), Some(cn), Some(il)) = (
                decoded.get("region_code"),
                decoded.get("facility_code"),
                decoded.get("card_number"),
                decoded.get("issue_level"),
            ) {
                if let (Ok(rc_n), Ok(fc_n), Ok(cn_n), Ok(il_n)) = (
                    rc.parse::<u32>(),
                    fc.parse::<u32>(),
                    cn.parse::<u32>(),
                    il.parse::<u32>(),
                ) {
                    return Some(build_gallagher_clone(rc_n, fc_n, cn_n, il_n));
                }
            }
            None
        }

        CardType::PAC => {
            if let Some(raw) = decoded.get("raw") {
                return Some(build_pac_clone_raw(raw));
            }
            if let Some(cn) = decoded.get("card_number") {
                return Some(build_pac_clone(cn));
            }
            Some(build_pac_clone(uid))
        }

        CardType::Noralsy => {
            if let Some(raw) = decoded.get("raw") {
                Some(build_noralsy_clone(raw))
            } else {
                Some(build_noralsy_clone(uid))
            }
        }

        CardType::Jablotron => {
            if let Some(cn) = decoded.get("card_number") {
                Some(build_jablotron_clone(cn))
            } else {
                Some(build_jablotron_clone(uid))
            }
        }

        CardType::SecuraKey => {
            if let Some(raw) = decoded.get("raw") {
                Some(build_securakey_clone(raw))
            } else {
                Some(build_securakey_clone(uid))
            }
        }

        CardType::Visa2000 => {
            if let Some(cn) = decoded.get("card_number") {
                if let Ok(cn_n) = cn.parse::<u32>() {
                    return Some(build_visa2000_clone(cn_n));
                }
            }
            None
        }

        CardType::Motorola => {
            if let Some(raw) = decoded.get("raw") {
                Some(build_motorola_clone(raw))
            } else {
                Some(build_motorola_clone(uid))
            }
        }

        CardType::IDTECK => {
            if let Some(raw) = decoded.get("raw") {
                Some(build_idteck_clone(raw))
            } else {
                Some(build_idteck_clone(uid))
            }
        }

        // Non-cloneable LF types
        CardType::COTAG | CardType::EM4x50 | CardType::Hitag => None,

        // HF cloning not yet implemented in this module
        CardType::MifareClassic1K
        | CardType::MifareClassic4K
        | CardType::MifareUltralight
        | CardType::NTAG
        | CardType::DESFire
        | CardType::IClass => None,
    }
}

/// Determine the wipe command based on blank type.
pub fn build_wipe_command(blank_type: &BlankType, password: Option<&str>) -> String {
    match blank_type {
        BlankType::EM4305 => build_em4305_wipe().to_string(),
        BlankType::T5577 => match password {
            Some(pw) => build_t5577_wipe_with_password(pw),
            None => build_t5577_wipe().to_string(),
        },
        // Other blank types don't have a wipe command in this module
        _ => String::new(),
    }
}
