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

use rooms::{resolve_rooms, RoomFile};

/// One gate's whole configuration, already checked. Holding this value means the gate may listen.
#[derive(Debug)]
pub struct GateConfig {
    pub listen: SocketAddr,
    pub database: PathBuf,
    pub root_source_specs: Vec<String>,
    pub single_root_source: bool,
    pub rooms: Vec<Room>,
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("cannot read the gate config {path}: {source}")]
    Read { path: PathBuf, source: std::io::Error },
    #[error("the gate config {path} is not valid TOML: {source}")]
    Toml { path: PathBuf, source: toml::de::Error },
    #[error("listen = {value:?} is not an address and port the gate can bind: {detail}")]
    Listen { value: String, detail: String },
    #[error("the room manifest {path} cannot be read: {source}")]
    ManifestRead { path: PathBuf, source: std::io::Error },
    #[error("the room manifest {path} is not a valid gate manifest: {refusal}")]
    Manifest { path: PathBuf, refusal: Refusal },
    #[error("two rooms share the context {context:?}; a context is a gate's name and scopes its markers")]
    DuplicateContext { context: String },
    #[error("the gate hosts no rooms; list at least one [[rooms]] with a manifest")]
    NoRooms,
    #[error("{detail}")]
    SourceCount { detail: String },
    #[error("root_sources entry {spec:?} cannot be used: {detail}")]
    SourceSpec { spec: String, detail: String },
    #[error("room {context:?} has no pinned h1 root: set h1_root for the room or for the whole gate")]
    H1RootMissing { context: String },
    #[error("h1_root {value:?} is not 32 bytes of hex: {detail}")]
    H1Root { value: String, detail: String },
    #[error(
        "rooms {first:?} (h1 {first_h1}) and {second:?} (h1 {second_h1}) pin the same h1 nullifier-set \
         root, but their windows end at different heights; give each room its own h1_root"
    )]
    SharedH1Root {
        first: String,
        first_h1: u32,
        second: String,
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
    rooms: Vec<RoomFile>,
}

/// Reads and checks one gate config. Every refusal here happens before the socket is bound.
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
    })
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
