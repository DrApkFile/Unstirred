//! The pure rules over thousands of random cases (A28-A31). **Synthetic and offline:** nothing
//! here opens a wallet, a database or a socket. Every case is generated from its own seed, and
//! a failure prints the seed that produced it, so one printed number reproduces one case.

mod properties {
    pub mod binding;
    pub mod generate;
    pub mod manifest_rule;
    pub mod round_id;
    pub mod seeded;
    pub mod windows;
}
