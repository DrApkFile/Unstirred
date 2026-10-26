#![doc = "The one file a gate operator writes, and every reason not to start. A gate that cannot \
read its rooms, its database path, its root sources or its pinned h1 root never listens: a \
half-configured door is worse than a closed one. Paths inside the file are resolved against the \
file's own directory, so a gate and its manifests move together."]

mod rooms;

use std::net::SocketAddr;
use std::path::{Path, PathBuf};

use serde::Deserialize;
use unstirred_core::Refusal;
use unstirred_verifier::parse_root_source_spec;

pub use rooms::Room;

/// The public service addresses a gate's holders need, as that gate advertises them. These are
/// the operator's own infrastructure — a lightwalletd and an nf-server — and they are the same
/// two values for every holder who knocks, so publishing them says nothing about any one of them
/// (LEAKS L18). Both are optional; a gate that sets neither advertises neither.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Services {
    /// The nullifier-PIR server a holder may use for the private lookup at h1.
    pub pir_url: Option<String>,
    /// The lightwalletd a holder may use to read the tree state at h0.
    pub lightwalletd_url: Option<String>,
}

impl Services {
    /// Whether this gate advertises anything at all, which is what decides if `GET /rooms`
    /// carries the field.
    pub fn any(&self) -> bool {
        self.pir_url.is_some() || self.lightwalletd_url.is_some()
    }
}

use rooms::{resolve_rooms, RoomFile};

/// One gate's whole configuration, already checked. Holding this value means the gate may listen.
#[derive(Debug)]
pub struct GateConfig {
    /// The address and port to bind.
    pub listen: SocketAddr,
    /// The gate's SQLite file, resolved against the config file's own directory.
    pub database: PathBuf,
    /// The root-source specs, kept as text: each admission builds its sources afresh.
    pub root_source_specs: Vec<String>,
    /// Whether the operator opted in to one root source. Every output then carries the
    /// single-source note.
    pub single_root_source: bool,
    /// The rooms this gate hosts, each with its manifest and its pinned h1 root.
    pub rooms: Vec<Room>,
    /// The public service addresses this gate advertises, if any.
    pub services: Services,
}

/// Why a gate will not start. Every one of these is settled before the socket is bound.
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    /// The config file could not be read.
    #[error("cannot read the gate config {path}: {source}")]
    Read {
        /// The config path that was tried.
        path: PathBuf,
        /// What the filesystem reported.
        source: std::io::Error,
    },
    /// The config file is not TOML.
    #[error("the gate config {path} is not valid TOML: {source}")]
    Toml {
        /// The config path.
        path: PathBuf,
        /// What the parser reported, including the unknown key for a field the gate would ignore.
        source: toml::de::Error,
    },
    /// `listen` is not an address and port that can be bound.
    #[error("listen = {value:?} is not an address and port the gate can bind: {detail}")]
    Listen {
        /// The value as written.
        value: String,
        /// What is wrong with it.
        detail: String,
    },
    /// A room's manifest file could not be read.
    #[error("the room manifest {path} cannot be read: {source}")]
    ManifestRead {
        /// The manifest path, resolved against the config's directory.
        path: PathBuf,
        /// What the filesystem reported.
        source: std::io::Error,
    },
    /// A room's manifest is not a manifest a holder could prove against.
    #[error("the room manifest {path} is not a valid gate manifest: {refusal}")]
    Manifest {
        /// The manifest path.
        path: PathBuf,
        /// The refusal `parse_manifest` gave, naming the term that is wrong.
        refusal: Refusal,
    },
    /// Two rooms claim one context. A context scopes a gate's markers, so two rooms sharing one
    /// would share single use.
    #[error("two rooms share the context {context:?}; a context is a gate's name and scopes its markers")]
    DuplicateContext {
        /// The context both rooms claim.
        context: String,
    },
    /// The config lists no room, so there would be nothing to admit anyone to.
    #[error("the gate hosts no rooms; list at least one [[rooms]] with a manifest")]
    NoRooms,
    /// The number of root sources does not match the two-source rule, or the single-source opt-in.
    #[error("{detail}")]
    SourceCount {
        /// What was listed and what is required.
        detail: String,
    },
    /// One `root_sources` entry is not a usable source spec.
    #[error("root_sources entry {spec:?} cannot be used: {detail}")]
    SourceSpec {
        /// The spec as written.
        spec: String,
        /// What is wrong with it.
        detail: String,
    },
    /// A room has no pinned h1 nullifier-set root, so no proof for it could be checked.
    #[error("room {context:?} has no pinned h1 root: set h1_root for the room or for the whole gate")]
    H1RootMissing {
        /// The room that is missing one.
        context: String,
    },
    /// A pinned h1 root is not 32 bytes of hex.
    #[error("h1_root {value:?} is not 32 bytes of hex: {detail}")]
    H1Root {
        /// The value as written.
        value: String,
        /// What is wrong with it.
        detail: String,
    },
    /// An advertised service address is not one a holder could reach (A49).
    #[error("{field} = {value:?} cannot be advertised to holders: {detail}")]
    ServiceUrl {
        /// Which key it was: `pir_url` or `lightwalletd_url`.
        field: &'static str,
        /// The value as written.
        value: String,
        /// What is wrong with it.
        detail: String,
    },
    #[error(
        "rooms {first:?} (h1 {first_h1}) and {second:?} (h1 {second_h1}) pin the same h1 nullifier-set \
         root, but their windows end at different heights; give each room its own h1_root"
    )]
    /// Two rooms pin one h1 root but end at different heights, so at most one of them can be
    /// right. The gate refuses rather than admit against a root that is not its own window's.
    SharedH1Root {
        /// The first room's context.
        first: String,
        /// Its window's h1.
        first_h1: u32,
        /// The second room's context.
        second: String,
        /// Its window's h1.
        second_h1: u32,
    },
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ConfigFile {
    listen: String,
    database: String,
    root_sources: Vec<String>,
    #[serde(default)]
    single_root_source: bool,
    #[serde(default)]
    h1_root: Option<String>,
    #[serde(default)]
    pir_url: Option<String>,
    #[serde(default)]
    lightwalletd_url: Option<String>,
    #[serde(default)]
    rooms: Vec<RoomFile>,
}

/// Reads and checks one gate config. Every refusal here happens before the socket is bound: a
/// half-configured door is worse than a closed one.
///
/// ```
/// use unstirred_gate::load_config;
///
/// // Nothing about a missing file is guessed at, and nothing starts.
/// assert!(load_config("no-such-gate.toml").is_err());
/// ```
pub fn load_config(path: impl AsRef<Path>) -> Result<GateConfig, ConfigError> {
    let path = path.as_ref();
    let text = std::fs::read_to_string(path).map_err(|source| ConfigError::Read {
        path: path.to_path_buf(),
        source,
    })?;
    let file: ConfigFile = toml::from_str(&text).map_err(|source| ConfigError::Toml {
        path: path.to_path_buf(),
        source,
    })?;
    let beside = path.parent().unwrap_or(Path::new(".")).to_path_buf();
    Ok(GateConfig {
        listen: parse_listen(&file.listen)?,
        database: beside.join(&file.database),
        root_source_specs: checked_sources(&file.root_sources, file.single_root_source)?,
        single_root_source: file.single_root_source,
        rooms: resolve_rooms(&file.rooms, file.h1_root.as_deref(), &beside)?,
        services: Services {
            pir_url: advertisable("pir_url", file.pir_url)?,
            lightwalletd_url: advertisable("lightwalletd_url", file.lightwalletd_url)?,
        },
    })
}

/// An address a gate is willing to put its name to. A holder is going to send a query to whatever
/// this says, so a gate that cannot say where does not start: the refusal belongs here, before the
/// socket is bound, and not in the holder's terminal (A49).
fn advertisable(field: &'static str, value: Option<String>) -> Result<Option<String>, ConfigError> {
    let Some(value) = value else {
        return Ok(None);
    };
    let fail = |detail: String| ConfigError::ServiceUrl {
        field,
        value: value.clone(),
        detail,
    };
    let uri: http::Uri = value.parse().map_err(|e| fail(format!("{e}")))?;
    match (uri.scheme_str(), uri.host()) {
        (Some("http") | Some("https"), Some(_)) => Ok(Some(value)),
        (scheme, _) => Err(fail(format!(
            "a holder reaches it over http or https and needs a host, not {scheme:?}"
        ))),
    }
}

fn parse_listen(value: &str) -> Result<SocketAddr, ConfigError> {
    value.parse().map_err(|e| ConfigError::Listen {
        value: value.to_owned(),
        detail: format!("{e}"),
    })
}

/// Builds every source once so an unusable spec is a startup refusal, then keeps the specs: the
/// sources themselves are built again inside each admission, which is where they are read.
fn checked_sources(specs: &[String], single: bool) -> Result<Vec<String>, ConfigError> {
    match (specs.len(), single) {
        (1, true) | (2, false) => {}
        (count, true) => {
            return Err(ConfigError::SourceCount {
                detail: format!("single_root_source = true takes exactly one root source, the config lists {count}"),
            })
        }
        (count, false) => {
            return Err(ConfigError::SourceCount {
                detail: format!(
                    "exactly two root sources are required, or one with single_root_source = true; \
                     the config lists {count}"
                ),
            })
        }
    }
    for spec in specs {
        parse_root_source_spec(spec).map_err(|detail| ConfigError::SourceSpec {
            spec: spec.clone(),
            detail,
        })?;
    }
    Ok(specs.to_vec())
}
