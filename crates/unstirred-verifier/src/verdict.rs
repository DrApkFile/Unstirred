#![doc = "The PRD section 3.5 verdict object, and the mapping from a refusal to the verdict kind the \
closed code set assigns it."]

use unstirred_core::{Refusal, VerdictKind};

use crate::engine::Verdict;

pub(crate) fn as_verdict(refusal: Refusal) -> Verdict {
    match refusal.code.spec().kind {
        VerdictKind::Refused => Verdict::Refused(refusal),
        VerdictKind::Inconclusive => Verdict::Inconclusive(refusal),
    }
}

/// The PRD section 3.5 verdict object for `--json`: the verdict alone when admitted, plus code,
/// message and retryable on any refusal. The single-source note, when one applies, is the only
/// other field this object may carry.
pub fn verdict_object(verdict: &Verdict, note: Option<&str>) -> String {
    let mut object = match verdict {
        Verdict::Admitted => String::from("{\"verdict\":\"ADMITTED\""),
        Verdict::Refused(refusal) | Verdict::Inconclusive(refusal) => {
            let spec = refusal.code.spec();
            format!(
                "{{\"verdict\":\"{}\",\"code\":\"{}\",\"message\":\"{}\",\"retryable\":{}",
                spec.kind,
                spec.wire_name,
                json_escape(spec.message),
                spec.retryable
            )
        }
    };
    if let Some(note) = note {
        object.push_str(&format!(",\"note\":\"{}\"", json_escape(note)));
    }
    object.push('}');
    object
}

fn json_escape(text: &str) -> String {
    text.replace('\\', "\\\\").replace('"', "\\\"")
}
