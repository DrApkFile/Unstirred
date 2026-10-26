#!/usr/bin/env python3
"""Lays the 32 narration files onto one 120 s track at their film times.

Only for the transcript: the film itself plays the lines as separate <audio> clips, so this file
is a read-back of what the voice says and when, not an input to the render.
"""
import json, subprocess, pathlib, importlib.util, sys

SC = pathlib.Path(__file__).resolve().parent
ROOT = SC.parent                                        # the video/ project root
spec = importlib.util.spec_from_file_location("b", SC / "build_index.py")
b = importlib.util.module_from_spec(spec); spec.loader.exec_module(b)

lines = json.load(open(SC / "narration.json"))
rows = sorted(((k, b.ANCHOR[k]) for k, _s, _a, _t in lines), key=lambda r: r[1])
rate = subprocess.run(["ffprobe", "-v", "error", "-show_entries", "stream=sample_rate",
                       "-of", "default=nw=1:nk=1", str(ROOT / "assets/vo/n01.wav")],
                      capture_output=True, text=True).stdout.strip()

args = ["ffmpeg", "-hide_banner", "-nostats", "-y",
        "-f", "lavfi", "-t", "120", "-i", f"anullsrc=r={rate}:cl=mono"]
parts, labels = [], ["[0]"]
for i, (key, at) in enumerate(rows, start=1):
    args += ["-i", str(ROOT / "assets/vo" / f"{key}.wav")]
    ms = int(round(at * 1000))
    parts.append(f"[{i}]adelay={ms}|{ms}[a{i}]")
    labels.append(f"[a{i}]")
graph = ";".join(parts) + ";" + "".join(labels) + f"amix=inputs={len(rows)+1}:normalize=0:duration=first[out]"
out = ROOT / "assets/vo/narration-full.wav"
args += ["-filter_complex", graph, "-map", "[out]", "-ac", "1", str(out)]
r = subprocess.run(args, capture_output=True, text=True)
if r.returncode:
    sys.exit(r.stderr[-2000:])
print("wrote", out)
