use std::sync::LazyLock;

use regex::Regex;
use serde::Serialize;

use crate::pm3::output_parser::strip_ansi;

/// Parsed result from `hw version` output — contains client/firmware versions
/// and hardware variant for firmware flash decisions.
#[derive(Debug, Clone, Serialize)]
pub struct HwVersionInfo {
    pub model: String,
    pub client_version: String,
    pub os_version: String,
    /// "rdv4", "rdv4-bt", "generic", or "generic-256"
    pub hardware_variant: String,
    pub versions_match: bool,
}

// ---------------------------------------------------------------------------
// Regexes for parsing `hw version` output
// ---------------------------------------------------------------------------

/// Matches the client version line: `client: Iceman/master/v4.20728-234-g1a2b3c4d5-dirty`
static CLIENT_VERSION_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)client\s*:\s*(.+)").expect("bad client version regex")
});

/// Fallback: captures the first non-empty line after `[ Client ]` section header.
/// Real PM3 v4.20728+ outputs version directly without `client:` prefix.
static CLIENT_SECTION_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)\[\s*Client\s*\]\s*\n\s*(.+)").expect("bad client section regex")
});

/// Matches the OS (firmware) version line in both formats:
/// - Old: `os: Iceman/master/v4.20725-100-g9876543ab`
/// - Real: `OS......... Iceman/master/v4.20728-358-ga2ba91043-suspect`
static OS_VERSION_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?im)^\s*os[\s.:]+(.+)").expect("bad os version regex")
});

/// Extracts commit hash from version string: `v4.20728-234-g1a2b3c4d5-dirty` → `1a2b3c4d5`
static COMMIT_HASH_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"-g([0-9a-fA-F]{7,})").expect("bad commit hash regex")
});

/// Extracts base version: `v4.20728` from `Iceman/master/v4.20728-234-g...`
static BASE_VERSION_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"v(\d+\.\d+)").expect("bad base version regex")
});

/// Detects AT91SAM7S256 (256K flash variant)
static UC_256K_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)AT91SAM7S256").expect("bad uc 256k regex")
});

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Parse the full `hw version` output into structured version info.
///
/// Extracts client version, OS (firmware) version, hardware variant,
/// and whether the two versions match (by commit hash, then base version).
pub fn parse_detailed_hw_version(output: &str) -> HwVersionInfo {
    let clean = strip_ansi(output);

    let model = parse_model(&clean);
    let client_version = CLIENT_VERSION_RE
        .captures(&clean)
        .or_else(|| CLIENT_SECTION_RE.captures(&clean))
        .map(|c| c[1].trim().to_string())
        .unwrap_or_default();
    let os_version = OS_VERSION_RE
        .captures(&clean)
        .map(|c| c[1].trim().to_string())
        .unwrap_or_default();
    let hardware_variant = detect_hardware_variant(&clean);
    let versions_match = compare_versions(&client_version, &os_version);

    HwVersionInfo {
        model,
        client_version,
        os_version,
        hardware_variant,
        versions_match,
    }
}

/// Compare two PM3 version strings.
///
/// Strategy:
/// 1. Extract commit hashes (`-gHHHHHHH`). If both present, compare them.
/// 2. Fallback: compare base versions (`v4.NNNNN`).
/// 3. If neither is parseable, return false (mismatch — safer to prompt update).
///
/// Strips `-dirty` and `-suspect` suffixes before comparing.
pub fn compare_versions(client_ver: &str, os_ver: &str) -> bool {
    // Both empty = can't determine → mismatch
    if client_ver.is_empty() || os_ver.is_empty() {
        return false;
    }

    // Primary: compare commit hashes
    let client_commit = extract_commit_hash(client_ver);
    let os_commit = extract_commit_hash(os_ver);

    if let (Some(ref cc), Some(ref oc)) = (client_commit, os_commit) {
        return cc.eq_ignore_ascii_case(oc);
    }

    // Fallback: compare base version numbers (v4.NNNNN)
    let client_base = extract_base_version(client_ver);
    let os_base = extract_base_version(os_ver);

    if let (Some(ref cb), Some(ref ob)) = (client_base, os_base) {
        return cb == ob;
    }

    // Can't compare — assume mismatch
    false
}

/// Detect hardware variant from `hw version` output.
///
/// - `AT91SAM7S256` in uC line → `"generic-256"`
/// - `External flash: present` AND `Smartcard reader: present` AND `FPC USART` for BT → `"rdv4-bt"`
/// - `External flash: present` AND `Smartcard reader: present` → `"rdv4"`
/// - Otherwise → `"generic"`
pub fn detect_hardware_variant(output: &str) -> String {
    if UC_256K_RE.is_match(output) {
        return "generic-256".to_string();
    }

    let has_ext_flash = output
        .lines()
        .any(|l| l.to_lowercase().contains("external flash") && l.to_lowercase().contains("present"));
    let has_smartcard = output
        .lines()
        .any(|l| l.to_lowercase().contains("smartcard") && l.to_lowercase().contains("present"));

    if has_ext_flash && has_smartcard {
        // RDV4 with BlueShark BT addon has FPC USART support
        let has_bt = output
            .lines()
            .any(|l| l.to_lowercase().contains("fpc usart") && l.to_lowercase().contains("present"));
        if has_bt {
            "rdv4-bt".to_string()
        } else {
            "rdv4".to_string()
        }
    } else {
        "generic".to_string()
    }
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

fn extract_commit_hash(version: &str) -> Option<String> {
    COMMIT_HASH_RE
        .captures(version)
        .map(|c| c[1].to_lowercase())
}

fn extract_base_version(version: &str) -> Option<String> {
    BASE_VERSION_RE.captures(version).map(|c| c[1].to_string())
}

fn parse_model(output: &str) -> String {
    for line in output.lines() {
        let trimmed = line.trim();
        if (trimmed.contains("Prox") && trimmed.contains("RFID"))
            || trimmed.contains("Proxmark")
        {
            let cleaned = trimmed.trim_matches(|c: char| !c.is_alphanumeric() && c != ' ');
            if !cleaned.is_empty() {
                return cleaned.to_string();
            }
        }
    }
    "Proxmark3".to_string()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_HW_VERSION: &str = r#"
 [ Proxmark3 RFID instrument ]

 [ Client ]
  client: Iceman/master/v4.20728-234-g1a2b3c4d5

 [ ARM ]
  os: Iceman/master/v4.20728-234-g1a2b3c4d5

 [ FPGA ]
  LF image built for 2s30vq100 on 2024-01-15 at 10:30:00

 [ Hardware ]
  --= uC: AT91SAM7S512 Rev B
  --= Nonvolatile Program Memory Size: 512K bytes
  --= External flash: present
  --= Smartcard reader: present
"#;

    const SAMPLE_MISMATCH: &str = r#"
 [ Proxmark3 RFID instrument ]

 [ Client ]
  client: Iceman/master/v4.20728-234-g1a2b3c4d5

 [ ARM ]
  os: Iceman/master/v4.20725-100-g9876543ab

 [ Hardware ]
  --= uC: AT91SAM7S512 Rev B
"#;

    const SAMPLE_GENERIC_256: &str = r#"
 [ Proxmark3 RFID instrument ]

 [ Client ]
  client: Iceman/master/v4.20728

 [ ARM ]
  os: Iceman/master/v4.20728

 [ Hardware ]
  --= uC: AT91SAM7S256 Rev C
"#;

    #[test]
    fn test_parse_rdv4_matching() {
        let info = parse_detailed_hw_version(SAMPLE_HW_VERSION);
        assert_eq!(info.hardware_variant, "rdv4");
        assert!(info.versions_match);
        assert!(info.client_version.contains("v4.20728"));
        assert!(info.os_version.contains("v4.20728"));
    }

    #[test]
    fn test_parse_mismatch() {
        let info = parse_detailed_hw_version(SAMPLE_MISMATCH);
        assert!(!info.versions_match);
        assert_eq!(info.hardware_variant, "generic");
    }

    #[test]
    fn test_parse_generic_256() {
        let info = parse_detailed_hw_version(SAMPLE_GENERIC_256);
        assert_eq!(info.hardware_variant, "generic-256");
        assert!(info.versions_match); // same base version, no commit hash
    }

    #[test]
    fn test_compare_same_commit() {
        assert!(compare_versions(
            "Iceman/master/v4.20728-234-g1a2b3c4d5",
            "Iceman/master/v4.20728-234-g1a2b3c4d5"
        ));
    }

    #[test]
    fn test_compare_different_commit() {
        assert!(!compare_versions(
            "Iceman/master/v4.20728-234-g1a2b3c4d5",
            "Iceman/master/v4.20725-100-g9876543ab"
        ));
    }

    #[test]
    fn test_compare_dirty_suffix() {
        // Same commit but one has -dirty — commit hash portion still matches
        assert!(compare_versions(
            "Iceman/master/v4.20728-234-g1a2b3c4d5-dirty",
            "Iceman/master/v4.20728-234-g1a2b3c4d5"
        ));
    }

    #[test]
    fn test_compare_base_version_only() {
        // No commit hash — fallback to base version comparison
        assert!(compare_versions(
            "Iceman/master/v4.20728",
            "Iceman/master/v4.20728"
        ));
        assert!(!compare_versions(
            "Iceman/master/v4.20728",
            "Iceman/master/v4.20725"
        ));
    }

    #[test]
    fn test_compare_empty() {
        assert!(!compare_versions("", ""));
        assert!(!compare_versions("Iceman/master/v4.20728", ""));
    }

    #[test]
    fn test_detect_rdv4() {
        let output = "uC: AT91SAM7S512\nExternal flash: present\nSmartcard reader: present";
        assert_eq!(detect_hardware_variant(output), "rdv4");
    }

    #[test]
    fn test_detect_rdv4_bt() {
        let output = "uC: AT91SAM7S512\nExternal flash: present\nSmartcard reader: present\nFPC USART for BT add-on support: present";
        assert_eq!(detect_hardware_variant(output), "rdv4-bt");
    }

    #[test]
    fn test_detect_generic() {
        let output = "uC: AT91SAM7S512\nExternal flash: not present";
        assert_eq!(detect_hardware_variant(output), "generic");
    }

    #[test]
    fn test_detect_generic_256() {
        let output = "uC: AT91SAM7S256 Rev C";
        assert_eq!(detect_hardware_variant(output), "generic-256");
    }

    /// Real PM3 v4.20728 output — no `client:` prefix, `OS.........` with dots
    const SAMPLE_REAL_PM3: &str = r#"
[ Proxmark3 ]
[ Client ]
Iceman/master/v4.20728-358-ga2ba91043-suspect 2026-02-09 00:22:45 c0679a575
Compiler.................. MinGW-w64 15.2.0
Platform.................. Windows (64b) / x86_64
[ ARM ]
Bootrom.... Iceman/master/v4.20469-164-g0e95c62ad-suspect 2025-08-02 22:16:55 ef5b2e843
OS......... Iceman/master/v4.20728-358-ga2ba91043-suspect 2026-02-09 00:22:17 c0679a575
[ Hardware ]
--= uC: AT91SAM7S512 Rev B
--= Embedded flash memory 512K bytes ( 71% used )
"#;

    #[test]
    fn test_parse_real_pm3_output() {
        let info = parse_detailed_hw_version(SAMPLE_REAL_PM3);
        assert!(info.client_version.contains("v4.20728"), "client: {}", info.client_version);
        assert!(info.os_version.contains("v4.20728"), "os: {}", info.os_version);
        assert!(info.versions_match, "should match — same commit hash");
        assert_eq!(info.hardware_variant, "generic");
    }

    /// Real PM3 output with mismatched versions
    const SAMPLE_REAL_MISMATCH: &str = r#"
[ Proxmark3 ]
[ Client ]
Iceman/master/v4.20728-358-ga2ba91043-suspect 2026-02-09 00:22:45 c0679a575
[ ARM ]
Bootrom.... Iceman/master/v4.20469-164-g0e95c62ad-suspect 2025-08-02 22:16:55 ef5b2e843
OS......... Iceman/master/v4.20469-164-g0e95c62ad-suspect 2025-08-02 22:16:55 ef5b2e843
[ Hardware ]
--= uC: AT91SAM7S512 Rev B
"#;

    #[test]
    fn test_parse_real_pm3_mismatch() {
        let info = parse_detailed_hw_version(SAMPLE_REAL_MISMATCH);
        assert!(info.client_version.contains("v4.20728"), "client: {}", info.client_version);
        assert!(info.os_version.contains("v4.20469"), "os: {}", info.os_version);
        assert!(!info.versions_match, "should NOT match — different commits");
    }
}
