use unicode_normalization::UnicodeNormalization;

/// Canonical form of an action string
///
/// This prevents semantic-gap attacks where different encodings
/// of the same action bypass checks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonicalAction {
    /// The canonicalized form
    pub canonical_form: String,

    /// Original input (for auditing)
    pub original: String,
}

/// Canonicalize an action string to prevent semantic-gap attacks
///
/// Applies:
/// 1. Null byte removal (anti-injection)
/// 2. Unicode normalization (NFC)
/// 3. Whitespace trimming
/// 4. Optional case folding
pub fn canonicalize_action(raw: &str) -> CanonicalAction {
    // 1. Remove null bytes (anti-injection)
    let sanitized = raw.replace('\0', "");

    // 2. Unicode normalization (NFC - composed form)
    let normalized: String = sanitized.nfc().collect();

    // 3. Trim whitespace
    let trimmed = normalized.trim();

    // 4. Create canonical form
    let canonical = trimmed.to_string();

    CanonicalAction {
        canonical_form: canonical,
        original: raw.to_string(),
    }
}

/// Canonicalize and convert to lowercase (for case-insensitive matching)
pub fn canonicalize_case_insensitive(raw: &str) -> CanonicalAction {
    let mut result = canonicalize_action(raw);
    result.canonical_form = result.canonical_form.to_lowercase();
    result
}

/// Check if two action strings are semantically equivalent
pub fn are_equivalent(a: &str, b: &str) -> bool {
    let canon_a = canonicalize_action(a);
    let canon_b = canonicalize_action(b);
    canon_a.canonical_form == canon_b.canonical_form
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_null_byte_removal() {
        let input = "delete\0/etc/passwd";
        let canonical = canonicalize_action(input);

        assert_eq!(canonical.canonical_form, "delete/etc/passwd");
        assert!(!canonical.canonical_form.contains('\0'));
    }

    #[test]
    fn test_unicode_normalization() {
        // é can be represented as single char (U+00E9) or e + combining accent (U+0065 U+0301)
        let composed = "café";
        let decomposed = "cafe\u{0301}";

        let canon_composed = canonicalize_action(composed);
        let canon_decomposed = canonicalize_action(decomposed);

        // Both should normalize to same form
        assert_eq!(
            canon_composed.canonical_form,
            canon_decomposed.canonical_form
        );
    }

    #[test]
    fn test_whitespace_trimming() {
        let input = "  delete file.txt  ";
        let canonical = canonicalize_action(input);

        assert_eq!(canonical.canonical_form, "delete file.txt");
    }

    #[test]
    fn test_case_insensitive() {
        let input = "DELETE File.TXT";
        let canonical = canonicalize_case_insensitive(input);

        assert_eq!(canonical.canonical_form, "delete file.txt");
    }

    #[test]
    fn test_preserves_original() {
        let input = "  DELETE\0file  ";
        let canonical = canonicalize_action(input);

        assert_eq!(canonical.original, input);
        assert_ne!(canonical.canonical_form, input);
    }

    #[test]
    fn test_equivalence_checking() {
        assert!(are_equivalent("delete file", "delete file"));
        assert!(are_equivalent("delete file", "  delete file  "));
        assert!(!are_equivalent("delete file", "remove file"));
    }

    #[test]
    fn test_complex_unicode() {
        // Various unicode representations should normalize
        let input1 = "naïve";
        let input2 = "nai\u{0308}ve";

        let canon1 = canonicalize_action(input1);
        let canon2 = canonicalize_action(input2);

        assert_eq!(canon1.canonical_form, canon2.canonical_form);
    }

    #[test]
    fn test_empty_string() {
        let canonical = canonicalize_action("");
        assert_eq!(canonical.canonical_form, "");
    }

    #[test]
    fn test_only_whitespace() {
        let canonical = canonicalize_action("   \t\n  ");
        assert_eq!(canonical.canonical_form, "");
    }
}
