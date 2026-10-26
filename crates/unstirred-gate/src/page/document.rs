#![doc = "Assembling one page: a template, the design tokens, the renderer, the live poller when \
the page is live, and the state as JSON. The result is one self-contained HTML document with \
nothing to fetch but this gate's own JSON routes — and in replay mode not even that, because the \
poller is left out entirely, so a replayed act has no network call site at all (LEAKS L10)."]

use crate::page::contract::{Mode, PageState};

const PICKER_TEMPLATE: &str = include_str!("../../ui/picker.html");
const ROOM_TEMPLATE: &str = include_str!("../../ui/room.html");
const TOKENS: &str = include_str!("../../ui/tokens.css");
const RENDERER: &str = include_str!("../../ui/render.js");
const POLLER: &str = include_str!("../../ui/live.js");
/// The page's icon, as the base64 of a 32x32 PNG cut from `assets/icon.png` by
/// `scripts/make-favicon.sh`. It is carried in the document as a `data:` URI rather than fetched:
/// bytes already in the page are not a request to an origin, which is what keeps a replayed act
/// openable from a file with nothing behind it (A02c, LEAKS L10).
const FAVICON: &str = include_str!("../../ui/favicon.b64");

const TOKENS_SLOT: &str = "/*TOKENS*/";
const RENDERER_SLOT: &str = "/*RENDERER*/";
const POLLER_SLOT: &str = "/*POLLER*/";
const FAVICON_SLOT: &str = "/*FAVICON*/";
const CONNECT_SLOT: &str = "/*CONNECT*/";
const STATE_SLOT: &str = "/*STATE*/";

/// The room picker, as one self-contained document. The state is checked first, so a state the
/// renderer would mislead a viewer with is never served.
pub fn picker_document(state: &PageState) -> Result<String, String> {
    document(PICKER_TEMPLATE, state)
}

/// One room's page, as one self-contained document. In replay mode the live poller is left out
/// of the document entirely, so a replayed act has no network call site at all.
pub fn room_document(state: &PageState) -> Result<String, String> {
    document(ROOM_TEMPLATE, state)
}

fn document(template: &str, state: &PageState) -> Result<String, String> {
    state.check()?;
    let json = serde_json::to_string(state).map_err(|e| format!("{e}"))?;
    // A replay document is assembled without the poller at all, and its own Content-Security-Policy
    // forbids the browser to connect anywhere: an act renders from the state it carries or not at all.
    let (poller, connect) = match state.mode {
        Mode::Live => (format!("<script>\n{POLLER}\n</script>"), "'self'"),
        Mode::Replay => (String::new(), "'none'"),
    };
    let assembled = template
        .replace(TOKENS_SLOT, TOKENS)
        .replace(RENDERER_SLOT, RENDERER)
        .replace(POLLER_SLOT, &poller)
        .replace(CONNECT_SLOT, connect)
        .replace(FAVICON_SLOT, &FAVICON.replace('\n', ""));
    if !assembled.contains(STATE_SLOT) {
        return Err("the page template has no state slot".to_owned());
    }
    Ok(assembled.replace(STATE_SLOT, &inert(&json)))
}

/// JSON that cannot end the script element that carries it, whatever a message or a provenance
/// line happens to contain. These three escapes are valid JSON and are read back by `JSON.parse`
/// unchanged.
fn inert(json: &str) -> String {
    json.replace('<', "\\u003c")
        .replace('>', "\\u003e")
        .replace('&', "\\u0026")
}

#[cfg(test)]
mod tests {
    use super::TOKENS;

    const FRAME: &str = include_str!("../../ui/frame.md");

    /// `ui/frame.md` is the design system the demo video is cut against, and it is a mirror of the
    /// stylesheet rather than a second source. A token added or changed in one and not the other
    /// is the way a design drifts in silence, so it fails here.
    #[test]
    fn every_design_token_appears_in_the_video_design_system() {
        let mut missing = Vec::new();
        for line in TOKENS.lines() {
            let line = line.trim();
            if !line.starts_with("--") {
                continue;
            }
            let Some((name, value)) = line.trim_end_matches(';').split_once(':') else {
                continue;
            };
            let row = format!("| `{}` | ", name.trim());
            let cell = format!("`{}`", value.trim());
            let found = FRAME
                .lines()
                .find(|candidate| candidate.starts_with(&row))
                .filter(|candidate| candidate.contains(&cell));
            if found.is_none() {
                missing.push(format!("{} = {}", name.trim(), value.trim()));
            }
        }
        assert_eq!(
            missing,
            Vec::<String>::new(),
            "ui/frame.md does not mirror ui/tokens.css"
        );
    }
}
