// Does any page the gate serves hide text to make itself fit?
//
// Drives a local chromium over the DevTools protocol, loads every page a running gate serves — the
// picker, a live room, each recorded act, each of those again with ?present=1 — at the video's own
// 1920x1080 and at a laptop's 1326x683, and reports, for each: whether the stage outgrew its
// 1080px, whether any box hides what does not fit, and whether any text has left the stage. It
// also reports the smallest type size on the page, which is what ?present=1 raises.
//
// This is NOT a step of scripts/gate.sh: it needs a browser and a node, and this workspace depends
// on neither. It is how the recorded layout check in evidence/ was made, so that run can be
// repeated rather than taken on trust.
//
// Usage: unstirred gate serve --config <file>   # in another terminal
//        node scripts/page-layout-check.mjs http://127.0.0.1:8080 holders-room second-room
// Exit 0 if nothing clips anywhere, 1 otherwise.

import { spawn } from "node:child_process";

const [, , base, firstRoom, secondRoom] = process.argv;
if (!base || !firstRoom) {
  console.error("usage: node scripts/page-layout-check.mjs <gate url> <room> [second room]");
  process.exit(2);
}

const SIZES = [
  [1920, 1080],
  [1326, 683],
];

function pages() {
  const acts = [
    ["act1-rule", firstRoom],
    ["act2-admitted", firstRoom],
    ["act3-refused", firstRoom],
    ["act5-second-room", secondRoom || firstRoom],
  ];
  const out = [
    ["picker", "/"],
    [`live ${firstRoom}`, `/room/${firstRoom}`],
  ];
  for (const [act, room] of acts) {
    out.push([act, `/room/${room}?replay=${act}`]);
  }
  return out.flatMap(([name, path]) => [
    [name, path],
    [`${name} + present`, path + (path.includes("?") ? "&" : "?") + "present=1"],
  ]);
}

const PROBE = `(() => {
  const out = { clipped: [], outside: [], stage: null };
  const stage = document.querySelector(".stage");
  if (!stage) { out.error = "no stage"; return JSON.stringify(out); }
  out.stage = { clientH: stage.clientHeight, overflow: stage.scrollHeight - stage.clientHeight };
  const describe = (el) =>
    el.tagName.toLowerCase() + (el.className ? "." + String(el.className).trim().split(/\\s+/).join(".") : "") +
    " :: " + (el.textContent || "").trim().slice(0, 44);
  stage.querySelectorAll("*").forEach((el) => {
    const style = getComputedStyle(el);
    const hides = (axis) => ["hidden", "clip", "auto", "scroll"].indexOf(style[axis]) >= 0;
    const overY = el.scrollHeight - el.clientHeight;
    const overX = el.scrollWidth - el.clientWidth;
    if ((hides("overflowY") && overY > 1) || (hides("overflowX") && overX > 1)) {
      out.clipped.push(describe(el) + " (" + overY + "px down, " + overX + "px across)");
    }
  });
  const box = stage.getBoundingClientRect();
  stage.querySelectorAll("*").forEach((el) => {
    if (el.children.length || !(el.textContent || "").trim()) return;
    const r = el.getBoundingClientRect();
    if (r.bottom > box.bottom + 1 || r.right > box.right + 1 || r.top < box.top - 1 || r.left < box.left - 1) {
      out.outside.push(describe(el));
    }
  });
  const sizes = [];
  stage.querySelectorAll("*").forEach((el) => {
    if (!el.children.length && (el.textContent || "").trim()) sizes.push(parseFloat(getComputedStyle(el).fontSize));
  });
  out.minFontPx = sizes.length ? Math.min.apply(null, sizes) : null;
  out.scale = Number(getComputedStyle(document.documentElement).getPropertyValue("--scale").trim());
  return JSON.stringify(out);
})()`;

const port = 9444;
const chrome = spawn(
  "chromium",
  [
    "--headless=new",
    `--remote-debugging-port=${port}`,
    "--no-first-run",
    "--no-default-browser-check",
    "--disable-gpu",
    "--hide-scrollbars",
    "about:blank",
  ],
  { stdio: ["ignore", "ignore", "ignore"] }
);

async function debuggerUrl() {
  for (let attempt = 0; attempt < 100; attempt += 1) {
    try {
      const answer = await fetch(`http://127.0.0.1:${port}/json/version`);
      return (await answer.json()).webSocketDebuggerUrl;
    } catch {
      await new Promise((r) => setTimeout(r, 150));
    }
  }
  throw new Error("chromium never opened a debugging port");
}

const socket = new WebSocket(await debuggerUrl());
await new Promise((resolve, reject) => {
  socket.onopen = resolve;
  socket.onerror = reject;
});
let nextId = 1;
const waiting = new Map();
socket.onmessage = (event) => {
  const message = JSON.parse(event.data);
  if (message.id && waiting.has(message.id)) {
    const { resolve, reject } = waiting.get(message.id);
    waiting.delete(message.id);
    message.error ? reject(new Error(JSON.stringify(message.error))) : resolve(message.result);
  }
};
const send = (method, params = {}, sessionId) =>
  new Promise((resolve, reject) => {
    const id = nextId++;
    waiting.set(id, { resolve, reject });
    socket.send(JSON.stringify({ id, method, params, sessionId }));
  });

const { targetId } = await send("Target.createTarget", { url: "about:blank" });
const { sessionId } = await send("Target.attachToTarget", { targetId, flatten: true });
const page = (method, params) => send(method, params, sessionId);
await page("Page.enable");
await page("Runtime.enable");

let bad = 0;
console.log("| Viewport | Page | Stage overflow | Boxes that hide text | Text outside the stage | Smallest type |");
console.log("|---|---|---|---|---|---|");
for (const [width, height] of SIZES) {
  for (const [name, path] of pages()) {
    await page("Emulation.setDeviceMetricsOverride", { width, height, deviceScaleFactor: 1, mobile: false });
    await page("Page.navigate", { url: base.replace(/\/$/, "") + path });
    await new Promise((r) => setTimeout(r, 900));
    const { result } = await page("Runtime.evaluate", { expression: PROBE, returnByValue: true });
    const found = JSON.parse(result.value);
    if (found.error || found.stage.overflow > 0 || found.clipped.length || found.outside.length) {
      bad += 1;
    }
    console.log(
      `| ${width}x${height} | \`${name}\` | ${found.stage ? found.stage.overflow + "px" : "—"} | ` +
        `${found.clipped.length ? found.clipped.join("; ") : "none"} | ` +
        `${found.outside.length ? found.outside.join("; ") : "none"} | ${found.minFontPx}px |`
    );
  }
}
socket.close();
chrome.kill("SIGKILL");
console.log("");
console.log(bad === 0 ? "LAYOUT CHECK: nothing clips, nothing is hidden" : `LAYOUT CHECK FAILED on ${bad} pages`);
process.exit(bad === 0 ? 0 : 1);
