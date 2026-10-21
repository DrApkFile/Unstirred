/* One pure function of one state, and the glue that mounts it.

   render(state) builds the whole page and reads nothing but `state`: no clock, no randomness, no
   network, no browser storage. Every number on screen is computed from the state the gate handed
   this page, so the live page and a replayed act differ only in where that state came from, and
   the demo video can be rendered from the same document a judge opens.

   The state's shape is the contract in src/page/contract.rs. */

(function (global) {
  "use strict";

  var ZATOSHIS_PER_ZEC = 100000000;
  var MARKER_PREFIX_BYTES = 6;
  var MARKERS_SHOWN = 4;
  var MEMBERS_MAX = 1000000;
  var STAGE_W = 1920;
  var STAGE_H = 1080;
  var MEMBERS_INPUT = "members";

  function node(tag, className, content) {
    var element = document.createElement(tag);
    if (className) {
      element.className = className;
    }
    if (content !== undefined && content !== null) {
      element.textContent = String(content);
    }
    return element;
  }

  function filled(parent, children) {
    for (var i = 0; i < children.length; i += 1) {
      if (children[i]) {
        parent.appendChild(children[i]);
      }
    }
    return parent;
  }

  function panel(className, children) {
    return filled(node("section", className ? "panel " + className : "panel"), children);
  }

  function list(className, items) {
    var ul = node("ul", className);
    /* A list styled with list-style:none loses its list semantics in some screen readers. */
    ul.setAttribute("role", "list");
    for (var i = 0; i < items.length; i += 1) {
      ul.appendChild(node("li", null, items[i]));
    }
    return ul;
  }

  function digits(value) {
    var all = String(value);
    var out = "";
    var seen = 0;
    for (var i = all.length - 1; i >= 0; i -= 1) {
      out = all.charAt(i) + out;
      seen += 1;
      if (seen % 3 === 0 && i > 0) {
        out = "," + out;
      }
    }
    return out;
  }

  /* Zatoshis to ZEC, exactly: integer arithmetic, trailing zeros dropped, never rounded up. */
  function zec(zatoshis) {
    var whole = Math.floor(zatoshis / ZATOSHIS_PER_ZEC);
    var fraction = String((zatoshis % ZATOSHIS_PER_ZEC) + ZATOSHIS_PER_ZEC).slice(1);
    while (fraction.length > 0 && fraction.charAt(fraction.length - 1) === "0") {
      fraction = fraction.slice(0, fraction.length - 1);
    }
    return fraction.length > 0 ? digits(whole) + "." + fraction : digits(whole);
  }

  /* A time the page was given, in UTC. Not a clock read: the same number always renders the same
     string, which is what lets a replayed act be rendered frame by frame. */
  function utcStamp(seconds) {
    var when = new Date(seconds * 1000);
    if (isNaN(when.getTime())) {
      return "an unreadable time";
    }
    var iso = when.toISOString();
    return iso.slice(0, 10) + " " + iso.slice(11, 19) + " UTC";
  }

  function shortHex(hex) {
    return hex.slice(0, MARKER_PREFIX_BYTES * 2) + "…";
  }

  /* The first MARKERS_SHOWN markers, shortened, and a line saying how many were left out. */
  function markerLines(markers, whenEmpty) {
    var lines = [];
    for (var i = 0; i < markers.length && i < MARKERS_SHOWN; i += 1) {
      lines.push(shortHex(markers[i].marker));
    }
    if (markers.length > MARKERS_SHOWN) {
      lines.push("and " + digits(markers.length - MARKERS_SHOWN) + " more");
    }
    if (lines.length === 0) {
      lines.push(whenEmpty);
    }
    return lines;
  }

  function roomOf(state, context) {
    for (var i = 0; i < state.rooms.length; i += 1) {
      if (state.rooms[i].context === context) {
        return state.rooms[i];
      }
    }
    return null;
  }

  /* The newest thing this room has answered, whether or not it was written down. A refusal is
     never stored, so it can only come from the gate's in-memory feed. */
  function latest(room) {
    var all = room.recent.concat(room.verdicts);
    var newest = null;
    for (var i = 0; i < all.length; i += 1) {
      if (newest === null || all[i].at > newest.at) {
        newest = all[i];
      }
    }
    return newest;
  }

  function verdictClass(verdict) {
    if (!verdict) {
      return "verdict-none";
    }
    if (verdict.verdict === "ADMITTED") {
      return "verdict-admitted";
    }
    if (verdict.verdict === "REFUSED") {
      return "verdict-refused";
    }
    return "verdict-inconclusive";
  }

  function sharedMarkerCount(state) {
    var seenIn = {};
    for (var i = 0; i < state.rooms.length; i += 1) {
      var room = state.rooms[i];
      for (var m = 0; m < room.markers.length; m += 1) {
        var marker = room.markers[m].marker;
        if (!seenIn[marker]) {
          seenIn[marker] = {};
        }
        seenIn[marker][room.context] = true;
      }
    }
    var shared = 0;
    for (var key in seenIn) {
      if (Object.prototype.hasOwnProperty.call(seenIn, key)) {
        var rooms = 0;
        for (var context in seenIn[key]) {
          if (Object.prototype.hasOwnProperty.call(seenIn[key], context)) {
            rooms += 1;
          }
        }
        if (rooms > 1) {
          shared += 1;
        }
      }
    }
    return shared;
  }

  function sourceBadge(state) {
    var label =
      state.mode === "replay"
        ? "replay · " + state.replay + " · " + state.source
        : "live from this gate · " + state.source;
    return node("p", "badge", label);
  }

  function ruleSentence(room) {
    return (
      "Held at least " +
      zec(Number(room.min_zatoshis)) +
      " ZEC privately, unmoved from block " +
      digits(room.window.h0) +
      " through " +
      digits(room.window.h1) +
      "."
    );
  }

  function headPanel(state, room) {
    var head = node("div", "head");
    var left = node("div");
    left.appendChild(node("h1", null, room ? room.context : "This gate's rooms"));
    left.appendChild(
      node(
        "p",
        "rule-aside",
        "network " + (room ? room.network : state.rooms[0].network) + " · one door, one rule, the same for everyone"
      )
    );
    head.appendChild(left);
    head.appendChild(sourceBadge(state));
    return head;
  }

  function rulePanel(room) {
    return panel("rule-panel", [node("h2", "panel-title", "The door rule"), node("p", "rule", ruleSentence(room))]);
  }

  function verdictPanel(state, room) {
    var newest = latest(room);
    var children = [node("h2", "panel-title", "The latest verdict at this door")];
    children.push(node("p", "verdict", newest ? newest.verdict : "NOBODY HAS KNOCKED YET"));
    if (newest && newest.code) {
      children.push(node("p", "code", newest.code));
      if (state.codes[newest.code]) {
        children.push(node("p", "message", state.codes[newest.code].message));
      }
    } else if (newest) {
      children.push(node("p", "message", "The door opened. One marker set was written down, and nothing else."));
    } else {
      children.push(node("p", "message", "This room has answered nothing yet in this gate's lifetime."));
    }
    if (newest) {
      var when = "answered " + utcStamp(newest.at);
      var retry = newest.code ? state.codes[newest.code] : null;
      if (retry) {
        when += retry.retryable ? " · retryable: yes" : " · retryable: no";
      }
      children.push(node("p", "rule-aside", when));
      if (newest.at_note) {
        children.push(node("p", "rule-aside", newest.at_note));
      }
    }
    var section = panel("verdict-panel " + verdictClass(newest), children);
    /* The one thing on a live page that changes on its own, so it is the one live region. */
    section.setAttribute("aria-live", "polite");
    return section;
  }

  function learnedPanel(room) {
    var kept = markerLines(room.markers, "nothing yet");
    var written = room.verdicts.length > 0 ? room.verdicts[0] : null;
    var left = node("div");
    left.appendChild(node("h3", null, "What the door wrote down"));
    left.appendChild(list("kept", kept));
    left.appendChild(
      node(
        "p",
        "rule-aside",
        written
          ? digits(room.markers.length) + " markers, " + written.verdict + " at " + utcStamp(written.at) + ", nothing else"
          : "nothing written down in this room yet"
      )
    );
    var right = node("div");
    right.appendChild(node("h3", null, "What it never saw"));
    right.appendChild(
      list("never", [
        "your name",
        "your address",
        "your balance",
        "which coins you hold",
        "your transaction history",
        "your IP address"
      ])
    );
    var grid = filled(node("div", "learned"), [left, right]);
    return panel(null, [node("h2", "panel-title", "What the door learned"), grid]);
  }

  function timelinePanel(room) {
    var bar = node("div", "timeline-bar");
    bar.appendChild(node("div", "timeline-held"));
    var ends = filled(node("div", "timeline-ends"), [
      node("span", null, "h0 " + digits(room.window.h0)),
      node("span", null, "h1 " + digits(room.window.h1))
    ]);
    var timeline = filled(node("div", "timeline"), [bar, ends]);
    return panel(null, [
      node("h2", "panel-title", "The window"),
      timeline,
      node(
        "p",
        "message",
        "Coins that already existed at block " +
          digits(room.window.h0) +
          " and were still unspent at block " +
          digits(room.window.h1) +
          ": " +
          digits(room.window.h1 - room.window.h0) +
          " blocks of holding still. Arrive later, or spend inside it, and there is nothing to prove."
      )
    ]);
  }

  function roomCard(state, room) {
    var here = state.room === room.context;
    var card = node("div", here ? "room-card room-card-here" : "room-card");
    card.appendChild(node("h3", null, room.context + (here ? "  (this room)" : "")));
    card.appendChild(node("p", "rule-aside", "markers: " + digits(room.markers.length)));
    card.appendChild(list("markers", markerLines(room.markers, "none yet")));
    return card;
  }

  function roomsPanel(state) {
    var cards = [];
    for (var i = 0; i < state.rooms.length; i += 1) {
      cards.push(roomCard(state, state.rooms[i]));
    }
    var shared = sharedMarkerCount(state);
    var tally = node("div", "room-card");
    tally.appendChild(
      node("p", shared === 0 ? "shared shared-zero" : "shared", "markers shared between rooms: " + digits(shared))
    );
    tally.appendChild(
      node(
        "p",
        "rule-aside",
        "Each door derives its own markers, so the same coins at two doors cannot be linked by what either keeps."
      )
    );
    cards.push(tally);
    return panel(null, [node("h2", "panel-title", "The same coins at two doors"), filled(node("div", "rooms"), cards)]);
  }

  function botPanel(state, room, onMembers) {
    var members = Math.max(0, Math.min(MEMBERS_MAX, Math.floor(Number(state.members) || 0)));
    var perMember = Number(room.min_zatoshis);
    var label = node("label", null, "fake members");
    label.setAttribute("for", MEMBERS_INPUT);
    var input = node("input");
    input.id = MEMBERS_INPUT;
    input.type = "number";
    input.min = "0";
    input.max = String(MEMBERS_MAX);
    input.step = "1";
    input.value = String(members);
    input.addEventListener("input", function () {
      onMembers(input.value);
    });
    var cost = node("p", "cost", zec(members * perMember) + " ZEC");
    var row = filled(node("div", "bot"), [label, input, cost]);
    return panel(null, [
      node("h2", "panel-title", "What a bot army would cost"),
      row,
      node(
        "p",
        "message",
        digits(members) +
          " members at " +
          zec(perMember) +
          " ZEC each, held unmoved across the whole window. Not staked, not locked: simply not spent, and provably so."
      )
    ]);
  }

  function footPanel(state, room) {
    var lines = state.provenance.slice(0);
    if (room) {
      lines.unshift(
        "The rule above is this room's published manifest: nothing else qualifies a holder, and nothing else is asked."
      );
      lines.push(
        "h1 nullifier-set root " + room.h1_root + " (" + room.h1_root_provenance + ")."
      );
      lines.push("room log shows at most " + digits(room.limit) + " rows, newest first.");
    }
    if (state.gate.note) {
      lines.push(state.gate.note);
    }
    return list("foot", lines);
  }

  /* Five fixed rows on a fixed stage, so the layout is the same pixel for pixel in a browser and
     in the rendered video, and no act can push the frame out of shape. */
  function roomStage(state, onMembers) {
    var room = roomOf(state, state.room);
    var stage = node("div", "stage stage-room");
    stage.appendChild(headPanel(state, room));
    stage.appendChild(rulePanel(room));
    stage.appendChild(
      filled(node("div", "thirds"), [verdictPanel(state, room), learnedPanel(room), timelinePanel(room)])
    );
    stage.appendChild(filled(node("div", "wide-and-narrow"), [roomsPanel(state), botPanel(state, room, onMembers)]));
    stage.appendChild(footPanel(state, room));
    return stage;
  }

  function pickerStage(state) {
    var stage = node("div", "stage stage-picker");
    stage.appendChild(headPanel(state, null));
    stage.appendChild(
      panel(null, [
        node("h2", "panel-title", "What a door here asks"),
        node("p", "rule", "One rule, published in advance, the same for everyone who knocks."),
        node(
          "p",
          "rule-aside",
          "A door learns that someone held enough, long enough. It never learns who, or how much, or which coins."
        )
      ])
    );
    var cards = [];
    for (var i = 0; i < state.rooms.length; i += 1) {
      var room = state.rooms[i];
      var link = node("a", null, room.context);
      link.href = "/room/" + encodeURIComponent(room.context);
      var card = node("div", "room-card");
      card.appendChild(link);
      card.appendChild(node("p", "message", ruleSentence(room)));
      card.appendChild(
        node("p", "rule-aside", "network " + room.network + " · markers: " + digits(room.markers.length))
      );
      cards.push(card);
    }
    stage.appendChild(filled(node("div", "picker-rooms"), cards));
    stage.appendChild(footPanel(state, null));
    return stage;
  }

  function render(state, onMembers) {
    var change =
      onMembers ||
      function () {
        return undefined;
      };
    return state.room ? roomStage(state, change) : pickerStage(state);
  }

  function mount(state, focusId) {
    var app = document.getElementById("app");
    if (!app) {
      return;
    }
    while (app.firstChild) {
      app.removeChild(app.firstChild);
    }
    app.appendChild(
      render(state, function (value) {
        state.members = Math.max(0, Math.min(MEMBERS_MAX, Math.floor(Number(value) || 0)));
        mount(state, MEMBERS_INPUT);
      })
    );
    if (focusId) {
      var focused = document.getElementById(focusId);
      if (focused) {
        focused.focus();
      }
    }
  }

  /* The stage is always 1920 by 1080, so the video and the browser show the same layout. This is
     the only thing on the page that reads the viewport, and it reads nothing else. */
  function fit() {
    var scale = Math.min(global.innerWidth / STAGE_W, global.innerHeight / STAGE_H);
    document.documentElement.style.setProperty("--scale", String(scale));
  }

  function boot() {
    var carrier = document.getElementById("unstirred-state");
    if (!carrier) {
      return;
    }
    var state = JSON.parse(carrier.textContent);
    global.Unstirred = { state: state, render: render, mount: mount, fit: fit };
    fit();
    global.addEventListener("resize", fit);
    mount(state);
  }

  boot();
})(window);
