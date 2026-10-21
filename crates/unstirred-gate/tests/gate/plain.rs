//! A gate on a real socket with two rooms and no wallet at all. Nothing here proves anything, so
//! nothing here costs a Halo2 proof: these are the tests of the page, the routes and the feed,
//! which never need a package that verifies. The roots are injected and no URL is contacted.

use std::error::Error;
use std::path::Path;
use std::sync::Arc;

use unstirred_core::{build_manifest, Network, Window};
use unstirred_gate::{load_config, start, CollectedLog, GateClient, GateState, RunningGate};
use unstirred_verifier::store::Clock;

use super::roots::FixedRoots;

pub const HOLDERS_ROOM: &str = "holders-room";
pub const SECOND_ROOM: &str = "second-room";
pub const H0: u32 = 4_505_790;
pub const H1: u32 = 4_506_510;
const TIP: u32 = H1 + 10_000;
const NOW: i64 = 1_700_000_000;
const H1_ROOT_HEX: &str = "5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a";

/// The gate's clock, stopped, so every time this fixture puts on screen is the same one.
#[derive(Debug, Clone, Copy)]
pub struct StoppedClock;

impl Clock for StoppedClock {
    fn unix_seconds(&self) -> i64 {
        NOW
    }
}

pub struct PlainGate {
    pub client: GateClient,
    _gate: RunningGate,
}

/// Starts a gate whose database lives in `directory`. Calling it twice on one directory is a
/// restart: the file is the same and the process is not, which is what A43 needs.
pub fn start_in(directory: &Path) -> Result<PlainGate, Box<dyn Error>> {
    for context in [HOLDERS_ROOM, SECOND_ROOM] {
        let json = build_manifest(Network::Test, context, Window { h0: H0, h1: H1 })?;
        std::fs::write(directory.join(format!("{context}.json")), &json)?;
    }
    let config_path = directory.join("gate.toml");
    std::fs::write(&config_path, config_toml())?;
    let config = load_config(&config_path)?;
    let roots = FixedRoots {
        nc_root_at_h0: [0x5b; 32],
        tip: TIP,
        sources: vec!["fake:first".to_owned(), "fake:second".to_owned()],
    };
    let gate = start(GateState::new(
        config,
        Arc::new(roots),
        Arc::new(StoppedClock),
        Arc::new(CollectedLog::default()),
    ))?;
    let client = GateClient::new(&gate.url())?;
    Ok(PlainGate { client, _gate: gate })
}

fn config_toml() -> String {
    format!(
        r#"listen = "127.0.0.1:0"
database = "gate.sqlite"
root_sources = ["lwd:https://first.invalid:443", "zebra:http://second.invalid:18232"]
h1_root = "{H1_ROOT_HEX}"

[[rooms]]
manifest = "{HOLDERS_ROOM}.json"

[[rooms]]
manifest = "{SECOND_ROOM}.json"
"#
    )
}
