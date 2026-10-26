#![doc = "Every route and nothing else, matched by hand: a gate that cannot be asked for anything \
it does not host. The five JSON routes a holder uses, the recent-verdict feed the page polls, and \
the two pages the gate serves itself. The path builders are here too, so the client in this crate \
and the service cannot drift apart."]

use hyper::Method;

/// The room list, and the prefix every room's own routes sit under.
pub const ROOMS: &str = "/rooms";
/// The prefix for the page a gate serves for one room.
pub const ROOM_PAGE: &str = "/room";
/// The leaf that serves a room's published rule.
pub const MANIFEST: &str = "manifest";
/// The leaf a holder posts to for a challenge.
pub const CHALLENGE: &str = "challenge";
/// The leaf a holder posts a package to.
pub const ADMIT: &str = "admit";
/// The leaf that serves a room's committed admissions and markers.
pub const LOG: &str = "log";
/// The leaf the page polls for verdicts just answered, including refusals, which are never
/// written.
pub const RECENT: &str = "recent";

/// What one request asks for. The context is whatever the path carried; whether the gate hosts a
/// room by that name is the dispatcher's question, not the router's.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Route {
    /// `GET /` — the room picker page.
    Picker,
    /// `GET /room/<context>` — one room's page.
    RoomPage(String),
    /// `GET /rooms` — the room list, with any advertised services.
    RoomList,
    /// `GET /rooms/<context>/manifest` — the room's published rule.
    Manifest(String),
    /// `POST /rooms/<context>/challenge` — issue a challenge.
    Challenge(String),
    /// `POST /rooms/<context>/admit` — answer one with a package.
    Admit(String),
    /// `GET /rooms/<context>/log` — the committed admissions and markers.
    Log(String),
    /// `GET /rooms/<context>/recent` — the verdicts just answered.
    Recent(String),
}

/// Why a request matched no route: an unknown path, or a known one asked with the wrong method.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NoRoute {
    /// No route has this path, whatever the method.
    UnknownPath,
    /// The path is a route, but not with this method.
    WrongMethod {
        /// The method that would have worked, for the `Allow` header.
        allowed: &'static str,
    },
}

/// One of a room's JSON paths. The client in this crate builds its requests with this, so a
/// renamed leaf cannot leave the client asking for the old one.
///
/// ```
/// use unstirred_gate::router::{room_path, ADMIT};
/// assert_eq!(room_path("holders-room", ADMIT), "/rooms/holders-room/admit");
/// ```
pub fn room_path(context: &str, leaf: &str) -> String {
    format!("{ROOMS}/{context}/{leaf}")
}

/// The page path for one room.
pub fn room_page_path(context: &str) -> String {
    format!("{ROOM_PAGE}/{context}")
}

/// Matches one request. Nothing is matched by pattern or prefix: a path the gate does not host is
/// an unknown path, and a known path asked with the wrong method says which method to use.
///
/// ```
/// use hyper::Method;
/// use unstirred_gate::router::{route, NoRoute, Route};
///
/// assert_eq!(route(&Method::GET, "/rooms"), Ok(Route::RoomList));
/// assert_eq!(
///     route(&Method::POST, "/rooms/holders-room/admit"),
///     Ok(Route::Admit("holders-room".to_owned())),
/// );
///
/// // A challenge is a POST, and asking for it any other way says so.
/// assert_eq!(
///     route(&Method::GET, "/rooms/holders-room/challenge"),
///     Err(NoRoute::WrongMethod { allowed: "POST" }),
/// );
/// assert_eq!(route(&Method::GET, "/admin"), Err(NoRoute::UnknownPath));
/// ```
pub fn route(method: &Method, path: &str) -> Result<Route, NoRoute> {
    let trimmed = path.trim_end_matches('/');
    if trimmed.is_empty() {
        return only(method, &Method::GET, Route::Picker, "GET");
    }
    if trimmed == ROOMS {
        return only(method, &Method::GET, Route::RoomList, "GET");
    }
    if let Some(context) = single_segment(trimmed, ROOM_PAGE) {
        return only(method, &Method::GET, Route::RoomPage(context), "GET");
    }
    let Some((context, leaf)) = room_leaf(trimmed) else {
        return Err(NoRoute::UnknownPath);
    };
    match leaf {
        MANIFEST => only(method, &Method::GET, Route::Manifest(context), "GET"),
        LOG => only(method, &Method::GET, Route::Log(context), "GET"),
        RECENT => only(method, &Method::GET, Route::Recent(context), "GET"),
        CHALLENGE => only(method, &Method::POST, Route::Challenge(context), "POST"),
        ADMIT => only(method, &Method::POST, Route::Admit(context), "POST"),
        _ => Err(NoRoute::UnknownPath),
    }
}

/// The one path segment under `prefix`, when that is the whole of the rest of the path.
fn single_segment(trimmed: &str, prefix: &str) -> Option<String> {
    let rest = trimmed.strip_prefix(prefix)?.strip_prefix('/')?;
    (!rest.is_empty() && !rest.contains('/')).then(|| rest.to_owned())
}

fn room_leaf(trimmed: &str) -> Option<(String, &str)> {
    let rest = trimmed.strip_prefix(ROOMS)?.strip_prefix('/')?;
    let (context, leaf) = rest.split_once('/')?;
    (!context.is_empty() && !leaf.is_empty() && !leaf.contains('/')).then(|| (context.to_owned(), leaf))
}

fn only(asked: &Method, required: &Method, route: Route, allowed: &'static str) -> Result<Route, NoRoute> {
    if asked == required {
        Ok(route)
    } else {
        Err(NoRoute::WrongMethod { allowed })
    }
}

#[cfg(test)]
mod tests {
    use super::{route, NoRoute, Route, ADMIT, CHALLENGE};
    use hyper::Method;

    #[test]
    fn the_json_routes_match_and_nothing_else_does() {
        assert_eq!(route(&Method::GET, "/rooms"), Ok(Route::RoomList));
        assert_eq!(route(&Method::GET, "/rooms/"), Ok(Route::RoomList));
        assert_eq!(
            route(&Method::POST, "/rooms/holders-room/admit"),
            Ok(Route::Admit("holders-room".to_owned()))
        );
        assert_eq!(
            route(&Method::GET, "/rooms/holders-room/manifest"),
            Ok(Route::Manifest("holders-room".to_owned()))
        );
        assert_eq!(
            route(&Method::GET, "/rooms/holders-room/log"),
            Ok(Route::Log("holders-room".to_owned()))
        );
        assert_eq!(
            route(&Method::GET, "/rooms/holders-room/recent"),
            Ok(Route::Recent("holders-room".to_owned()))
        );
        assert_eq!(
            route(&Method::POST, "/rooms/holders-room/challenge"),
            Ok(Route::Challenge("holders-room".to_owned()))
        );
        assert_eq!(
            route(&Method::GET, &format!("/rooms/holders-room/{ADMIT}")),
            Err(NoRoute::WrongMethod { allowed: "POST" })
        );
        assert_eq!(
            route(&Method::GET, &format!("/rooms/holders-room/{CHALLENGE}")),
            Err(NoRoute::WrongMethod { allowed: "POST" })
        );
        assert_eq!(route(&Method::GET, "/rooms/holders-room"), Err(NoRoute::UnknownPath));
        assert_eq!(
            route(&Method::GET, "/rooms/holders-room/markers"),
            Err(NoRoute::UnknownPath)
        );
        assert_eq!(
            route(&Method::GET, "/rooms/holders-room/log/extra"),
            Err(NoRoute::UnknownPath)
        );
        assert_eq!(route(&Method::GET, "/gate.sqlite"), Err(NoRoute::UnknownPath));
    }

    /// The two pages are their own routes: the picker at the root, and one page per room under a
    /// prefix of its own, so no page path can ever be mistaken for the room list's JSON.
    #[test]
    fn the_pages_are_their_own_routes() {
        assert_eq!(route(&Method::GET, "/"), Ok(Route::Picker));
        assert_eq!(route(&Method::GET, ""), Ok(Route::Picker));
        assert_eq!(route(&Method::POST, "/"), Err(NoRoute::WrongMethod { allowed: "GET" }));
        assert_eq!(
            route(&Method::GET, "/room/holders-room"),
            Ok(Route::RoomPage("holders-room".to_owned()))
        );
        assert_eq!(
            route(&Method::GET, "/room/holders-room/"),
            Ok(Route::RoomPage("holders-room".to_owned()))
        );
        assert_eq!(route(&Method::GET, "/room"), Err(NoRoute::UnknownPath));
        assert_eq!(route(&Method::GET, "/room/"), Err(NoRoute::UnknownPath));
        assert_eq!(route(&Method::GET, "/room/holders-room/log"), Err(NoRoute::UnknownPath));
    }
}
