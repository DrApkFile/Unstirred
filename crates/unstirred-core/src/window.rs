use crate::codes::{Code, Refusal};
use crate::manifest::Window;

pub const NULLIFIER_SYNC_STEP: u32 = 10;
pub const FINALITY_DEPTH: u32 = 100;

/// The window rules that do not need a chain: h0 below h1, and an h1 the nullifier server can
/// snapshot. A gate owner can be told about these before any node is reachable, which is what
/// `unstirred gate init` does with them.
pub fn check_window_shape(window: Window) -> Result<(), Refusal> {
    if window.h0 >= window.h1 {
        return Err(Refusal::new(Code::WindowInvalid, "h0 must be strictly below h1"));
    }
    if !window.h1.is_multiple_of(NULLIFIER_SYNC_STEP) {
        return Err(Refusal::new(
            Code::WindowInvalid,
            "h1 must be a multiple of 10 (nullifier snapshot step)",
        ));
    }
    Ok(())
}

pub fn check_window(window: Window, chain_tip: u32) -> Result<(), Refusal> {
    check_window_shape(window)?;
    let final_height = chain_tip.saturating_sub(FINALITY_DEPTH);
    if window.h1 > final_height {
        return Err(Refusal::new(
            Code::WindowInvalid,
            format!(
                "h1 {} is not final yet: chain tip {chain_tip}, need h1 <= tip - {FINALITY_DEPTH}",
                window.h1
            ),
        ));
    }
    Ok(())
}
