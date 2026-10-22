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
  var MARKERS_SHOWN = 3;
  var MEMBERS_MAX = 1000000;
  var STAGE_W = 1920;
  var STAGE_H = 1080;
  var MEMBERS_INPUT = "members";
  /* Testnet's NU7 block target, which briefing/CLAUDE.md records as 25 s (observed ~27 s). It is a
     property of the chain, not of this gate, so every line that uses it says "about" and names it. */
  var SECONDS_PER_BLOCK = 25;
  /* How much of the timeline sits outside the window, so h0 and h1 are points on an axis rather
     than its ends, and a holder line can run past either of them. */
  var AXIS_MARGIN = 0.14;
  /* How much of the rail a holder line that runs off the end is drawn as, before it fades out. */
  var HOLDER_STUB = 0.07;

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

  function oneDecimal(value) {
    var rounded = Math.round(value * 10) / 10;
    return rounded === Math.round(rounded) ? String(Math.round(rounded)) : String(rounded);
  }

  /* How long a run of blocks takes, roughly, at the chain's target block time. */
  function roughly(blocks) {
    var seconds = blocks * SECONDS_PER_BLOCK;
    var hours = seconds / 3600;
    if (hours < 1) {
      return Math.round(seconds / 60) + " minutes";
    }
    if (hours < 48) {
      return oneDecimal(hours) + (oneDecimal(hours) === "1" ? " hour" : " hours");
    }
    return oneDecimal(hours / 24) + " days";
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

  /* How many markers more than one room holds. Only meaningful once at least two rooms hold any:
     before that there is nothing to compare, and a zero would read as a finding. */
  function sharing(state) {
    var holding = [];
    var seenIn = {};
    for (var i = 0; i < state.rooms.length; i += 1) {
      var room = state.rooms[i];
      if (room.markers.length > 0) {
        holding.push(room.context);
      }
      for (var m = 0; m < room.markers.length; m += 1) {
        var marker = room.markers[m].marker;
        if (!Object.prototype.hasOwnProperty.call(seenIn, marker)) {
          seenIn[marker] = {};
        }
        seenIn[marker][room.context] = true;
      }
    }
    if (holding.length < 2) {
      return { comparable: false, rooms: holding.length, shared: 0 };
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
    return { comparable: true, rooms: holding.length, shared: shared };
  }

  function sourceBadge(state) {
    var label;
    if (state.mode === "replay") {
      label = "replay · " + state.replay + " · " + state.source;
    } else {
      label = state.source === "gate" ? "live from this gate" : "live from this gate · " + state.source;
    }
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
    left.appendChild(node("p", "rule-aside", "network " + (room ? room.network : state.rooms[0].network)));
    head.appendChild(left);
    head.appendChild(sourceBadge(state));
    return head;
  }

  /* The rule is the hero until this door has answered something; after that the verdict is, and
     the rule holds its ground as one compact line. */
  function rulePanel(room, asHero) {
    return panel(asHero ? "rule-panel rule-hero" : "rule-panel rule-line", [
      node("h2", "panel-title", "The door rule"),
      node("p", "rule", ruleSentence(room))
    ]);
  }

  function verdictPanel(state, room, asHero) {
    var newest = latest(room);
    var children = [node("h2", "panel-title", "The latest verdict at this door")];
    children.push(node("p", "verdict", newest ? newest.verdict : "Nobody has knocked yet"));
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
      /* A time the run did not record is never shown as if it had been: the card says it was
         derived, and the provenance foot says how. */
      if (newest.at_note) {
        when += " (time derived)";
      }
      var words = newest.code ? state.codes[newest.code] : null;
      if (words) {
        when += words.retryable ? " · retryable: yes" : " · retryable: no";
      }
      children.push(node("p", "rule-aside", when));
    }
    var section = panel(
      "verdict-panel " + verdictClass(newest) + (asHero ? " verdict-hero" : ""),
      children
    );
    /* The one thing on a live page that changes on its own, so it is the one live region. */
    section.setAttribute("aria-live", "polite");
    return section;
  }

  /* What a refusal did to the store, in the store's own terms: nothing. */
  function unchangedLine(room) {
    var newest = latest(room);
    if (!newest || newest.verdict === "ADMITTED") {
      return null;
    }
    return newest.verdict === "REFUSED"
      ? "unchanged: the refusal wrote nothing"
      : "unchanged: nothing was decided, so nothing was written";
  }

  function learnedPanel(room) {
    var written = room.verdicts.length > 0 ? room.verdicts[0] : null;
    var left = node("div");
    left.appendChild(node("h3", null, "What the door wrote down"));
    left.appendChild(list("kept", markerLines(room.markers, "nothing yet")));
    var unchanged = unchangedLine(room);
    if (unchanged) {
      left.appendChild(node("p", "unchanged", unchanged));
    }
    left.appendChild(
      node(
        "p",
        "rule-aside",
        written
          ? digits(room.markers.length) + " markers, " + written.verdict + ", nothing else"
          : "nothing written down here yet"
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

  /* The axis a window is drawn on: the window itself plus a margin on each side, whatever a
     holder's line does. Stretching it to reach a line that begins twenty thousand blocks after h1
     would leave the window a sliver of the rail with its caption wrapped into a column of single
     words; the window is what the room is about, so it keeps the axis. A line that runs past an
     end is drawn to that end, faded off it, and the note underneath says in blocks how far it
     really goes. */
  function axisOf(room) {
    var margin = Math.max(1, Math.round((room.window.h1 - room.window.h0) * AXIS_MARGIN));
    return { low: room.window.h0 - margin, high: room.window.h1 + margin };
  }

  /* One holder's line against one window: where it can be drawn on an axis that may not reach it,
     and what to say about it. Every number here is computed from the two block heights the state
     carries, so a line that is off the end still names where it really is. */
  function holderOnAxis(axis, window_, holder) {
    var stub = Math.max(1, Math.round((axis.high - axis.low) * HOLDER_STUB));
    var label = holder.label + ": blocks " + digits(holder.from) + " to " + digits(holder.to);
    if (holder.from <= window_.h0 && holder.to >= window_.h1) {
      return {
        from: holder.from,
        to: holder.to,
        rail: "holder-covers",
        note: "holder-covers-note",
        text: label + " — covers the whole window"
      };
    }
    if (holder.from > window_.h1) {
      return {
        from: axis.high - stub,
        to: axis.high,
        rail: "holder-short holder-beyond-end",
        note: "holder-short-note",
        text:
          label +
          " — " +
          digits(holder.from - window_.h1) +
          " blocks after h1, never inside this window"
      };
    }
    if (holder.to < window_.h0) {
      return {
        from: axis.low,
        to: axis.low + stub,
        rail: "holder-short holder-beyond-start",
        note: "holder-short-note",
        text:
          label +
          " — gone " +
          digits(window_.h0 - holder.to) +
          " blocks before h0, never inside this window"
      };
    }
    return {
      from: Math.max(axis.low, holder.from),
      to: Math.min(axis.high, holder.to),
      rail: "holder-short",
      note: "holder-short-note",
      text: label + " — does not cover the window, so there is nothing to prove"
    };
  }

  function atPercent(axis, height) {
    return ((height - axis.low) / (axis.high - axis.low)) * 100;
  }

  function placed(className, axis, from, to, text) {
    var element = node("div", className, text);
    var left = atPercent(axis, from);
    element.style.left = left + "%";
    if (to !== null) {
      element.style.width = atPercent(axis, to) - left + "%";
    }
    return element;
  }

  /* The window as a line with two named points and the band between them, with room for one
     holder's own line underneath. A holder whose line does not cover the whole band has nothing to
     prove here, and is drawn short of it rather than described. */
  function timelinePanel(room) {
    var axis = axisOf(room);
    var window_ = room.window;
    /* The caption sits in the flow above the rail, pinned to the band's own span: in flow it can
       wrap to a second line and push the rail down, rather than growing over it. */
    var caption = node("div", "axis-caption", "coins must exist here and stay unspent through here");
    var captionLeft = atPercent(axis, window_.h0);
    caption.style.marginLeft = captionLeft + "%";
    caption.style.width = atPercent(axis, window_.h1) - captionLeft + "%";
    var rail = node("div", "axis");
    rail.appendChild(node("div", "axis-rule"));
    rail.appendChild(placed("axis-band", axis, window_.h0, window_.h1, null));
    rail.appendChild(placed("axis-point", axis, window_.h0, null, null));
    rail.appendChild(placed("axis-point", axis, window_.h1, null, null));
    rail.appendChild(placed("axis-mark axis-mark-start", axis, window_.h0, null, "h0 " + digits(window_.h0)));
    rail.appendChild(placed("axis-mark axis-mark-end", axis, window_.h1, null, "h1 " + digits(window_.h1)));
    var children = [node("h2", "panel-title", "The window"), caption, rail];
    if (room.holder) {
      var line = holderOnAxis(axis, window_, room.holder);
      var holder = node("div", "axis holder-rail");
      holder.appendChild(placed("axis-holder " + line.rail, axis, line.from, line.to, null));
      children.push(holder);
      children.push(node("p", "holder-note " + line.note, line.text));
    }
    children.push(
      node(
        "p",
        "message",
        "Coins must exist by block " +
          digits(window_.h0) +
          " and stay unspent through block " +
          digits(window_.h1) +
          ": " +
          digits(window_.h1 - window_.h0) +
          " blocks, about " +
          roughly(window_.h1 - window_.h0) +
          "."
      )
    );
    return panel(null, children);
  }

  function roomCard(state, room) {
    var here = state.room === room.context;
    var card = node("div", here ? "room-card room-card-here" : "room-card");
    card.appendChild(node("h3", null, room.context + (here ? " (this room)" : "")));
    card.appendChild(node("p", "rule-aside", "markers: " + digits(room.markers.length)));
    card.appendChild(list("markers", markerLines(room.markers, "none yet")));
    return card;
  }

  function roomsPanel(state) {
    var cards = [];
    for (var i = 0; i < state.rooms.length; i += 1) {
      cards.push(roomCard(state, state.rooms[i]));
    }
    var tally = sharing(state);
    var headline = tally.comparable
      ? node(
          "p",
          tally.shared === 0 ? "shared shared-zero" : "shared",
          "markers shared between rooms: " + digits(tally.shared)
        )
      : node("p", "shared shared-waiting", "nothing to compare yet");
    var aside = tally.comparable
      ? "Each door derives its own markers, so the same coins at two doors cannot be linked by what either keeps."
      : tally.rooms === 0
        ? "No room here holds a marker yet, so there is nothing to compare between doors."
        : "Only one room here holds markers. A second room has to admit something before the two can be compared.";
    return panel(null, [
      node("h2", "panel-title", "The same coins at two doors"),
      filled(node("div", "rooms"), cards),
      headline,
      node("p", "rule-aside", aside)
    ]);
  }

  function botPanel(state, room, onMembers) {
    var members = Math.max(0, Math.min(MEMBERS_MAX, Math.floor(Number(state.members) || 0)));
    var perMember = Number(room.min_zatoshis);
    var blocks = room.window.h1 - room.window.h0;
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
    var row = filled(node("div", "bot"), [label, input]);
    return panel(null, [
      node("h2", "panel-title", "What a bot army would cost"),
      row,
      node("p", "cost", zec(members * perMember) + " ZEC"),
      node(
        "p",
        "message",
        digits(members) +
          " members at " +
          zec(perMember) +
          " ZEC each, held unmoved for " +
          digits(blocks) +
          " blocks (about " +
          roughly(blocks) +
          " at ~" +
          SECONDS_PER_BLOCK +
          " s per block). Not staked, not locked: simply not spent, and provably so."
      )
    ]);
  }

  /* Where these numbers came from, including how a derived time was derived. In presentation mode
     the technical notes come off — nobody checks a root hex off a projector — and what is left is
     the one line saying where the numbers came from. Nothing load-bearing is lost: the verdict card
     still marks a derived time as derived, and a refusal that wrote nothing says so where it was
     written down. */
  function footPanel(state, room) {
    var lines = state.present ? state.provenance.slice(0, 1) : state.provenance.slice(0);
    if (room && !state.present) {
      var newest = latest(room);
      if (newest && newest.at_note) {
        lines.push(newest.at_note);
      }
      lines.push("h1 nullifier-set root " + room.h1_root + " (" + room.h1_root_provenance + ").");
      /* The cap is on what the log route answers, which only a live page ever asks. */
      if (state.mode === "live") {
        lines.push("Room log shows at most " + digits(room.limit) + " rows, newest first.");
      }
    }
    if (state.gate.note) {
      lines.push(state.gate.note);
    }
    return list("foot", lines);
  }

  function stageClasses(state, extra) {
    return "stage " + extra + (state.present ? " present" : "");
  }

  /* No row is ever smaller than its own content: the two rows of panels take what they need and
     then share what is left, so nothing is cut off to make a row fit. The stage itself stays
     exactly 1920 by 1080 at scale 1, which is the frame the video is rendered in. */
  function roomStage(state, onMembers) {
    var room = roomOf(state, state.room);
    var verdictIsHero = latest(room) !== null;
    var stage = node("div", stageClasses(state, "stage-room"));
    stage.appendChild(headPanel(state, room));
    stage.appendChild(rulePanel(room, !verdictIsHero));
    stage.appendChild(
      filled(node("div", "hero-row"), [verdictPanel(state, room, verdictIsHero), learnedPanel(room)])
    );
    stage.appendChild(
      filled(node("div", "thirds"), [timelinePanel(room), roomsPanel(state), botPanel(state, room, onMembers)])
    );
    stage.appendChild(footPanel(state, room));
    return stage;
  }

  function pickerStage(state) {
    var stage = node("div", stageClasses(state, "stage-picker"));
    stage.appendChild(headPanel(state, null));
    /* The rule and the rooms are one block at the top of what is left, so the room cards stay the
       height of their own content and there is no gap above them. */
    var body = node("div", "picker-body");
    body.appendChild(
      panel("rule-panel rule-hero", [
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
      var card = node("div", "room-card picker-card");
      card.appendChild(node("h3", null, room.context));
      card.appendChild(node("p", "message", ruleSentence(room)));
      card.appendChild(
        node("p", "rule-aside", "network " + room.network + " · markers: " + digits(room.markers.length))
      );
      var link = node("a", "open-room", "Open room");
      link.href = "/room/" + encodeURIComponent(room.context);
      card.appendChild(link);
      cards.push(card);
    }
    body.appendChild(filled(node("div", "picker-rooms"), cards));
    stage.appendChild(body);
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
