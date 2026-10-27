#!/usr/bin/env python3
"""Writes video/index.html: the scene mounts, the one caption track and the two sound tracks.

Caption text is the narration line word for word and starts on the same second as its audio, so
the words on screen and the words in the ear cannot drift. The music lane is derived from where
the voice actually is, rather than written by hand.
"""
import json, subprocess, pathlib, html, sys

ROOT = pathlib.Path(__file__).resolve().parent.parent   # the video/ project root
VO = ROOT / "assets/vo"
TOTAL = 90.0

SCENES = [
    ("s1-hook",       0.0,   5.0),
    ("s2-install",    5.0,  10.0),
    ("s3-door",      15.0,  10.0),
    ("s4-knock",     25.0,  22.0),
    ("s5-newcomer",  47.0,  11.0),
    ("s6-code",      58.0,  12.0),
    ("s7-uses",      70.0,   7.0),
    ("s8-tryit",     77.0,   8.0),
    ("s9-close",     85.0,   5.0),
]

ANCHOR = {
    "n01":  0.25, "n02":  2.75,
    "n03":  5.5, "n04":  8.4, "n05": 12.0,
    "n06": 15.5, "n07": 20.0,
    "n08": 25.4, "n09": 30.4, "n10": 32.0, "n11": 34.8,
    "n12": 37.8, "n13": 41.6, "n14": 43.9,
    "n15": 48.0, "n16": 53.6,
    "n17": 58.5, "n18": 64.6,
    "n19": 70.6,
    "n20": 77.5, "n21": 81.6,
    "n22": 85.3, "n23": 87.6,
}

MUSIC_BED = 0.30      # between lines
MUSIC_DUCK = 0.11     # while the voice speaks
RAMP = 0.2            # seconds each side of a duck
MUSIC_LEN = 105.038   # assets/music.mp3, as ffprobe reads it
FADE_IN = 1.5
FADE_OUT = 3.0        # the tail the bed fades over


def wav_duration(path):
    out = subprocess.run(
        ["ffprobe", "-v", "error", "-show_entries", "format=duration",
         "-of", "default=nw=1:nk=1", str(path)],
        capture_output=True, text=True, check=True)
    return round(float(out.stdout.strip()), 3)


def lane(points):
    pts = ",".join('{"t":%g,"v":%g}' % (round(t, 3), round(v, 3)) for t, v in points)
    return '{"version":1,"lanes":[{"target":"volume","points":[%s]}]}' % pts


def main():
    lines = json.load(open(pathlib.Path(__file__).with_name("narration.json")))
    rows = []
    for key, scene, _planned, text in lines:
        wav = VO / f"{key}.wav"
        if not wav.exists():
            sys.exit(f"missing narration audio: {wav}")
        rows.append({"id": key, "scene": scene, "text": text,
                     "at": ANCHOR[key], "dur": wav_duration(wav)})
    rows.sort(key=lambda r: r["at"])

    # every line must finish before the next one starts, and inside its own scene
    scene_end = {name: start + dur for name, start, dur in SCENES}
    for i, r in enumerate(rows):
        end = r["at"] + r["dur"]
        last = i + 1 == len(rows)
        nxt = TOTAL if last else rows[i + 1]["at"]
        if end > (nxt if last else nxt - 0.12):
            sys.exit(f"{r['id']} ends {end:.2f} but {rows[i+1]['id'] if i+1<len(rows) else 'the film'} "
                     f"starts {nxt:.2f}: shorten the line or move the anchor")
        if end > scene_end[r["scene"]] + 0.9:
            sys.exit(f"{r['id']} ends {end:.2f}, past its scene {r['scene']} ({scene_end[r['scene']]:.2f})")

    # ---- caption track: each caption holds until the next line speaks
    captions = []
    for i, r in enumerate(rows):
        until = rows[i + 1]["at"] if i + 1 < len(rows) else TOTAL
        captions.append((r["id"], r["at"], round(until - r["at"], 3), r["text"]))

    # ---- music lane, derived from where the voice is
    speaking = [(r["at"], r["at"] + r["dur"]) for r in rows]
    merged = []
    for a, b in speaking:
        if merged and a - merged[-1][1] < 0.9:
            merged[-1][1] = b
        else:
            merged.append([a, b])

    def bed_points(clip_start, clip_dur, fade_in, fade_out_from, tail):
        """One volume lane in clip-local seconds: up to the bed, ducked under every line the
        voice speaks inside this clip, then down to nothing over the clip's own tail."""
        pts = [(0.0, 0.0), (fade_in, MUSIC_BED)]
        level = MUSIC_BED
        for a, b in merged:
            la, lb = a - clip_start, b - clip_start
            if lb <= fade_in or la >= fade_out_from:
                continue
            la = max(la, fade_in + 0.05)
            lb = min(lb, fade_out_from)
            pts += [(max(la - RAMP, pts[-1][0] + 0.02), MUSIC_BED), (la, MUSIC_DUCK)]
            if lb + RAMP < fade_out_from:
                pts += [(lb, MUSIC_DUCK), (lb + RAMP, MUSIC_BED)]
                level = MUSIC_BED
            else:
                pts += [(lb, MUSIC_DUCK)]
                level = MUSIC_DUCK
        pts.append((fade_out_from, level))
        pts.append((tail, 0.0))
        out = []
        for t, v in pts:
            t = min(round(t, 3), clip_dur)
            if out and t <= out[-1][0]:
                continue
            out.append((t, v))
        return out

    # The track is 105.0 s and the film is 90 s, so one clip covers it: it is positioned so the
    # track's own ending lands on the film's last frame, and the 2:00 cut's crossfade is gone
    # with the second clip it existed to join.
    media_start = round(MUSIC_LEN - TOTAL, 3)
    lane_points = lane(bed_points(0.0, TOTAL, FADE_IN, TOTAL - FADE_OUT, TOTAL))

    scene_html = "\n".join(f'''      <div
        class="clip"
        id="scene-{name}"{'' if name == 's1-hook' else chr(10) + '        data-hold'}
        data-composition-id="{name}"
        data-composition-src="compositions/{name}.html"
        data-start="{start:g}"
        data-duration="{dur:g}"
        data-track-index="0"
        data-width="1920"
        data-height="1080"
      ></div>''' for name, start, dur in SCENES)

    caption_html = "\n".join(
        f'''      <div class="caption clip" id="caption-{cid}" data-start="{at:g}" data-duration="{dur:g}" data-track-index="9">
        <span>{html.escape(text)}</span>
      </div>''' for cid, at, dur, text in captions)

    voice_html = "\n".join(
        f'''      <audio id="vo-{r['id']}" src="assets/vo/{r['id']}.wav" data-start="{r['at']:g}" data-duration="{r['dur']:g}" data-track-index="11" data-volume="1"></audio>'''
        for r in rows)

    music_html = f'''      <audio
        id="music"
        src="assets/music.mp3"
        data-start="0"
        data-duration="{TOTAL:g}"
        data-media-start="{media_start:g}"
        data-track-index="12"
        data-automation='{lane_points}'
      ></audio>'''

    doc = TEMPLATE.format(total=f"{TOTAL:g}", scenes=scene_html, captions=caption_html,
                          voice=voice_html, music=music_html)
    (ROOT / "index.html").write_text(doc)

    print(f"{len(rows)} narration lines, {rows[-1]['at'] + rows[-1]['dur']:.2f} s to the last word")
    print(f"voice covers {sum(r['dur'] for r in rows):.1f} s of {TOTAL:g} s")
    for r in rows:
        print(f"  {r['id']}  {r['at']:6.2f} +{r['dur']:5.2f}  {r['text']}")


TEMPLATE = '''<!doctype html>
<html lang="en">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=1920, height=1080" />
    <title>Unstirred — demo</title>
    <script src="https://cdn.jsdelivr.net/npm/gsap@3.14.2/dist/gsap.min.js"></script>
    <link rel="stylesheet" href="assets/film.css" />
    <script src="assets/film.js"></script>
    <style>
      #root {{
        position: relative;
        width: 100%;
        height: 100%;
        overflow: hidden;
        background: var(--paper);
      }}

      /* The one caption track. Each caption is its narration line word for word and starts on the
         same second as that line's audio, so the words on screen and the words in the ear are the
         same sentence; a caption holds until the next one speaks, so text never leaves the frame.
         Captions live here rather than in the scenes so the whole pass reads in one place, and so
         nothing below y 964 belongs to a scene. */
      .caption {{
        position: absolute;
        left: 0;
        right: 0;
        top: 1000px;
        height: 64px;
        display: flex;
        align-items: center;
        justify-content: center;
        pointer-events: none;
      }}

      .caption span {{
        display: block;
        max-width: 1500px;
        text-align: center;
        font-family: var(--sans);
        font-size: 30px;
        line-height: 1.2;
        font-weight: 500;
        letter-spacing: -0.005em;
        color: var(--ink);
      }}
    </style>
  </head>
  <body>
    <div
      id="root"
      data-composition-id="main"
      data-start="0"
      data-duration="{total}"
      data-width="1920"
      data-height="1080"
    >
{scenes}

{captions}

      <!-- The voice: Kokoro-82M (af_heart, speed 1.0) run locally through `hyperframes tts`, one
           file per line, placed at the second its caption appears. -->
{voice}

      <!-- The bed. assets/music.mp3 is 105.0 s and the film is 90 s, so one clip covers the
           whole film: it starts 15.038 s into the file so the track's own ending lands on the
           last frame. The lane holds the bed at 0.30 between lines and ducks it to 0.11 while
           the voice speaks, with 0.2 s ramps, and fades out over the closing 3 s. -->
{music}
    </div>
    <script>
      const tl = gsap.timeline({{ paused: true }});
      document.querySelectorAll(".caption").forEach((caption) => {{
        const at = Number(caption.getAttribute("data-start"));
        tl.fromTo(
          caption.firstElementChild,
          {{ opacity: 0, y: 12 }},
          {{ opacity: 1, y: 0, duration: 0.32, ease: "power2.out" }},
          at
        );
      }});
      window.__timelines["main"] = tl;
      tl.seek(0);
    </script>
  </body>
</html>
'''

if __name__ == "__main__":
    main()
