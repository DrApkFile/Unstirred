//! Writes every recorded act, and the room picker, as static HTML.
//!
//! A replay page fetches nothing — its own Content-Security-Policy forbids it to connect anywhere
//! — so the acts are a static site that needs no gate, no chain, no wallet and no nf-server. This
//! writes them with the gate's own `room_document` and `picker_document`, so an exported page is
//! the same bytes the gate serves at `/room/<context>?replay=<act>`; it is not a second rendering
//! of the same design. Both the demo video's browser scenes and any later published site read
//! these files, which is why there is one exporter rather than one per consumer.
//!
//! Run it through `scripts/export-replay-site.sh`, which names the output directory.

use std::fs;
use std::path::Path;

use unstirred_gate::page::{picker_document, replay_state, room_document, PageState};
use unstirred_gate::REPLAY_ACTS;

/// The act the picker is drawn from: the one recorded before anything had been admitted, so the
/// room cards show the rule and an empty log, which is what a visitor meets first.
const PICKER_ACT: &str = "act1-rule";

fn main() -> Result<(), String> {
    let mut arguments = std::env::args().skip(1);
    let out = arguments.next().ok_or("usage: export-replay-site <directory>")?;
    if arguments.next().is_some() {
        return Err("usage: export-replay-site <directory>".to_owned());
    }
    let out = Path::new(&out);
    fs::create_dir_all(out).map_err(|e| format!("{}: {e}", out.display()))?;

    let mut written = Vec::new();
    for (act, _) in REPLAY_ACTS {
        for present in [false, true] {
            let state = act_state(act, present)?;
            let name = file_name(act, present);
            write(out, &name, &room_document(&state)?)?;
            written.push(name);
        }
    }
    let mut picker = act_state(PICKER_ACT, false)?;
    picker.room = None;
    for room in &mut picker.rooms {
        room.page = Some(first_act_of(&room.context)?);
    }
    write(out, "index.html", &picker_document(&picker)?)?;
    written.push("index.html".to_owned());

    for name in &written {
        println!("{}", out.join(name).display());
    }
    println!(
        "{} page(s) written; each one renders from the state it carries and fetches nothing",
        written.len()
    );
    Ok(())
}

/// Where an exported picker's room card opens: the first recorded act of that room. A gate route
/// has no file behind it in a static export, so a published picker that kept `/room/<context>`
/// would hand a visitor a 404 (A51).
fn first_act_of(context: &str) -> Result<String, String> {
    for (act, _) in REPLAY_ACTS {
        let state = act_state(act, false)?;
        if state.room.as_deref() == Some(context) {
            return Ok(file_name(act, false));
        }
    }
    Err(format!("no recorded act is about room {context:?}"))
}

fn act_state(act: &str, present: bool) -> Result<PageState, String> {
    let mut state = replay_state(act)
        .ok_or_else(|| format!("no act is named {act:?}"))?
        .map_err(|detail| format!("{act} cannot be rendered: {detail}"))?;
    state.present = present;
    Ok(state)
}

/// `act2-admitted.html` beside `act2-admitted.present.html`, so the two readings of one act sort
/// together and a URL says which it is.
fn file_name(act: &str, present: bool) -> String {
    if present {
        format!("{act}.present.html")
    } else {
        format!("{act}.html")
    }
}

fn write(out: &Path, name: &str, document: &str) -> Result<(), String> {
    let path = out.join(name);
    fs::write(&path, document).map_err(|e| format!("{}: {e}", path.display()))
}
