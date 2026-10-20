//! A30: the manifest's context rule accepts exactly the charset and length it documents, and a
//! manifest that parses survives being written and read again.

use unstirred_core::{parse_manifest, Code};

use super::generate::{context_is_allowed, manifest_json};
use super::seeded::{mixed_text, over_seeds, CASES};

/// A30: the context rule accepts exactly the charset and length the manifest documents, and a
/// manifest that parses survives being written and read again unchanged.
#[test]
fn a30_the_context_rule_accepts_exactly_its_charset_and_a_manifest_round_trips() {
    let failures = over_seeds(CASES, |rng| {
        let context = mixed_text(rng, 70);
        let allowed = context_is_allowed(&context);
        let parsed = parse_manifest(&manifest_json(&context));
        match (allowed, &parsed) {
            (true, Ok(manifest)) => {
                let written =
                    serde_json::to_string(manifest).map_err(|e| format!("a parsed manifest must serialise: {e}"))?;
                let again = parse_manifest(&written).map_err(|e| format!("it must parse again: {e}"))?;
                if &again == manifest {
                    Ok(())
                } else {
                    Err(format!("context {context:?} changed through a round trip"))
                }
            }
            (false, Err(refusal)) if refusal.code == Code::ManifestInvalid => Ok(()),
            _ => Err(format!(
                "context {context:?} (len {}): the rule allows it = {allowed}, parsing said {parsed:?}",
                context.len()
            )),
        }
    });
    assert!(failures.is_empty(), "contexts judged wrongly: {failures:#?}");
}
