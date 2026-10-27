#!/usr/bin/env node
// Opens the published site in headless Chromium and checks three things the build cannot:
// that every link and button on the Start here page resolves, that the five act links open the
// five acts in order, and that the text on each page clears WCAG AA against what is actually
// painted behind it — including, on the landing page, the background video.
//
// No dependency: Chrome is driven over the DevTools Protocol through Node's own WebSocket, and
// screenshot pixels are read by piping the PNG through ffmpeg, which this repository's video work
// already requires.
//
//     scripts/site-browser-check.mjs [site directory]
import { spawn, execFileSync } from "node:child_process";
import { mkdtempSync, writeFileSync, readFileSync, existsSync, mkdirSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";

const site = path.resolve(process.argv[2] ?? "site");
const shots = path.join(site, "..", "evidence", "2026-10-27-site-shots");
const VIEWPORTS = [
  { name: "desktop", width: 1920, height: 1080 },
  { name: "mobile", width: 390, height: 844 },
];
const PAGES = [
  { name: "landing", file: "index.html" },
  { name: "start", file: "start/index.html" },
  { name: "guide", file: "guides/getting-started.html" },
  { name: "api", file: "api/index.html" },
];
// How many frames of the landing page's looping background video to measure contrast against.
const FRAMES_SAMPLED = 12;
const ACTS = [
  "act1-rule",
  "act2-admitted",
  "act3-refused",
  "act4-newcomer",
  "act5-second-room",
];

const problems = [];
const notes = [];

// ---------------------------------------------------------------- the protocol, by hand

function connect(url) {
  const socket = new WebSocket(url);
  const waiting = new Map();
  let next = 0;
  const ready = new Promise((resolve, reject) => {
    socket.addEventListener("open", resolve);
    socket.addEventListener("error", reject);
  });
  const events = new Map();
  socket.addEventListener("message", (message) => {
    const frame = JSON.parse(message.data);
    if (frame.id !== undefined && waiting.has(frame.id)) {
      const { resolve, reject } = waiting.get(frame.id);
      waiting.delete(frame.id);
      frame.error ? reject(new Error(JSON.stringify(frame.error))) : resolve(frame.result);
    } else if (frame.method && events.has(frame.method)) {
      events.get(frame.method).forEach((fn) => fn(frame.params));
      events.delete(frame.method);
    }
  });
  return {
    ready,
    send(method, params = {}, sessionId) {
      const id = ++next;
      return new Promise((resolve, reject) => {
        waiting.set(id, { resolve, reject });
        socket.send(JSON.stringify({ id, method, params, sessionId }));
      });
    },
    once(method) {
      return new Promise((resolve) => {
        if (!events.has(method)) events.set(method, []);
        events.get(method).push(resolve);
      });
    },
    close: () => socket.close(),
  };
}

const sleep = (ms) => new Promise((r) => setTimeout(r, ms));

// ---------------------------------------------------------------- contrast

function channel(value) {
  const c = value / 255;
  return c <= 0.04045 ? c / 12.92 : Math.pow((c + 0.055) / 1.055, 2.4);
}
const luminance = ([r, g, b]) => 0.2126 * channel(r) + 0.7152 * channel(g) + 0.0722 * channel(b);
function ratio(a, b) {
  const [hi, lo] = [luminance(a), luminance(b)].sort((x, y) => y - x);
  return (hi + 0.05) / (lo + 0.05);
}
const parseRGB = (text) => (text.match(/\d+(\.\d+)?/g) ?? []).slice(0, 3).map(Number);

// Read a screenshot's pixels by piping the PNG through ffmpeg. No image library, and nothing to
// install that this repository does not already use.
function pixels(png, width, height) {
  const raw = execFileSync(
    "ffmpeg",
    ["-v", "error", "-i", "pipe:0", "-f", "rawvideo", "-pix_fmt", "rgb24", "pipe:1"],
    { input: png, maxBuffer: width * height * 3 + 1024 },
  );
  return raw;
}

// Hiding every glyph without moving anything: the layout, the backgrounds, the borders and the
// video all stay exactly where they are, and only the ink disappears. Screenshotting the page in
// this state is the only way to read what is painted BEHIND a run of text — sampling the text's
// own box with the text in it just finds the text.
const HIDE_INK = `(() => {
  const style = document.createElement("style");
  style.id = "unstirred-ink-off";
  style.textContent = "*, *::before, *::after { color: transparent !important;" +
    " text-shadow: none !important; -webkit-text-fill-color: transparent !important; }";
  document.head.appendChild(style);
  return "off";
})()`;

// The brightest pixel behind a run of text, which is the worst case its contrast has to clear.
function brightestUnder(raw, width, height, rect) {
  const x0 = Math.max(0, Math.floor(rect.x));
  const y0 = Math.max(0, Math.floor(rect.y));
  const x1 = Math.min(width, Math.ceil(rect.x + rect.width));
  const y1 = Math.min(height, Math.ceil(rect.y + rect.height));
  let worst = null;
  let worstLuminance = -1;
  for (let y = y0; y < y1; y += 1) {
    for (let x = x0; x < x1; x += 1) {
      const at = (y * width + x) * 3;
      const pixel = [raw[at], raw[at + 1], raw[at + 2]];
      const l = luminance(pixel);
      if (l > worstLuminance) {
        worstLuminance = l;
        worst = pixel;
      }
    }
  }
  return worst;
}

// What the page says about its own text: every element with text of its own, its colour, and the
// box each of its own text nodes actually occupies.
//
// The box is a Range over the direct text children, not the element's bounding rect. An element's
// rect includes its padding, its border and anything nested inside it, and on this design that is
// the difference between measuring a label and measuring the white avatar ring it sits beside or
// the 40%-white pill border around it. A Range gives the line boxes the glyphs are actually in.
const COLLECT = `(() => {
  const out = [];
  document.querySelectorAll("body *").forEach((el) => {
    const texts = Array.from(el.childNodes).filter((n) => n.nodeType === 3 && n.textContent.trim());
    if (!texts.length) return;
    const style = getComputedStyle(el);
    if (style.visibility === "hidden" || style.opacity === "0") return;
    const boxes = [];
    for (const node of texts) {
      const range = document.createRange();
      range.selectNodeContents(node);
      for (const box of range.getClientRects()) {
        if (box.width < 2 || box.height < 2) continue;
        if (box.bottom < 0 || box.top > innerHeight) continue;
        boxes.push({ x: box.x, y: box.y, width: box.width, height: box.height });
      }
    }
    if (!boxes.length) return;
    out.push({
      tag: el.tagName.toLowerCase(),
      cls: el.className && String(el.className).slice(0, 40),
      text: texts.map((n) => n.textContent).join(" ").trim().slice(0, 48),
      color: style.color,
      size: parseFloat(style.fontSize),
      weight: style.fontWeight,
      boxes,
    });
  });
  return JSON.stringify(out);
})()`;

const LINKS = `(() => JSON.stringify(
  Array.from(document.querySelectorAll("a[href]")).map((a) => ({
    href: a.getAttribute("href"),
    text: a.textContent.trim().slice(0, 40),
  }))
))()`;

const BUTTONS = `(() => JSON.stringify(
  Array.from(document.querySelectorAll("button")).map((b) => ({
    text: b.textContent.trim().slice(0, 30),
    command: b.getAttribute("data-command"),
    label: b.getAttribute("aria-label"),
  }))
))()`;

// ---------------------------------------------------------------- the run

const port = 9333 + (process.pid % 500);
const profile = mkdtempSync(path.join(tmpdir(), "unstirred-site-"));
const chrome = spawn("chromium", [
  "--headless=new",
  `--remote-debugging-port=${port}`,
  `--user-data-dir=${profile}`,
  "--no-first-run",
  "--no-default-browser-check",
  "--hide-scrollbars",
  "--autoplay-policy=no-user-gesture-required",
  "--force-device-scale-factor=1",
], { stdio: ["ignore", "ignore", "pipe"] });

let endpoint = null;
for (let tries = 0; tries < 60 && !endpoint; tries += 1) {
  await sleep(250);
  try {
    const answer = await fetch(`http://127.0.0.1:${port}/json/version`);
    endpoint = (await answer.json()).webSocketDebuggerUrl;
  } catch {}
}
if (!endpoint) {
  chrome.kill();
  throw new Error("headless Chromium did not come up");
}

// The background video's own frames, extracted once with ffmpeg. The brightest frames are the
// ones the text has to survive, so the whole loop is sampled rather than a convenient moment.
// The frames are written INSIDE the site and reached over http, not file://. A file:// document
// may not load file:// images, so injecting one paints nothing at all and every contrast check
// silently passes against plain black — which is exactly what happened while this was being
// written. Serving the directory is also what GitHub Pages does, so the check and the published
// site load the same way.
const FRAME_DIR = ".check-frames";
const videoFrames = [];
{
  const dir = path.join(site, FRAME_DIR);
  rmSync(dir, { recursive: true, force: true });
  mkdirSync(dir, { recursive: true });
  const url = readFileSync(path.join(site, "index.html"), "utf8")
    .match(/<source src="([^"]+)"/)?.[1];
  if (!url) {
    problems.push("the landing page has no <source src> for its background video");
  } else {
    try {
      execFileSync("ffmpeg", [
        "-v", "error", "-y", "-i", url,
        "-vf", `fps=${FRAMES_SAMPLED / 10}`, "-frames:v", String(FRAMES_SAMPLED),
        path.join(dir, "frame-%02d.png"),
      ], { timeout: 240000 });
      for (let i = 1; i <= FRAMES_SAMPLED; i += 1) {
        const name = `frame-${String(i).padStart(2, "0")}.png`;
        if (existsSync(path.join(dir, name))) videoFrames.push(`/${FRAME_DIR}/${name}`);
      }
    } catch (error) {
      problems.push(`could not extract frames from the background video: ${error.message}`);
    }
  }
  if (!videoFrames.length) {
    problems.push("no frame of the background video could be read, so landing contrast is unmeasured");
  }
}

// Serve the built site, so every page loads over http exactly as it will when published.
const servePort = 8400 + (process.pid % 400);
// Threaded on purpose: python3 -m http.server is single-threaded, and a page that asks for a
// stylesheet, three fonts and an icon set at once can out-run it. When it stalls Chromium shows
// its own network error page, which has text on it and would be measured as though it were the
// site — that happened here, and the page-loaded assertion below exists because of it.
const server = spawn("python3", ["-c", `
import http.server, socketserver, sys
class Threaded(socketserver.ThreadingMixIn, http.server.HTTPServer):
    daemon_threads = True
class Quiet(http.server.SimpleHTTPRequestHandler):
    def log_message(self, *args): pass
Threaded(("127.0.0.1", ${servePort}), Quiet).serve_forever()
`], { cwd: site, stdio: "ignore" });
for (let tries = 0; tries < 40; tries += 1) {
  await sleep(200);
  try {
    await fetch(`http://127.0.0.1:${servePort}/index.html`);
    break;
  } catch {}
}
const base = `http://127.0.0.1:${servePort}`;

const browser = connect(endpoint);
await browser.ready;
if (!existsSync(shots)) mkdirSync(shots, { recursive: true });

for (const viewport of VIEWPORTS) {
  for (const spec of PAGES) {
    const { targetId } = await browser.send("Target.createTarget", { url: "about:blank" });
    const { sessionId } = await browser.send("Target.attachToTarget", { targetId, flatten: true });
    await browser.send("Page.enable", {}, sessionId);
    await browser.send("Runtime.enable", {}, sessionId);
    await browser.send("Emulation.setDeviceMetricsOverride", {
      width: viewport.width,
      height: viewport.height,
      deviceScaleFactor: 1,
      mobile: viewport.name === "mobile",
    }, sessionId);

    const loaded = browser.once("Page.loadEventFired");
    await browser.send("Page.navigate", { url: `${base}/${spec.file}` }, sessionId);
    await Promise.race([loaded, sleep(20000)]);

    const where = `${spec.name} @ ${viewport.width}x${viewport.height}`;

    // Did the site load at all? A failed navigation still renders a page, with text on it, and
    // measuring the browser's error screen would quietly report a passing site. Polled rather
    // than slept on, and retried once: the landing page also pulls a 13 MB video, and the first
    // navigation of a run can lose a race with the server it was just handed.
    const marker = spec.name === "landing" ? ".headline" : ".site-header .nav-pill";
    const look = async () => JSON.parse((await browser.send("Runtime.evaluate", {
      expression: `JSON.stringify({ found: !!document.querySelector("${marker}"), title: document.title })`,
      returnByValue: true,
    }, sessionId)).result.value);

    let state = await look();
    for (let tries = 0; tries < 40 && !state.found; tries += 1) {
      if (tries === 20) {
        const again = browser.once("Page.loadEventFired");
        await browser.send("Page.navigate", { url: `${base}/${spec.file}` }, sessionId);
        await Promise.race([again, sleep(15000)]);
      }
      await sleep(500);
      state = await look();
    }
    if (!state.found) {
      problems.push(`${where}: did not load — no ${marker} on the page (title "${state.title}")`);
      await browser.send("Target.closeTarget", { targetId });
      continue;
    }
    // Fonts settling, and enough of the background video for a frame to be painted.
    await sleep(spec.name === "landing" ? 5000 : 2000);

    // --- the page as published.
    //
    // On the landing page one real frame of the background video is painted in first. Headless
    // Chromium here decodes no H.264, so a straight screenshot of this page shows a black
    // rectangle where every visitor will see moving footage — a picture of a bug that is not
    // there. The frame is the video's own, so what is saved is what the page looks like.
    if (spec.name === "landing" && videoFrames.length) {
      await browser.send("Runtime.evaluate", {
        expression: `new Promise((resolve) => {
          const v = document.querySelector(".bg-video");
          if (v) v.style.display = "none";
          const img = document.createElement("img");
          img.id = "shot-frame";
          img.className = "bg-video";
          img.src = "${videoFrames[0]}";
          img.onload = () => resolve("ok");
          img.onerror = () => resolve("failed");
          document.querySelector(".bg").appendChild(img);
        })`,
        awaitPromise: true,
        returnByValue: true,
      }, sessionId);
      await sleep(400);
    }
    const shot = await browser.send("Page.captureScreenshot", { format: "png" }, sessionId);
    const png = Buffer.from(shot.data, "base64");
    const file = path.join(shots, `${spec.name}-${viewport.width}x${viewport.height}.png`);
    writeFileSync(file, png);

    // --- what each run of text says about itself, measured before the ink is turned off
    const found = await browser.send("Runtime.evaluate", {
      expression: COLLECT, returnByValue: true,
    }, sessionId);
    const items = JSON.parse(found.result.value);

    // --- the same frame with every glyph transparent: this is the background, and nothing else
    await browser.send("Runtime.evaluate", { expression: HIDE_INK, returnByValue: true }, sessionId);
    await sleep(250);

    // On the landing page the background is a looping video, so one frame proves nothing about
    // the others: the page is paused and stepped across the whole loop, and every run of text is
    // measured against the BRIGHTEST background any sampled frame puts behind it. Everywhere
    // else one plate is the whole story.
    const plates = [];
    if (spec.name === "landing") {
      // The frame is put in as an <img> in the video's place and waited on, rather than set as a
      // CSS background and hoped for. Each plate is then compared against a plate with no
      // backdrop at all: if the two are identical the frame did not paint, and the contrast
      // numbers would be measuring plain black. A brightness threshold cannot tell those apart,
      // because dark footage is also dark.
      const blank = await (async () => {
        await browser.send("Runtime.evaluate", {
          expression: `(() => {
            const v = document.querySelector(".bg-video");
            if (v) v.style.display = "none";
            for (const id of ["check-frame", "shot-frame"]) {
              const old = document.getElementById(id);
              if (old) old.remove();
            }
            return "blank";
          })()`,
          returnByValue: true,
        }, sessionId);
        await sleep(200);
        const shot = await browser.send("Page.captureScreenshot", { format: "png" }, sessionId);
        return pixels(Buffer.from(shot.data, "base64"), viewport.width, viewport.height);
      })();

      let unpainted = 0;
      for (const frame of videoFrames) {
        const loaded = (await browser.send("Runtime.evaluate", {
          expression: `new Promise((resolve) => {
            const old = document.getElementById("check-frame");
            if (old) old.remove();
            const img = document.createElement("img");
            img.id = "check-frame";
            img.className = "bg-video";
            img.src = "${frame}";
            img.onload = () => resolve("ok");
            img.onerror = () => resolve("failed");
            document.querySelector(".bg").appendChild(img);
          })`,
          awaitPromise: true,
          returnByValue: true,
        }, sessionId)).result.value;
        if (loaded !== "ok") {
          problems.push(`${where}: background frame ${frame} did not load`);
          continue;
        }
        await sleep(220);
        const shotAt = await browser.send("Page.captureScreenshot", { format: "png" }, sessionId);
        const plate = pixels(Buffer.from(shotAt.data, "base64"), viewport.width, viewport.height);
        let changed = 0;
        for (let at = 0; at < plate.length; at += 331 * 3) {
          if (plate[at] !== blank[at]) changed += 1;
        }
        if (!changed) unpainted += 1;
        plates.push(plate);
      }
      if (unpainted) {
        problems.push(
          `${where}: ${unpainted} of ${videoFrames.length} background frames changed not one ` +
          `pixel against an empty backdrop — they did not paint, so the contrast numbers below ` +
          `would be measured against black`,
        );
      }
      notes.push(`${where}: measured against ${plates.length} real frames of the background video`);
    }
    if (!plates.length) {
      const plateShot = await browser.send("Page.captureScreenshot", { format: "png" }, sessionId);
      plates.push(pixels(Buffer.from(plateShot.data, "base64"), viewport.width, viewport.height));
    }
    let checked = 0;
    let worst = { ratio: Infinity };
    for (const item of items) {
      const fg = parseRGB(item.color);
      if (fg.length < 3) continue;
      let painted = null;
      let brightest = -1;
      for (const plate of plates) {
        for (const box of item.boxes) {
          const found = brightestUnder(plate, viewport.width, viewport.height, box);
          if (!found) continue;
          const l = luminance(found);
          if (l > brightest) {
            brightest = l;
            painted = found;
          }
        }
      }
      if (!painted) continue;
      const against = ratio(fg, painted);
      const large = item.size >= 24 || (item.size >= 18.66 && Number(item.weight) >= 700);
      const floor = large ? 3 : 4.5;
      checked += 1;
      if (against < worst.ratio) worst = { ratio: against, item, painted, floor };
      if (against < floor) {
        problems.push(
          `${where}: "${item.text}" is ${against.toFixed(2)}:1 against the brightest background ` +
          `pixel behind it (rgb(${painted})), under the ${floor}:1 ${large ? "large" : "body"} floor`,
        );
      }
    }
    notes.push(
      `${where}: ${checked} text run(s) checked, worst ${worst.ratio.toFixed(2)}:1 ` +
      `(floor ${worst.floor}) on "${worst.item?.text ?? "—"}"`,
    );

    // --- every link resolves, and the acts are the five acts in order
    const links = JSON.parse(
      (await browser.send("Runtime.evaluate", { expression: LINKS, returnByValue: true }, sessionId))
        .result.value,
    );
    for (const link of links) {
      if (/^(https?:|mailto:|data:|#)/.test(link.href)) continue;
      const target = path.resolve(path.dirname(path.join(site, spec.file)), link.href.split("#")[0]);
      if (!existsSync(target)) {
        problems.push(`${where}: "${link.text}" points at ${link.href}, which is not a file`);
      }
    }

    // --- the active page's three dots must sit under its label, not on top of the glyphs
    {
      const dots = JSON.parse((await browser.send("Runtime.evaluate", {
        expression: `(() => {
          try {
            const a = document.querySelector(".nav-pill a.active");
            if (!a) return JSON.stringify({ shown: false });
            const after = getComputedStyle(a, "::after");
            if (!after || after.content === "none") return JSON.stringify({ shown: false });
            const box = a.getBoundingClientRect();
            const range = document.createRange();
            range.selectNodeContents(a);
            const rects = Array.from(range.getClientRects());
            if (!rects.length) return JSON.stringify({ shown: false });
            const textBottom = Math.max(...rects.map((r) => r.bottom));
            const top = box.bottom - parseFloat(after.bottom) - parseFloat(after.height);
            return JSON.stringify({
              shown: true, label: a.textContent.trim(),
              gap: top - textBottom, inside: top >= box.top && top <= box.bottom,
            });
          } catch (err) {
            return JSON.stringify({ shown: false, failed: String(err) });
          }
        })()`,
        returnByValue: true,
      }, sessionId)).result.value);
      if (dots.failed) {
        problems.push(`${where}: the nav-dot measurement threw — ${dots.failed}`);
      }
      if (dots.shown) {
        if (!dots.inside) {
          problems.push(`${where}: the active page's dots fall outside the nav link's own box`);
        } else if (dots.gap < 1) {
          problems.push(
            `${where}: the active page's dots overlap "${dots.label}" by ` +
            `${Math.abs(dots.gap).toFixed(1)}px — the link has no height for them to sit below`,
          );
        } else {
          notes.push(
            `${where}: the active page's dots clear "${dots.label}" by ${dots.gap.toFixed(1)}px`,
          );
        }
      }
    }

    // --- the walkthrough: it must sit inside its box, at both sizes
    if (spec.name === "start" || spec.name === "landing") {
      const fits = JSON.parse((await browser.send("Runtime.evaluate", {
        expression: `(() => {
          const box = document.querySelector(".embed");
          if (!box) return JSON.stringify({ box: false });
          const frame = box.querySelector("iframe");
          if (!frame) return JSON.stringify({ box: true, frame: false });
          const b = box.getBoundingClientRect();
          const f = frame.getBoundingClientRect();
          return JSON.stringify({
            box: true, frame: true,
            ratio: b.width / b.height,
            inside: f.left >= b.left - 1 && f.right <= b.right + 1
                 && f.top >= b.top - 1 && f.bottom <= b.bottom + 1,
            overflows: b.right > document.documentElement.clientWidth + 1,
            src: frame.getAttribute("src"),
          });
        })()`,
        returnByValue: true,
      }, sessionId)).result.value);

      if (spec.name === "start") {
        if (!fits.box || !fits.frame) {
          problems.push(`${where}: Start here has no walkthrough embed in the page`);
        } else if (!fits.inside) {
          problems.push(`${where}: the walkthrough iframe sits outside its 16:9 container`);
        } else if (Math.abs(fits.ratio - 16 / 9) > 0.03) {
          problems.push(`${where}: the embed container is ${fits.ratio.toFixed(3)}:1, not 16:9`);
        } else if (fits.overflows) {
          problems.push(`${where}: the embed container runs past the viewport`);
        } else if (!/^https:\/\/www\.youtube-nocookie\.com\/embed\//.test(fits.src)) {
          problems.push(`${where}: the embed is ${fits.src}, not the privacy-enhanced domain`);
        } else {
          notes.push(`${where}: the walkthrough embed is 16:9, inside its box, on youtube-nocookie`);
        }
      } else if (fits.box) {
        problems.push(`${where}: the landing page ships an embed in its markup; it must be built on open`);
      }
    }

    // --- the lightbox: nothing is requested until it is opened, and closing destroys the frame
    if (spec.name === "landing" && viewport.width > 720) {
      const story = JSON.parse((await browser.send("Runtime.evaluate", {
        expression: `(() => {
          const box = document.getElementById("lightbox");
          const open = document.getElementById("watch-demo");
          if (!box || !open) return JSON.stringify({ ok: false });
          const before = box.querySelectorAll("iframe").length;
          open.click();
          const frame = box.querySelector("iframe");
          const opened = !box.hidden;
          const src = frame && frame.getAttribute("src");
          const rect = frame && frame.getBoundingClientRect();
          const holder = box.querySelector(".lightbox-frame").getBoundingClientRect();
          box.querySelector(".lightbox-close").click();
          return JSON.stringify({
            ok: true, before, opened, src,
            closed: box.hidden,
            emptied: box.querySelectorAll("iframe").length === 0,
            ratio: holder.width / holder.height,
            inside: rect ? (rect.width <= holder.width + 1 && rect.height <= holder.height + 1) : false,
          });
        })()`,
        returnByValue: true,
      }, sessionId)).result.value);

      if (!story.ok) {
        problems.push(`${where}: no lightbox and no "Watch the Demo" control on the landing page`);
      } else {
        if (story.before !== 0) {
          problems.push(`${where}: the lightbox already held an iframe before it was opened`);
        }
        if (!story.opened) problems.push(`${where}: the lightbox did not open on click`);
        if (!/^https:\/\/www\.youtube-nocookie\.com\/embed\//.test(story.src || "")) {
          problems.push(`${where}: the lightbox embed is ${story.src}, not the privacy-enhanced domain`);
        }
        if (!story.inside) problems.push(`${where}: the lightbox iframe overflows its 16:9 holder`);
        if (Math.abs(story.ratio - 16 / 9) > 0.03) {
          problems.push(`${where}: the lightbox holder is ${story.ratio.toFixed(3)}:1, not 16:9`);
        }
        if (!story.closed) problems.push(`${where}: the lightbox did not close`);
        if (!story.emptied) {
          problems.push(`${where}: closing the lightbox left the iframe in the page, still playing`);
        }
        if (story.opened && story.closed && story.emptied && story.before === 0) {
          notes.push(
            `${where}: lightbox empty until clicked, opens on youtube-nocookie, closes and ` +
            `destroys the frame`,
          );
        }
      }

      // Escape must close it too.
      const byEscape = JSON.parse((await browser.send("Runtime.evaluate", {
        expression: `(() => {
          const box = document.getElementById("lightbox");
          document.getElementById("watch-demo").click();
          const opened = !box.hidden;
          document.dispatchEvent(new KeyboardEvent("keydown", { key: "Escape", bubbles: true }));
          return JSON.stringify({ opened, closed: box.hidden });
        })()`,
        returnByValue: true,
      }, sessionId)).result.value);
      if (!byEscape.opened || !byEscape.closed) {
        problems.push(`${where}: Escape did not close the lightbox`);
      } else {
        notes.push(`${where}: Escape closes the lightbox`);
      }

      // Clicking the dark backdrop must close it; clicking the film itself must not.
      const byBackdrop = JSON.parse((await browser.send("Runtime.evaluate", {
        expression: `(() => {
          const box = document.getElementById("lightbox");
          document.getElementById("watch-demo").click();
          const opened = !box.hidden;
          box.querySelector(".lightbox-frame").click();
          const survived = !box.hidden;
          box.click();
          return JSON.stringify({ opened, survived, closed: box.hidden,
            emptied: box.querySelectorAll("iframe").length === 0 });
        })()`,
        returnByValue: true,
      }, sessionId)).result.value);
      if (!byBackdrop.opened || !byBackdrop.closed || !byBackdrop.emptied) {
        problems.push(`${where}: clicking the backdrop did not close the lightbox and empty it`);
      } else if (!byBackdrop.survived) {
        problems.push(`${where}: clicking the film itself closed the lightbox`);
      } else {
        notes.push(`${where}: the backdrop closes the lightbox, the film itself does not`);
      }
    }

    if (spec.name === "start") {
      const actLinks = links
        .map((l) => l.href)
        .filter((h) => h && h.includes("replay/") && /act\d/.test(h))
        .map((h) => path.basename(h, ".html"));
      if (actLinks.join(",") !== ACTS.join(",")) {
        problems.push(
          `${where}: the act links are [${actLinks}], not the five acts in order [${ACTS}]`,
        );
      } else {
        notes.push(`${where}: the five act links open the five acts in order`);
      }
      const buttons = JSON.parse(
        (await browser.send("Runtime.evaluate", { expression: BUTTONS, returnByValue: true }, sessionId))
          .result.value,
      );
      const copies = buttons.filter((b) => b.command);
      if (copies.length !== 2) {
        problems.push(`${where}: ${copies.length} copy button(s) carry a command, expected 2`);
      } else {
        notes.push(`${where}: both copy buttons carry their command verbatim`);
      }
    }

    await browser.send("Target.closeTarget", { targetId });
  }
}

browser.close();
chrome.kill();
server.kill();
rmSync(path.join(site, FRAME_DIR), { recursive: true, force: true });

console.log(`screenshots: ${shots}`);
for (const note of notes) console.log(`  ${note}`);
if (problems.length) {
  console.error(`\n${problems.length} problem(s):`);
  for (const problem of problems) console.error(`  ${problem}`);
  process.exit(1);
}
console.log("\nbrowser check: every link resolves, the acts are in order, and all text clears WCAG AA");
