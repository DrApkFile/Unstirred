#![doc = "What a running gate holds: its rooms by context, the path of the one database it opens \
per request, its root authority, its clock and its log. No connection, no source and no secret \
lives here, so every request can be answered from a blocking task without sharing anything that \
cannot cross threads."]

use std::collections::BTreeMap;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

use unstirred_verifier::store::{Clock, SystemClock};

use crate::config::{GateConfig, Room};
use crate::events::{GateLog, StdoutLog};
use crate::roots::{RootAuthority, SpecRootAuthority};

/// The largest admit body a gate will read. A real package is about 24 KB; anything near this
/// limit is not one, and nothing above it is parsed, verified or stored (A37).
pub const MAX_ADMIT_BODY_BYTES: usize = 256 * 1024;

/// How many rows a room's log answers with, newest first.
pub const ROOM_LOG_ROWS: u32 = 50;

#[derive(Debug, Clone)]
pub struct GateState {
    pub listen: SocketAddr,
    pub database: PathBuf,
    pub roots: Arc<dyn RootAuthority>,
    pub clock: Arc<dyn Clock + Send + Sync>,
    pub log: Arc<dyn GateLog>,
    rooms: Arc<BTreeMap<String, Room>>,
}

impl GateState {
    /// The state a served gate runs with: the config's own root sources, the system clock and
    /// standard output.
    pub fn from_config(config: GateConfig) -> Result<Self, String> {
        let roots = Arc::new(SpecRootAuthority::new(&config.root_source_specs)?);
        Ok(Self::new(config, roots, Arc::new(SystemClock), Arc::new(StdoutLog)))
    }

    pub fn new(
        config: GateConfig,
        roots: Arc<dyn RootAuthority>,
        clock: Arc<dyn Clock + Send + Sync>,
        log: Arc<dyn GateLog>,
    ) -> Self {
        let rooms = config
            .rooms
            .into_iter()
            .map(|room| (room.context().to_owned(), room))
            .collect();
        Self {
            listen: config.listen,
            database: config.database,
            roots,
            clock,
            log,
            rooms: Arc::new(rooms),
        }
    }

    pub fn room(&self, context: &str) -> Option<&Room> {
        self.rooms.get(context)
    }

    pub fn rooms(&self) -> impl Iterator<Item = &Room> {
        self.rooms.values()
    }

    /// The single-source label when one applies, which every answer and every startup line
    /// carries (A27, A39).
    pub fn note(&self) -> Option<&'static str> {
        self.roots.note()
    }
}
