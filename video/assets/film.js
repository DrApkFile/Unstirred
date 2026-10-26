/* The film's four motion helpers, loaded once by index.html so every scene drives its terminal,
   its pointer, its camera and its highlight the same way.

   Everything here is a pure function of timeline time: one linear driver per effect, state
   recomputed from that driver on each update, no mutable tracker and no clock. Seeking backwards
   lands on exactly the frame seeking forwards did, which is what frame-by-frame rendering needs.

   Composes rules/discrete-text-sequence (the smooth-slice variation), rules/context-sensitive-cursor,
   rules/cursor-click-ripple and rules/viewport-change from the hyperframes-animation skill. */

(function (global) {
  "use strict";

  /* The page is shown through a 1680x945 window at its own 1920x1080, so one block scale and no
     reflow: the layout inside the browser frame is the page's own at scale 1, just smaller. */
  var VIEWPORT_W = 1680;
  var VIEWPORT_H = 945;
  var PAGE_W = 1920;
  var PAGE_H = 1080;
  var BASE_SCALE = VIEWPORT_W / PAGE_W;
  var BLINK_PERIOD = 0.8;
  var TYPING_SOLID = 0.22;

  /* One terminal. Every `.line` inside `root` carries its own text in the markup — the static end
     state — plus `data-at`, the second it appears, and optionally `data-type`, how long it takes
     to type rather than simply print. `data-cursor` marks the lines the block cursor may sit on. */
  function terminal(tl, root, duration) {
    var lines = Array.prototype.slice.call(root.querySelectorAll(".line")).map(function (element) {
      return {
        element: element,
        text: element.textContent,
        at: Number(element.getAttribute("data-at") || 0),
        typeFor: Number(element.getAttribute("data-type") || 0),
        cursor: element.getAttribute("data-cursor") === "1"
      };
    });
    var cursor = root.querySelector(".cursor-block");

    function paint(t) {
      var carrying = null;
      var typing = false;
      for (var i = 0; i < lines.length; i += 1) {
        var line = lines[i];
        if (t < line.at) {
          line.element.textContent = "";
          line.element.style.visibility = "hidden";
          continue;
        }
        line.element.style.visibility = "visible";
        if (line.typeFor > 0) {
          var through = Math.min(1, (t - line.at) / line.typeFor);
          line.element.textContent = line.text.slice(0, Math.ceil(through * line.text.length));
          if (through < 1) {
            typing = true;
          }
        } else {
          line.element.textContent = line.text;
        }
        if (line.cursor) {
          carrying = line;
        }
      }
      if (!cursor) {
        return;
      }
      if (!carrying) {
        cursor.style.visibility = "hidden";
        return;
      }
      cursor.style.visibility = "visible";
      if (cursor.parentNode !== carrying.element) {
        carrying.element.appendChild(cursor);
      }
      cursor.style.opacity =
        typing || Math.sin((t / BLINK_PERIOD) * Math.PI * 2) > 0 ? "1" : "0";
    }

    var driver = { t: 0 };
    paint(0);
    tl.to(
      driver,
      {
        t: duration,
        duration: duration,
        ease: "none",
        onUpdate: function () {
          paint(driver.t);
        }
      },
      0
    );
  }

  /* A driven pointer. `legs` are moves — {at, dur, from: [x, y], to: [x, y]} in viewport pixels —
     and clicks — {at, click: true}, which dip the pointer and send one ripple out from it. */
  function pointer(tl, element, legs) {
    var ripple = element.querySelector(".ripple");
    var first = legs.find(function (leg) {
      return leg.from;
    });
    if (first) {
      global.gsap.set(element, { x: first.from[0], y: first.from[1] });
    }
    legs.forEach(function (leg) {
      if (leg.click) {
        tl.to(element, { scale: 0.86, duration: 0.09, yoyo: true, repeat: 1, ease: "power2.inOut" }, leg.at);
        if (ripple) {
          tl.fromTo(
            ripple,
            { scale: 0.2, opacity: 0.9 },
            { scale: 1.9, opacity: 0, duration: 0.65, ease: "power2.out" },
            leg.at
          );
        }
        return;
      }
      tl.fromTo(
        element,
        { x: leg.from[0], y: leg.from[1] },
        {
          x: leg.to[0],
          y: leg.to[1],
          duration: leg.dur,
          ease: leg.ease || "power2.inOut",
          immediateRender: false
        },
        leg.at
      );
    });
  }

  /* A virtual camera over the page in the browser frame. One composite transform on the world
     wrapper, so scale and translate can never disagree: a page point (px, py) is put at the
     viewport's centre by translate(centre - scale * point). Zoom 1 with the page's own centre
     leaves the transform at the resting scale, which is the frame every scene opens on. */
  function camera(tl, world, legs) {
    var cam = { zoom: 1, px: PAGE_W / 2, py: PAGE_H / 2 };

    function apply() {
      var scale = BASE_SCALE * cam.zoom;
      world.style.transform =
        "translate(" +
        (VIEWPORT_W / 2 - scale * cam.px) +
        "px, " +
        (VIEWPORT_H / 2 - scale * cam.py) +
        "px) scale(" +
        scale +
        ")";
    }

    apply();
    (legs || []).forEach(function (leg) {
      tl.to(
        cam,
        {
          zoom: leg.zoom,
          px: leg.px,
          py: leg.py,
          duration: leg.dur,
          ease: leg.ease || "power2.inOut",
          onUpdate: apply
        },
        leg.at
      );
    });
  }

  /* A highlight that dims everything but one rectangle of the page, in page pixels, by riding the
     same scale the page is shown at. It is placed over the viewport rather than inside the world,
     so a camera move does not drag it. */
  function spotlight(tl, element, rect, at, dur) {
    element.style.left = rect[0] * BASE_SCALE + "px";
    element.style.top = rect[1] * BASE_SCALE + "px";
    element.style.width = rect[2] * BASE_SCALE + "px";
    element.style.height = rect[3] * BASE_SCALE + "px";
    tl.fromTo(element, { opacity: 0 }, { opacity: 1, duration: dur, ease: "power2.out" }, at);
  }

  global.film = {
    terminal: terminal,
    pointer: pointer,
    camera: camera,
    spotlight: spotlight,
    BASE_SCALE: BASE_SCALE,
    TYPING_SOLID: TYPING_SOLID
  };
})(window);
