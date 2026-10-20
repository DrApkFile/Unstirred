#![doc = "One room of a gate: a manifest the verifier's own parser accepted, the JSON the gate will \
hand out for it, and the h1 nullifier-set root its operator pinned. Nothing is admitted into a \
room the gate could not read at startup."]

use std::path::Path;

use serde::Deserialize;
use unstirred_core::{parse_manifest, Manifest};

use super::ConfigError;

const ROOT_BYTES: usize = 32;

/// A room as the running gate holds it. `manifest_json` is what `GET manifest` answers: the
/// manifest re-serialised from the parsed value, so a holder can never be handed text this gate's
/// own parser would refuse.
#[derive(Debug, Clone)]
pub struct Room {
    pub manifest: Manifest,
    pub manifest_json: String,
    pub h1_root: [u8; ROOT_BYTES],
}

impl Room {
    pub fn context(&self) -> &str {
        &self.manifest.context
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct RoomFile {
    manifest: String,
    #[serde(default)]
    h1_root: Option<String>,
}

pub(super) fn resolve_rooms(
    files: &[RoomFile],
    gate_h1_root: Option<&str>,
    beside: &Path,
) -> Result<Vec<Room>, ConfigError> {
    if files.is_empty() {
        return Err(ConfigError::NoRooms);
    }
    let rooms = files
        .iter()
        .map(|file| resolve_room(file, gate_h1_root, beside))
        .collect::<Result<Vec<_>, _>>()?;
    require_distinct_contexts(&rooms)?;
    require_distinct_roots_per_height(&rooms)?;
    Ok(rooms)
}

fn resolve_room(file: &RoomFile, gate_h1_root: Option<&str>, beside: &Path) -> Result<Room, ConfigError> {
    let path = beside.join(&file.manifest);
    let json = std::fs::read_to_string(&path).map_err(|source| ConfigError::ManifestRead {
        path: path.clone(),
        source,
    })?;
    let manifest = parse_manifest(&json).map_err(|refusal| ConfigError::Manifest {
        path: path.clone(),
        refusal,
    })?;
    let pinned = file
        .h1_root
        .as_deref()
        .or(gate_h1_root)
        .ok_or_else(|| ConfigError::H1RootMissing {
            context: manifest.context.clone(),
        })?;
    let manifest_json = serve_as(&manifest, &path)?;
    Ok(Room {
        manifest,
        manifest_json,
        h1_root: root_from_hex(pinned)?,
    })
}

fn serve_as(manifest: &Manifest, path: &Path) -> Result<String, ConfigError> {
    serde_json::to_string_pretty(manifest).map_err(|e| ConfigError::Manifest {
        path: path.to_path_buf(),
        refusal: unstirred_core::Refusal::new(
            unstirred_core::Code::ManifestInvalid,
            format!("the manifest could not be re-serialised for publication: {e}"),
        ),
    })
}

fn root_from_hex(value: &str) -> Result<[u8; ROOT_BYTES], ConfigError> {
    let decoded = hex::decode(value.trim()).map_err(|e| ConfigError::H1Root {
        value: value.to_owned(),
        detail: format!("{e}"),
    })?;
    <[u8; ROOT_BYTES]>::try_from(decoded.as_slice()).map_err(|_| ConfigError::H1Root {
        value: value.to_owned(),
        detail: format!("it decodes to {} bytes", decoded.len()),
    })
}

fn require_distinct_contexts(rooms: &[Room]) -> Result<(), ConfigError> {
    for (index, room) in rooms.iter().enumerate() {
        if rooms[..index].iter().any(|earlier| earlier.context() == room.context()) {
            return Err(ConfigError::DuplicateContext {
                context: room.context().to_owned(),
            });
        }
    }
    Ok(())
}

/// A nullifier-set root belongs to a height. Rooms that share a window share a root; rooms whose
/// windows end at different heights cannot, and a gate that pinned one root for both would refuse
/// every honest proof at one of them with ROOT_MISMATCH.
fn require_distinct_roots_per_height(rooms: &[Room]) -> Result<(), ConfigError> {
    for (index, room) in rooms.iter().enumerate() {
        for earlier in &rooms[..index] {
            if earlier.h1_root == room.h1_root && earlier.manifest.window.h1 != room.manifest.window.h1 {
                return Err(ConfigError::SharedH1Root {
                    first: earlier.context().to_owned(),
                    first_h1: earlier.manifest.window.h1,
                    second: room.context().to_owned(),
                    second_h1: room.manifest.window.h1,
                });
            }
        }
    }
    Ok(())
}
