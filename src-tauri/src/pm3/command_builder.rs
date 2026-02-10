/// PM3 CLI command strings.
/// All commands assume the Iceman fork with `-f` flag for subprocess piping.

pub fn build_detect_command() -> &'static str {
    "hw version"
}

pub fn build_lf_search() -> &'static str {
    "lf search"
}

pub fn build_hf_search() -> &'static str {
    "hf search"
}

pub fn build_t5577_detect() -> &'static str {
    "lf t55xx detect"
}

pub fn build_t5577_chk() -> &'static str {
    "lf t55xx chk"
}

pub fn build_t5577_wipe() -> &'static str {
    "lf t55xx wipe"
}

pub fn build_em4100_clone(id: &str) -> String {
    format!("lf em 410x clone --id {}", id)
}

pub fn build_hid_clone(fc: u32, cn: u32) -> String {
    format!("lf hid clone -w H26 --fc {} --cn {}", fc, cn)
}

pub fn build_hid_clone_raw(raw: &str) -> String {
    format!("lf hid clone -r {}", raw)
}

pub fn build_indala_clone(raw: &str) -> String {
    format!("lf indala clone --raw {}", raw)
}

pub fn build_ioprox_clone(raw: &str) -> String {
    format!("lf io clone --raw {}", raw)
}

pub fn build_awid_clone(fc: u32, cn: u32) -> String {
    format!("lf awid clone --fc {} --cn {}", fc, cn)
}

pub fn build_paradox_clone(raw: &str) -> String {
    format!("lf paradox clone --raw {}", raw)
}

pub fn build_viking_clone(raw: &str) -> String {
    format!("lf viking clone --raw {}", raw)
}

pub fn build_pyramid_clone(fc: u32, cn: u32) -> String {
    format!("lf pyramid clone --fc {} --cn {}", fc, cn)
}

pub fn build_keri_clone(raw: &str) -> String {
    format!("lf keri clone --raw {}", raw)
}

pub fn build_nexwatch_clone(raw: &str) -> String {
    format!("lf nexwatch clone --raw {}", raw)
}

pub fn build_fdxb_clone(raw: &str) -> String {
    format!("lf fdxb clone --raw {}", raw)
}

/// Build the appropriate clone command for a given card type + data.
/// Returns None if clone is not supported for this type.
pub fn build_clone_command(
    card_type: &crate::cards::types::CardType,
    uid: &str,
    decoded: &std::collections::HashMap<String, String>,
) -> Option<String> {
    use crate::cards::types::CardType;

    match card_type {
        CardType::EM4100 => Some(build_em4100_clone(uid)),

        CardType::HIDProx => {
            if let (Some(fc), Some(cn)) = (decoded.get("facility_code"), decoded.get("card_number"))
            {
                if let (Ok(fc_n), Ok(cn_n)) = (fc.parse::<u32>(), cn.parse::<u32>()) {
                    return Some(build_hid_clone(fc_n, cn_n));
                }
            }
            if let Some(raw) = decoded.get("raw") {
                Some(build_hid_clone_raw(raw))
            } else {
                None
            }
        }

        CardType::Indala => Some(build_indala_clone(uid)),
        CardType::IOProx => Some(build_ioprox_clone(uid)),

        CardType::AWID => {
            if let (Some(fc), Some(cn)) = (decoded.get("facility_code"), decoded.get("card_number"))
            {
                if let (Ok(fc_n), Ok(cn_n)) = (fc.parse::<u32>(), cn.parse::<u32>()) {
                    return Some(build_awid_clone(fc_n, cn_n));
                }
            }
            None
        }

        CardType::Paradox => Some(build_paradox_clone(uid)),
        CardType::Viking => Some(build_viking_clone(uid)),

        CardType::Pyramid => {
            if let (Some(fc), Some(cn)) = (decoded.get("facility_code"), decoded.get("card_number"))
            {
                if let (Ok(fc_n), Ok(cn_n)) = (fc.parse::<u32>(), cn.parse::<u32>()) {
                    return Some(build_pyramid_clone(fc_n, cn_n));
                }
            }
            None
        }

        CardType::Keri => Some(build_keri_clone(uid)),
        CardType::NexWatch => Some(build_nexwatch_clone(uid)),
        CardType::FDX_B => Some(build_fdxb_clone(uid)),

        // HF cloning not yet implemented in this module
        CardType::MifareClassic1K
        | CardType::MifareClassic4K
        | CardType::MifareUltralight
        | CardType::NTAG
        | CardType::DESFire
        | CardType::IClass => None,
    }
}
