#![doc = "The one thing a gate remembers that its database does not: the verdicts it has just \
answered, per room, in memory only. A refusal writes nothing to the store by design, so without \
this the Holders' Room page could never show one; with it, a refusal is visible for as long as the \
process lives and no longer. Each entry is a verdict, its code and the second it was answered — no \
marker, no challenge, no package, no address — it is bounded per room, and it is never written to \
disk, so restarting the gate forgets every one (LEAKS L16)."]

use std::collections::{BTreeMap, VecDeque};
use std::sync::Mutex;

use unstirred_verifier::Verdict;

/// How many verdicts one room remembers. Old entries fall off the end rather than accumulating:
/// the page shows the latest verdict and a short tail, and nothing here is a record of anything.
pub const RECENT_PER_ROOM: usize = 20;

/// One answered verdict, as the feed keeps it. These three fields are the whole of it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RecentVerdict {
    pub verdict: &'static str,
    pub code: Option<&'static str>,
    pub at: i64,
}

impl RecentVerdict {
    pub fn of(verdict: &Verdict, at: i64) -> Self {
        Self {
            verdict: name_of(verdict),
            code: crate::reply::code_of(verdict),
            at,
        }
    }
}

/// The feed itself: newest first per room, bounded, and gone when the process ends.
#[derive(Debug, Default)]
pub struct RecentVerdicts {
    per_room: Mutex<BTreeMap<String, VecDeque<RecentVerdict>>>,
}

impl RecentVerdicts {
    pub fn record(&self, context: &str, verdict: &Verdict, at: i64) {
        let entry = RecentVerdict::of(verdict, at);
        let mut per_room = match self.per_room.lock() {
            Ok(per_room) => per_room,
            Err(poisoned) => poisoned.into_inner(),
        };
        let room = per_room.entry(context.to_owned()).or_default();
        room.push_front(entry);
        room.truncate(RECENT_PER_ROOM);
    }

    /// This room's verdicts, newest first. A room nothing has knocked on answers with none.
    pub fn newest(&self, context: &str) -> Vec<RecentVerdict> {
        let per_room = match self.per_room.lock() {
            Ok(per_room) => per_room,
            Err(poisoned) => poisoned.into_inner(),
        };
        per_room
            .get(context)
            .map(|room| room.iter().copied().collect())
            .unwrap_or_default()
    }
}

pub fn name_of(verdict: &Verdict) -> &'static str {
    match verdict {
        Verdict::Admitted => "ADMITTED",
        Verdict::Refused(_) => "REFUSED",
        Verdict::Inconclusive(_) => "INCONCLUSIVE",
    }
}
