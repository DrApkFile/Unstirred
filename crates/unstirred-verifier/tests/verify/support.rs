//! What a refusal has to look like for these tests, and the one-hex-digit change that keeps a
//! package parseable so the refusal must come from the check under test.

use std::error::Error;

use unstirred_core::Code;
use unstirred_verifier::Verdict;

pub type TestResult = Result<(), Box<dyn Error>>;

pub fn expect_code(verdict: Verdict, code: Code) -> TestResult {
    match verdict {
        Verdict::Refused(refusal) => {
            assert_eq!(
                refusal.code, code,
                "expected {code}, got {} ({})",
                refusal.code, refusal.detail
            );
            Ok(())
        }
        other => Err(format!("expected a refusal with {code}, got {other:?}").into()),
    }
}

/// Changes one hex digit without changing the field's length, so the package still parses and
/// the refusal has to come from the check under test, not from the reader.
pub fn flip_one_hex_digit(text: &str) -> String {
    let mut chars: Vec<char> = text.chars().collect();
    let first = chars.first().copied().unwrap_or('0');
    let replacement = if first == '0' { '1' } else { '0' };
    if let Some(slot) = chars.first_mut() {
        *slot = replacement;
    }
    chars.into_iter().collect()
}
