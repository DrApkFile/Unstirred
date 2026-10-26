#![doc = "What a running gate holds: its rooms by context, the path of the one database it opens \
per request, its root authority, its clock, its log and the in-memory feed of verdicts it has \
just answered. No connection, no source and no secret lives here, so every request can be \
answered from a blocking task without sharing anything that cannot cross threads."]

use std::collections::BTreeMap;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

use unstirred_verifier::store::{Clock, SystemClock};

use crate::config::{GateConfig, Room, Services};
use crate::events::{GateLog, StdoutLog};
use crate::recent::RecentVerdicts;
use crate::roots::{RootAuthority, SpecRootAuthority};

/// The largest admit body a gate will read. A real package is about 24 KB; anything near this
/// limit is not one, and nothing above it is parsed, verified or stored (A37).
pub const MAX_ADMIT_BODY_BYTES: usize = 256 * 1024;

/// The most rows a room's log answers with, newest first. It is a cap on the answer, not a
/// count of what the gate stores: a room with nothing in it answers with this limit and an
/// empty list.
pub const ROOM_LOG_LIMIT: u32 = 50;

/// Everything a running gate holds, cloneable because none of it is a connection or a secret.
#[derive(Debug, Clone)]
pub struct GateState {
    /// The address the gate was asked to bind.
    pub listen: SocketAddr,
    /// The database path. It is opened per request, not held open here.
    pub database: PathBuf,
    /// Where the h0 root and the tip come from.
    pub roots: Arc<dyn RootAuthority>,
    /// The clock every expiry check uses.
    pub clock: Arc<dyn Clock + Send + Sync>,
    /// Where the gate's own lines go.
    pub log: Arc<dyn GateLog>,
    /// The verdicts this process has answered, for the room pages. Never written to disk.
    pub recent: Arc<RecentVerdicts>,
    /// The public service addresses this gate advertises to its holders, if any (A49).
    pub services: Arc<Services>,
    rooms: Arc<BTreeMap<String, Room>>,
}

impl GateState {
    /// The state a served gate runs with: the config's own root sources, the system clock and
    /// standard output.
    pub fn from_config(config: GateConfig) -> Result<Self, String> {
        let roots = Arc::new(SpecRootAuthority::new(&config.root_source_specs)?);
        Ok(Self::new(config, roots, Arc::new(SystemClock), Arc::new(StdoutLog)))
    }

    /// The state with the root authority, clock and log given directly, which is how a test
    /// hands the gate fixed roots and a readable log.
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
            services: Arc::new(config.services),
            roots,
            clock,
            log,
            recent: Arc::new(RecentVerdicts::default()),
            rooms: Arc::new(rooms),
        }
    }

    /// The room by that name, or nothing. A gate hosts the rooms its config named and no
    /// others, so this is where an unknown context stops.
    pub fn room(&self, context: &str) -> Option<&Room> {
        self.rooms.get(context)
    }

    /// Every room this gate hosts, by context.
    pub fn rooms(&self) -> impl Iterator<Item = &Room> {
        self.rooms.values()
    }

    /// The single-source label when one applies, which every answer and every startup line
    /// carries (A27, A39).
    pub fn note(&self) -> Option<&'static str> {
        self.roots.note()
    }
}
