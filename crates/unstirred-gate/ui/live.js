/* The only part of the page that touches the network, and the only part a replayed act does not
   carry at all: a replay document is assembled without this file, so it has no network call site
   to disable (LEAKS L10).

   It reads two of this gate's own JSON routes per room, by relative path, and nothing else. No
   third-party request, no beacon, no analytics, no storage. What it fetches is exactly what the
   room log and the recent-verdict feed already publish to anyone. */

(function (global) {
  "use strict";

  var EVERY_MS = 2000;
  var MEMBERS_INPUT = "members";

  function readJson(path) {
    return fetch(path, { headers: { accept: "application/json" }, cache: "no-store" }).then(function (answer) {
      if (!answer.ok) {
        throw new Error(path + " answered " + answer.status);
      }
      return answer.json();
    });
  }

  function refreshRoom(room) {
    var context = encodeURIComponent(room.context);
    return Promise.all([
      readJson("/rooms/" + context + "/log"),
      readJson("/rooms/" + context + "/recent")
    ]).then(function (answers) {
      room.markers = answers[0].markers;
      room.verdicts = answers[0].verdicts;
      room.limit = answers[0].limit;
      room.recent = answers[1].verdicts;
    });
  }

  function refresh() {
    var page = global.Unstirred;
    if (!page) {
      return;
    }
    var state = page.state;
    var rooms = [];
    for (var i = 0; i < state.rooms.length; i += 1) {
      rooms.push(refreshRoom(state.rooms[i]));
    }
    Promise.all(rooms)
      .then(function () {
        var focused = document.activeElement;
        var keepFocus = focused && focused.id === MEMBERS_INPUT ? MEMBERS_INPUT : null;
        page.mount(state, keepFocus);
      })
      .catch(function () {
        /* A gate that did not answer this round leaves the last answer on screen. The page never
           invents a verdict, and never shows one it was not given. */
      });
  }

  global.setInterval(refresh, EVERY_MS);
  refresh();
})(window);
