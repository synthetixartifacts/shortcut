//! Shared filter/name predicates for model discovery.
//!
//! Used by the OpenAI and xAI fetchers to trim noise (image/audio/moderation
//! models, snapshot IDs) and pick a canonical alias when multiple are offered.

use std::collections::HashSet;

/// Return `true` if the model id belongs to ShortCut's supported OpenAI text family.
pub(super) fn is_openai_shortcut_model(id: &str) -> bool {
    let lower = id.to_ascii_lowercase();
    let is_text_family = lower.starts_with("gpt-") || lower.starts_with('o');
    if !is_text_family {
        return false;
    }

    ![
        "audio",
        "realtime",
        "transcribe",
        "tts",
        "embedding",
        "moderation",
        "image",
        "dall",
        "sora",
        "whisper",
        "codex",
        "search-preview",
        "computer-use",
        "omni-moderation",
        "gpt-oss",
    ]
    .iter()
    .any(|term| lower.contains(term))
}

/// `true` if the id looks like a dated snapshot (`base-YYYY-MM-DD` or `base-YYYYMMDD`).
pub(super) fn looks_like_snapshot(id: &str) -> bool {
    snapshot_base(id).is_some()
}

/// If `id` ends with a date suffix, return the base prefix (everything before the
/// dash + date). Returns `None` when there is no date suffix.
pub(super) fn snapshot_base(id: &str) -> Option<&str> {
    if id.len() > 11 && id.as_bytes()[id.len() - 11] == b'-' && is_iso_date(&id[id.len() - 10..]) {
        return Some(&id[..id.len() - 11]);
    }
    if id.len() > 9
        && id.as_bytes()[id.len() - 9] == b'-'
        && id[id.len() - 8..].chars().all(|ch| ch.is_ascii_digit())
    {
        return Some(&id[..id.len() - 9]);
    }
    None
}

/// `true` if `suffix` is a trailing `-YYYYMMDD`-style date fragment.
pub(super) fn has_snapshot_suffix(suffix: &str) -> bool {
    suffix.starts_with('-') && suffix[1..].chars().all(|ch| ch.is_ascii_digit())
}

/// `true` if `id` has a dated-snapshot sibling already present in `available`.
pub(super) fn has_snapshot_alias(id: &str, available: &HashSet<String>) -> bool {
    snapshot_base(id).is_some_and(|base| available.contains(base))
}

/// Pick the most user-friendly alias for an xAI model id.
///
/// Preference order:
/// 1. A fixed, non-dated alias (e.g. `grok-2`)
/// 2. A `*-latest` alias
/// 3. The first declared alias
/// 4. The raw id as a fallback.
pub(super) fn preferred_xai_name(id: &str, aliases: &[String]) -> String {
    aliases
        .iter()
        .find(|alias| !alias.contains("latest") && !looks_like_snapshot(alias))
        .cloned()
        .or_else(|| aliases.iter().find(|alias| alias.ends_with("-latest")).cloned())
        .or_else(|| aliases.first().cloned())
        .unwrap_or_else(|| id.to_string())
}

/// Validate a 10-char ISO-8601 date (`YYYY-MM-DD`).
fn is_iso_date(value: &str) -> bool {
    value.len() == 10
        && value.chars().enumerate().all(|(idx, ch)| match idx {
            4 | 7 => ch == '-',
            _ => ch.is_ascii_digit(),
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_openai_shortcut_model_accepts_text_family() {
        assert!(is_openai_shortcut_model("gpt-4o"));
        assert!(is_openai_shortcut_model("gpt-4o-mini"));
        assert!(is_openai_shortcut_model("o1"));
        assert!(is_openai_shortcut_model("o3-mini"));
    }

    #[test]
    fn is_openai_shortcut_model_rejects_non_text() {
        assert!(!is_openai_shortcut_model("dall-e-3"));
        assert!(!is_openai_shortcut_model("whisper-1"));
        assert!(!is_openai_shortcut_model("tts-1"));
        assert!(!is_openai_shortcut_model("gpt-4o-transcribe"));
        assert!(!is_openai_shortcut_model("gpt-4o-audio-preview"));
        assert!(!is_openai_shortcut_model("omni-moderation-latest"));
        assert!(!is_openai_shortcut_model("gpt-oss"));
        assert!(!is_openai_shortcut_model("sora-1"));
        assert!(!is_openai_shortcut_model("codex"));
    }

    #[test]
    fn is_openai_shortcut_model_rejects_unrelated_families() {
        // Not starting with "gpt-" or "o"
        assert!(!is_openai_shortcut_model("claude-3-opus"));
        assert!(!is_openai_shortcut_model("gemini-pro"));
    }

    #[test]
    fn is_iso_date_accepts_well_formed() {
        assert!(is_iso_date("2024-05-13"));
        assert!(is_iso_date("1999-12-31"));
    }

    #[test]
    fn is_iso_date_rejects_malformed() {
        assert!(!is_iso_date("2024/05/13"));
        assert!(!is_iso_date("24-05-13"));
        assert!(!is_iso_date("2024-5-13"));
        assert!(!is_iso_date(""));
        assert!(!is_iso_date("not-a-date"));
    }

    #[test]
    fn snapshot_base_handles_dashed_iso() {
        assert_eq!(snapshot_base("gpt-4o-2024-05-13"), Some("gpt-4o"));
        assert_eq!(snapshot_base("claude-3-5-sonnet-2024-06-20"), Some("claude-3-5-sonnet"));
    }

    #[test]
    fn snapshot_base_handles_compact_yyyymmdd() {
        assert_eq!(snapshot_base("claude-3-opus-20240229"), Some("claude-3-opus"));
    }

    #[test]
    fn snapshot_base_none_when_no_date() {
        assert_eq!(snapshot_base("gpt-4o"), None);
        assert_eq!(snapshot_base("grok-2"), None);
    }

    #[test]
    fn looks_like_snapshot_matches_dated_ids() {
        assert!(looks_like_snapshot("gpt-4o-2024-05-13"));
        assert!(looks_like_snapshot("claude-3-opus-20240229"));
        assert!(!looks_like_snapshot("gpt-4o"));
    }

    #[test]
    fn has_snapshot_suffix_recognises_date_fragment() {
        assert!(has_snapshot_suffix("-20240229"));
        assert!(!has_snapshot_suffix("20240229"));
        assert!(!has_snapshot_suffix("-latest"));
    }

    #[test]
    fn has_snapshot_alias_finds_base() {
        let mut available = HashSet::new();
        available.insert("gpt-4o".to_string());
        assert!(has_snapshot_alias("gpt-4o-2024-05-13", &available));
        assert!(!has_snapshot_alias("gpt-4o-mini-2024-05-13", &available));
    }

    #[test]
    fn preferred_xai_name_prefers_non_dated_alias() {
        let aliases = vec![
            "grok-2-2024-08-13".to_string(),
            "grok-2-latest".to_string(),
            "grok-2".to_string(),
        ];
        assert_eq!(preferred_xai_name("grok-2-2024-08-13", &aliases), "grok-2");
    }

    #[test]
    fn preferred_xai_name_falls_back_to_latest_then_first() {
        let aliases = vec!["grok-2-latest".to_string()];
        assert_eq!(preferred_xai_name("id", &aliases), "grok-2-latest");

        let aliases: Vec<String> = vec![];
        assert_eq!(preferred_xai_name("grok-abc", &aliases), "grok-abc");
    }
}
