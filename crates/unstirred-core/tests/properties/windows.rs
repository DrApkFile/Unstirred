//! A28: the window rule over random heights and tips, including the top of the height range,
//! where the finality subtraction has to saturate rather than wrap.

use rand::rngs::StdRng;
use rand::SeedableRng;
use unstirred_core::{check_window, Code, FINALITY_DEPTH, NULLIFIER_SYNC_STEP};

use super::generate::{random_tip, random_window};
use super::seeded::CASES;

/// A28: the window rule accepts exactly the windows PRD section 3 names and refuses every other,
/// including at the top of the height range, where the finality subtraction has to saturate.
#[test]
fn a28_the_window_rule_accepts_exactly_ordered_final_heights_on_the_nullifier_step() {
    let mut failures = Vec::new();
    let mut accepted = 0u32;
    for seed in 0..CASES {
        let mut rng = StdRng::seed_from_u64(seed);
        let window = random_window(&mut rng);
        let tip = random_tip(&mut rng, window.h1);
        let allowed = window.h0 < window.h1
            && window.h1.is_multiple_of(NULLIFIER_SYNC_STEP)
            && window.h1 <= tip.saturating_sub(FINALITY_DEPTH);
        if allowed {
            accepted += 1;
        }
        let outcome = check_window(window, tip);
        let agreed = match (allowed, &outcome) {
            (true, Ok(())) => true,
            (false, Err(refusal)) => refusal.code == Code::WindowInvalid,
            _ => false,
        };
        if !agreed {
            failures.push((
                seed,
                format!(
                    "h0 {}, h1 {}, tip {tip}: the rule allows it = {allowed}, the check said {outcome:?}",
                    window.h0, window.h1
                ),
            ));
        }
    }
    assert!(failures.is_empty(), "windows judged wrongly: {failures:#?}");
    assert!(
        accepted > 0,
        "no case produced a valid window, so this property proved nothing"
    );
}
