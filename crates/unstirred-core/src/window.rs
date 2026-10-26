#![doc = "What makes a holding window usable: an h0 below an h1, an h1 the nullifier server can \
snapshot, and an h1 far enough below the tip to be final. A window that fails any of these is \
WINDOW_INVALID, and the shape rules can be checked with no chain in reach at all."]

use crate::codes::{Code, Refusal};
use crate::manifest::Window;

/// The nullifier server snapshots the set every ten blocks, so an `h1` it could never have a
/// snapshot for is refused before anything else happens.
pub const NULLIFIER_SYNC_STEP: u32 = 10;

/// How far below the chain tip an `h1` must sit before this code will treat it as final. A window
/// that ends inside this depth could still be reorganised out from under an admission.
pub const FINALITY_DEPTH: u32 = 100;

/// The window rules that do not need a chain: h0 below h1, and an h1 the nullifier server can
/// snapshot. A gate owner can be told about these before any node is reachable, which is what
/// `unstirred gate init` does with them.
///
/// ```
/// use unstirred_core::{check_window_shape, Window};
///
/// assert!(check_window_shape(Window { h0: 4_505_790, h1: 4_506_510 }).is_ok());
/// assert!(check_window_shape(Window { h0: 4_506_510, h1: 4_505_790 }).is_err()); // h0 above h1
/// assert!(check_window_shape(Window { h0: 4_505_790, h1: 4_506_511 }).is_err()); // not a step of 10
/// ```
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

/// The shape rules, plus finality against a tip the caller fetched for itself. The verifier uses
/// this one: a window whose `h1` is not yet final is refused however well the proof verifies.
///
/// ```
/// use unstirred_core::{check_window, Window, FINALITY_DEPTH};
///
/// let window = Window { h0: 4_505_790, h1: 4_506_510 };
/// assert!(check_window(window, window.h1 + FINALITY_DEPTH).is_ok());
///
/// // One block short of final, and there is no verdict to be had.
/// let refusal = check_window(window, window.h1 + FINALITY_DEPTH - 1).unwrap_err();
/// assert_eq!(refusal.code, unstirred_core::Code::WindowInvalid);
/// ```
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
