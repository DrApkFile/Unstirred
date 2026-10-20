//! How the property tests run: one case per seed, every failure collected with the seed that
//! produced it, then the list asserted empty. A printed seed reproduces that one case exactly,
//! which is why the seed is the case number and nothing here reads the system clock or the OS
//! entropy pool.

use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

/// How many random cases each property runs. Large enough to reach the awkward corners, small
/// enough that the whole suite stays a few seconds.
pub const CASES: u64 = 4_000;

/// One failure: the seed that produced the case, and what was wrong with it.
pub type Failure = (u64, String);

/// Runs `case` once per seed and returns every failure, not just the first.
pub fn over_seeds(cases: u64, case: impl Fn(&mut StdRng) -> Result<(), String>) -> Vec<Failure> {
    (0..cases)
        .filter_map(|seed| {
            let mut rng = StdRng::seed_from_u64(seed);
            case(&mut rng).err().map(|failure| (seed, failure))
        })
        .collect()
}

/// A string from the alphabet a context rule has to judge: the characters it allows, and the
/// ones it must refuse.
pub fn mixed_text(rng: &mut StdRng, max_len: usize) -> String {
    const ALPHABET: &[char] = &[
        'a', 'b', 'z', '0', '5', '9', '-', 'A', 'Z', ' ', '_', '.', '/', 'é', '字',
    ];
    let len = rng.gen_range(0..=max_len);
    (0..len).map(|_| ALPHABET[rng.gen_range(0..ALPHABET.len())]).collect()
}
