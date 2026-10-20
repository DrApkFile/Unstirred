//! A gate that will not start. Every one of these is a refusal before the socket is bound: an
//! operator finds out from the command, not from a holder whose honest proof was refused.

use std::error::Error;
use std::path::Path;

use unstirred_gate::{load_config, ConfigError};

type TestResult = Result<(), Box<dyn Error>>;

const H1_ROOT: &str = "8af0c16b82ff1c9170de5dc87b3c06ef3a7144befdba71fc40959effe9f5be26";
const OTHER_ROOT: &str = "a16809847bf51d60ac6d9bb3f71cbbf27483050868b94b242ca831fe14bf692d";

/// A41: the config refusals, each one named. A gate starts only when it can read every room it
/// hosts, knows where to write, has the root sources its own rule requires, and has one pinned h1
/// root per window.
#[test]
fn a41_a_gate_refuses_to_start_on_an_invalid_config() -> TestResult {
    let directory = tempfile::tempdir()?;
    let home = directory.path();
    write_manifest(home, "holders-room", 4_505_790, 4_506_510)?;
    write_manifest(home, "later-room", 4_505_790, 4_506_520)?;
    std::fs::write(home.join("broken.json"), "{\"version\":\"unstirred/v2\"}")?;

    let good = config(
        "127.0.0.1:8080",
        &["lwd:https://a.invalid:443", "zebra:http://b.invalid:18232"],
        false,
        Some(H1_ROOT),
        "[[rooms]]\nmanifest = \"holders-room.json\"\n",
    );
    let loaded = load_config(written(home, "good.toml", &good)?)?;
    assert_eq!(loaded.rooms.len(), 1);
    assert_eq!(loaded.rooms[0].context(), "holders-room");
    assert_eq!(
        loaded.database,
        home.join("gate.sqlite"),
        "paths follow the config file"
    );

    match refusal(
        home,
        "no-rooms.toml",
        &config(
            "127.0.0.1:8080",
            &["lwd:https://a.invalid:443", "zebra:http://b.invalid:18232"],
            false,
            Some(H1_ROOT),
            "",
        ),
    )? {
        ConfigError::NoRooms => {}
        other => return Err(format!("expected NoRooms, got {other}").into()),
    }
    match refusal(
        home,
        "one-source.toml",
        &config(
            "127.0.0.1:8080",
            &["lwd:https://a.invalid:443"],
            false,
            Some(H1_ROOT),
            "[[rooms]]\nmanifest = \"holders-room.json\"\n",
        ),
    )? {
        ConfigError::SourceCount { .. } => {}
        other => return Err(format!("expected SourceCount, got {other}").into()),
    }
    match refusal(
        home,
        "two-single.toml",
        &config(
            "127.0.0.1:8080",
            &["lwd:https://a.invalid:443", "zebra:http://b.invalid:18232"],
            true,
            Some(H1_ROOT),
            "[[rooms]]\nmanifest = \"holders-room.json\"\n",
        ),
    )? {
        ConfigError::SourceCount { .. } => {}
        other => return Err(format!("expected SourceCount, got {other}").into()),
    }
    match refusal(
        home,
        "bad-source.toml",
        &config(
            "127.0.0.1:8080",
            &["lwd:https://a.invalid:443", "postgres://nope"],
            false,
            Some(H1_ROOT),
            "[[rooms]]\nmanifest = \"holders-room.json\"\n",
        ),
    )? {
        ConfigError::SourceSpec { .. } => {}
        other => return Err(format!("expected SourceSpec, got {other}").into()),
    }
    match refusal(
        home,
        "bad-listen.toml",
        &config(
            "not-an-address",
            &["lwd:https://a.invalid:443", "zebra:http://b.invalid:18232"],
            false,
            Some(H1_ROOT),
            "[[rooms]]\nmanifest = \"holders-room.json\"\n",
        ),
    )? {
        ConfigError::Listen { .. } => {}
        other => return Err(format!("expected Listen, got {other}").into()),
    }
    match refusal(
        home,
        "no-root.toml",
        &config(
            "127.0.0.1:8080",
            &["lwd:https://a.invalid:443", "zebra:http://b.invalid:18232"],
            false,
            None,
            "[[rooms]]\nmanifest = \"holders-room.json\"\n",
        ),
    )? {
        ConfigError::H1RootMissing { .. } => {}
        other => return Err(format!("expected H1RootMissing, got {other}").into()),
    }
    match refusal(
        home,
        "short-root.toml",
        &config(
            "127.0.0.1:8080",
            &["lwd:https://a.invalid:443", "zebra:http://b.invalid:18232"],
            false,
            Some("8af0c1"),
            "[[rooms]]\nmanifest = \"holders-room.json\"\n",
        ),
    )? {
        ConfigError::H1Root { .. } => {}
        other => return Err(format!("expected H1Root, got {other}").into()),
    }
    match refusal(
        home,
        "broken-manifest.toml",
        &config(
            "127.0.0.1:8080",
            &["lwd:https://a.invalid:443", "zebra:http://b.invalid:18232"],
            false,
            Some(H1_ROOT),
            "[[rooms]]\nmanifest = \"broken.json\"\n",
        ),
    )? {
        ConfigError::Manifest { .. } => {}
        other => return Err(format!("expected Manifest, got {other}").into()),
    }
    match refusal(
        home,
        "missing-manifest.toml",
        &config(
            "127.0.0.1:8080",
            &["lwd:https://a.invalid:443", "zebra:http://b.invalid:18232"],
            false,
            Some(H1_ROOT),
            "[[rooms]]\nmanifest = \"absent.json\"\n",
        ),
    )? {
        ConfigError::ManifestRead { .. } => {}
        other => return Err(format!("expected ManifestRead, got {other}").into()),
    }
    let twice = "[[rooms]]\nmanifest = \"holders-room.json\"\n\n[[rooms]]\nmanifest = \"holders-room.json\"\n";
    match refusal(
        home,
        "duplicate.toml",
        &config(
            "127.0.0.1:8080",
            &["lwd:https://a.invalid:443", "zebra:http://b.invalid:18232"],
            false,
            Some(H1_ROOT),
            twice,
        ),
    )? {
        ConfigError::DuplicateContext { context } => assert_eq!(context, "holders-room"),
        other => return Err(format!("expected DuplicateContext, got {other}").into()),
    }
    let two_windows = "[[rooms]]\nmanifest = \"holders-room.json\"\n\n[[rooms]]\nmanifest = \"later-room.json\"\n";
    match refusal(
        home,
        "shared-root.toml",
        &config(
            "127.0.0.1:8080",
            &["lwd:https://a.invalid:443", "zebra:http://b.invalid:18232"],
            false,
            Some(H1_ROOT),
            two_windows,
        ),
    )? {
        ConfigError::SharedH1Root { .. } => {}
        other => return Err(format!("expected SharedH1Root, got {other}").into()),
    }
    let overridden = format!(
        "[[rooms]]\nmanifest = \"holders-room.json\"\n\n[[rooms]]\nmanifest = \"later-room.json\"\nh1_root = \"{OTHER_ROOT}\"\n"
    );
    let loaded = load_config(written(
        home,
        "per-room-root.toml",
        &config(
            "127.0.0.1:8080",
            &["lwd:https://a.invalid:443", "zebra:http://b.invalid:18232"],
            false,
            Some(H1_ROOT),
            &overridden,
        ),
    )?)?;
    assert_eq!(loaded.rooms.len(), 2, "a room may pin its own h1 root");
    assert_ne!(loaded.rooms[0].h1_root, loaded.rooms[1].h1_root);

    let unknown_setting = format!("{good}\nauthority = \"trust me\"\n");
    match refusal(home, "unknown.toml", &unknown_setting)? {
        ConfigError::Toml { .. } => {}
        other => return Err(format!("expected Toml, got {other}").into()),
    }
    Ok(())
}

fn config(listen: &str, sources: &[&str], single: bool, h1_root: Option<&str>, rooms: &str) -> String {
    let quoted: Vec<String> = sources.iter().map(|source| format!("\"{source}\"")).collect();
    let mut text = format!(
        "listen = \"{listen}\"\ndatabase = \"gate.sqlite\"\nroot_sources = [{}]\n",
        quoted.join(", ")
    );
    if single {
        text.push_str("single_root_source = true\n");
    }
    if let Some(root) = h1_root {
        text.push_str(&format!("h1_root = \"{root}\"\n"));
    }
    text.push('\n');
    text.push_str(rooms);
    text
}

fn write_manifest(home: &Path, context: &str, h0: u32, h1: u32) -> TestResult {
    let json = unstirred_core::build_manifest(
        unstirred_core::Network::Test,
        context,
        unstirred_core::Window { h0, h1 },
    )?;
    std::fs::write(home.join(format!("{context}.json")), json)?;
    Ok(())
}

fn written(home: &Path, name: &str, text: &str) -> Result<std::path::PathBuf, Box<dyn Error>> {
    let path = home.join(name);
    std::fs::write(&path, text)?;
    Ok(path)
}

fn refusal(home: &Path, name: &str, text: &str) -> Result<ConfigError, Box<dyn Error>> {
    match load_config(written(home, name, text)?) {
        Err(problem) => Ok(problem),
        Ok(config) => Err(format!("{name} started a gate it should have refused: {config:?}").into()),
    }
}
