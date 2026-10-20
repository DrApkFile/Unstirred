#![doc = "Five routes and nothing else, matched by hand: a gate that cannot be asked for anything \
it does not host. The path builders are here too, so the client in this crate and the service \
cannot drift apart."]

use hyper::Method;

pub const ROOMS: &str = "/rooms";
pub const MANIFEST: &str = "manifest";
pub const CHALLENGE: &str = "challenge";
pub const ADMIT: &str = "admit";
pub const LOG: &str = "log";

/// What one request asks for. The context is whatever the path carried; whether the gate hosts a
/// room by that name is the dispatcher's question, not the router's.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Route {
    RoomList,
    Manifest(String),
    Challenge(String),
    Admit(String),
    Log(String),
}

/// Why a request matched no route: an unknown path, or a known one asked with the wrong method.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NoRoute {
    UnknownPath,
    WrongMethod { allowed: &'static str },
}

pub fn room_path(context: &str, leaf: &str) -> String {
    format!("{ROOMS}/{context}/{leaf}")
}

pub fn route(method: &Method, path: &str) -> Result<Route, NoRoute> {
    let trimmed = path.trim_end_matches('/');
    if trimmed == ROOMS || trimmed.is_empty() {
        return only(method, &Method::GET, Route::RoomList, "GET");
    }
    let rest = trimmed.strip_prefix(ROOMS).and_then(|rest| rest.strip_prefix('/'));
    let Some((context, leaf)) = rest.and_then(|rest| rest.split_once('/')) else {
        return Err(NoRoute::UnknownPath);
    };
    if context.is_empty() || leaf.contains('/') {
        return Err(NoRoute::UnknownPath);
    }
    let context = context.to_owned();
    match leaf {
        MANIFEST => only(method, &Method::GET, Route::Manifest(context), "GET"),
        LOG => only(method, &Method::GET, Route::Log(context), "GET"),
        CHALLENGE => only(method, &Method::POST, Route::Challenge(context), "POST"),
        ADMIT => only(method, &Method::POST, Route::Admit(context), "POST"),
        _ => Err(NoRoute::UnknownPath),
    }
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
    fn the_five_routes_match_and_nothing_else_does() {
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
}
