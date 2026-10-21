//! Reading a page the gate served, the way a browser would read it: which of its parts are code
//! and which is the state it carries, and whether anything in the code could ask for something
//! from somewhere else. The state block is excluded from the code scans on purpose — a root
//! source the gate *displays* is text on a page, not a request, and the two must not be confused.

use std::error::Error;

const STATE_CARRIER: &str = " type=\"application/json\" id=\"unstirred-state\"";

/// Every way an HTML document can ask a browser for a subresource, open a socket or phone home.
/// None of these may appear anywhere in a page this gate serves (A02c, LEAKS L10).
const OFF_ORIGIN: [&str; 17] = [
    "<script src",
    "<link",
    "<img",
    "<image",
    "<iframe",
    "<frame",
    "<object",
    "<embed",
    "<video",
    "<audio",
    "<track",
    "<base",
    "@import",
    "url(",
    "srcset",
    "integrity=",
    "crossorigin",
];

/// Every way the page could reach the network. A replayed act must contain none of them.
const NETWORK: [&str; 8] = [
    "fetch(",
    "XMLHttpRequest",
    "WebSocket",
    "EventSource",
    "sendBeacon",
    "importScripts",
    "setInterval(",
    "setTimeout(",
];

/// The document's script bodies and the state block, separated. `code` is everything a browser
/// would execute or apply; `state` is the JSON the page renders from.
pub struct Parts {
    pub code: String,
    pub state: String,
}

pub fn parts(document: &str) -> Result<Parts, Box<dyn Error>> {
    let mut state = String::new();
    let mut code = String::new();
    for (index, chunk) in document.split("<script").enumerate() {
        if index == 0 {
            code.push_str(chunk);
        } else if chunk.starts_with(STATE_CARRIER) {
            let body = chunk
                .split_once('>')
                .ok_or("the state block has no end of tag")?
                .1
                .split_once("</script>")
                .ok_or("the state block is never closed")?
                .0;
            state.push_str(body);
        } else {
            code.push_str(chunk);
        }
    }
    if state.trim().is_empty() {
        return Err("the document carries no state block".into());
    }
    Ok(Parts { code, state })
}

/// Anything in the document that would make a browser ask another origin for something.
pub fn off_origin_markers(document: &str) -> Vec<&'static str> {
    let lowered = document.to_lowercase();
    OFF_ORIGIN
        .into_iter()
        .filter(|marker| lowered.contains(&marker.to_lowercase()))
        .collect()
}

/// Absolute or protocol-relative URLs written into the page's own code. The state block is not
/// code: the gate names its root sources there so a judge can read them.
pub fn absolute_urls_in_code(code: &str) -> Vec<String> {
    let mut found = Vec::new();
    for opener in ["\"http", "'http", "\"//", "'//"] {
        if let Some(at) = code.find(opener) {
            found.push(code[at..code.len().min(at + 48)].to_owned());
        }
    }
    found
}

/// Every network call site in the page's code.
pub fn network_call_sites(code: &str) -> Vec<&'static str> {
    NETWORK.into_iter().filter(|marker| code.contains(marker)).collect()
}
