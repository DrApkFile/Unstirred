#![doc = "The one route that decides anything. It consults this gate's own root authority, opens \
its own database connection and calls the same `unstirred_verifier::admit` the `unstirred admit` \
command calls: the service adds no check of its own and skips none, and the body it answers with \
is the verdict object that command prints. Nothing in the package tells the gate which roots, \
heights or context to use."]

use unstirred_verifier::store::{Store, StoreError};
use unstirred_verifier::{admit as admit_package, as_verdict, verdict_object, PinnedRoots, Verdict};

use crate::handlers::blocking;
use crate::reply::{code_of, error, json, status_for, Reply};
use crate::state::GateState;

/// Admits one package into one room, or refuses it. The package body is never logged and never
/// stored; a body that is not a package is refused by the engine as PROOF_INVALID.
pub(super) async fn admit(state: &GateState, context: &str, body: Vec<u8>) -> (Reply, Option<String>) {
    let note = state.note();
    let Some(room) = state.room(context).cloned() else {
        return (
            error(
                hyper::StatusCode::NOT_FOUND,
                "this gate hosts no room by that name",
                note,
            ),
            None,
        );
    };
    let Ok(package_json) = String::from_utf8(body) else {
        return (
            error(
                hyper::StatusCode::BAD_REQUEST,
                "the request body is not UTF-8, so it is not a proof package; nothing was stored",
                note,
            ),
            None,
        );
    };
    let database = state.database.clone();
    let roots = state.roots.clone();
    let clock = state.clock.clone();
    let log = state.log.clone();
    let decided = blocking(move || {
        let readout = roots.consult(room.manifest.network, room.manifest.window.h0);
        log.line(&provenance(&readout, room.manifest.window.h0));
        let agreed = match readout.agreement {
            Ok(agreed) => agreed,
            Err(refusal) => return Ok(as_verdict(refusal)),
        };
        let mut store = Store::open(&database)?;
        admit_package(
            &room.manifest,
            &package_json,
            &PinnedRoots {
                nc_root_at_h0: agreed.nc_root_at_h0,
                nf_imt_root_at_h1: room.h1_root,
            },
            agreed.tip,
            &mut store,
            &*clock,
        )
    })
    .await;
    match decided {
        Ok(Ok(verdict)) => {
            // The store writes nothing for a refusal, so the room page would never see one. The
            // feed remembers the verdict, its code and this second, in memory only (LEAKS L16).
            state.recent.record(context, &verdict, state.clock.unix_seconds());
            answer(&verdict, note)
        }
        Ok(Err(fault)) => (database_fault(&fault, note), None),
        Err(detail) => (
            error(
                hyper::StatusCode::SERVICE_UNAVAILABLE,
                &format!("this gate could not decide: {detail}"),
                note,
            ),
            None,
        ),
    }
}

fn answer(verdict: &Verdict, note: Option<&str>) -> (Reply, Option<String>) {
    let code = code_of(verdict).map(str::to_owned);
    (json(status_for(verdict), verdict_object(verdict, note), note), code)
}

/// A database fault is not a verdict: the gate says it decided nothing rather than refusing a
/// holder for its own broken file.
fn database_fault(fault: &StoreError, note: Option<&str>) -> Reply {
    error(
        hyper::StatusCode::SERVICE_UNAVAILABLE,
        &format!("this gate's own database failed, so nothing was decided: {fault}"),
        note,
    )
}

/// The provenance of the roots this admission was decided against, in the gate's log: the same
/// facts `unstirred roots` prints, naming only the sources the operator configured.
fn provenance(readout: &unstirred_verifier::RootAuthorityReadout, h0: u32) -> String {
    let mut line = format!("roots at h0 {h0}:");
    for outcome in &readout.per_source {
        match &outcome.answer {
            Ok(answer) => line.push_str(&format!(
                " [{} root {} tip {}]",
                outcome.source,
                hex::encode(answer.nc_root_at_h0),
                answer.tip
            )),
            Err(refusal) => line.push_str(&format!(" [{} unusable: {}]", outcome.source, refusal.code)),
        }
    }
    match &readout.agreement {
        Ok(agreed) => line.push_str(&format!(
            " -> pinned {} tip {}",
            hex::encode(agreed.nc_root_at_h0),
            agreed.tip
        )),
        Err(refusal) => line.push_str(&format!(" -> {}", refusal.code)),
    }
    line
}
