"""Summarize only observed PresentMon data; missing samples remain missing."""
import argparse
import csv
import io
import json
import math
import os
from pathlib import Path
import statistics
from collections import defaultdict


def number(text):
    try:
        value = float(text)
        return value if math.isfinite(value) else None
    except (TypeError, ValueError):
        return None


def distribution(values):
    values = [v for v in values if v is not None and math.isfinite(v) and v > 0]
    if not values:
        return {"count": 0, "median": None, "min": None, "max": None}
    return {"count": len(values), "median": statistics.median(values),
            "min": min(values), "max": max(values)}


def console_rows(samples):
    lines = [row["csv"] for row in samples if "csv" in row]
    start = next((i for i, line in enumerate(lines)
                  if line.startswith("Application,") and "ProcessID" in line), None)
    if start is None:
        return []
    return list(csv.DictReader(io.StringIO("\n".join(lines[start:]))))


def console_fps(rows, intervals=None):
    """One-second windows per swap chain, using source time, not pipe arrival."""
    chains = defaultdict(list)
    for row in rows:
        seconds = source_seconds(row)
        interval = number(row.get("MsBetweenPresents"))
        if seconds is not None and interval is not None and interval > 0:
            chains[row["SwapChainAddress"]].append((seconds, interval))
    values = []
    for frames in chains.values():
        frames.sort()
        at = frames[0][0] + 1.0
        left = right = 0
        while at <= frames[-1][0]:
            while left < len(frames) and frames[left][0] <= at - 1.0:
                left += 1
            while right < len(frames) and frames[right][0] <= at:
                right += 1
            allowed = intervals is None or any(begin + 1 <= at <= end for begin, end in intervals)
            if allowed and right - left >= 2:
                mean = statistics.mean(interval for _, interval in frames[left:right])
                values.append(1000.0 / mean)
            at += 0.25
    return distribution(values)


def foreground_intervals(resources):
    """Conservative intervals bounded by observed foreground samples."""
    intervals = []
    begin = previous = None
    for row in resources:
        at = row["elapsed"]
        visible = row.get("target_foreground") is True
        gap = previous is not None and at - previous > 0.5
        if begin is not None and (not visible or gap):
            intervals.append((begin, previous))
            begin = None
        if visible and begin is None:
            begin = at
        previous = at
    if begin is not None:
        intervals.append((begin, previous))
    return intervals


def source_seconds(row):
    # The probe explicitly requests --qpc_time_ms. Avoid guessing units from
    # TimeInSeconds: v2.6.0 can put milliseconds under that legacy column name.
    qpc_ms = number(row.get("CPUStartQPCTimeInMs"))
    return qpc_ms / 1000 if qpc_ms is not None else None


def resource_summary(samples):
    summary = {}
    for name in ["capture", "reader", "workload", "target"]:
        available = [(r["elapsed"], r[name]) for r in samples if r.get(name)]
        if len(available) < 2:
            continue
        begin, end = available[0], available[-1]
        seconds = end[0] - begin[0]
        cpu = (end[1]["cpu_seconds"] - begin[1]["cpu_seconds"]) / seconds * 100
        summary[name] = {
            "cpu_percent_one_core": cpu,
            "cpu_percent_machine": cpu / (os.cpu_count() or 1),
            "peak_rss_mib": max(v["rss_bytes"] for _, v in available) / 2**20,
            "private_growth_mib": (end[1]["private_bytes"] - begin[1]["private_bytes"]) / 2**20,
        }
    system = [row["system_cpu"] for row in samples if row.get("system_cpu")]
    if len(system) >= 2:
        total = system[-1]["total"] - system[0]["total"]
        idle = system[-1]["idle"] - system[0]["idle"]
        summary["system_cpu_percent"] = 100 * (total - idle) / total if total else None
    return summary


def summarize(result):
    summary = {key: result.get(key) for key in
               ["mode", "pid", "seconds", "error", "capture_exit", "api_version"]}
    summary["resources"] = resource_summary(result["resources"])
    focus = [row["target_foreground"] for row in result["resources"] if "target_foreground" in row]
    summary["target_foreground_percent"] = 100 * sum(focus) / len(focus) if focus else None
    intervals = foreground_intervals(result["resources"]) if focus else None
    if intervals is not None:
        summary["foreground_intervals_seconds"] = intervals
    if result["mode"] == "api":
        chains = [chain for sample in result["samples"] for chain in sample.get("chains", [])
                  if chain["swap_chain_address"] != "0x0" and (chain.get("presented_fps") or 0) > 0]
        summary["swap_chains"] = sorted({row["swap_chain_address"] for row in chains})
        for metric in ["presented_fps", "displayed_fps", "application_fps", "cpu_frame_time"]:
            summary[metric] = distribution([row.get(metric) for row in chains])
            if intervals is not None:
                qualified = [chain.get(metric) for sample in result["samples"]
                    if any(begin + 1 <= sample["elapsed"] <= end for begin, end in intervals)
                    for chain in sample.get("chains", [])
                    if chain["swap_chain_address"] != "0x0" and (chain.get("presented_fps") or 0) > 0]
                summary["foreground_window_" + metric] = distribution(qualified)
        summary["first_valid_sample_seconds"] = next((row["elapsed"] for row in result["samples"]
            if any((chain.get("presented_fps") or 0) > 0 for chain in row.get("chains", []))), None)
    if result["mode"] == "console":
        rows = console_rows(result["samples"])
        target = [row for row in rows if number(row.get("ProcessID")) == result["pid"]]
        summary["frames"] = len(target)
        times = [source_seconds(row) for row in target]
        times = [value for value in times if value is not None]
        summary["source_coverage_seconds"] = max(times) - min(times) if len(times) >= 2 else 0
        summary["other_pid_frames"] = len(rows) - len(target)
        summary["columns"] = list(rows[0]) if rows else []
        summary["presented_fps"] = console_fps(target)
        if intervals is not None:
            absolute = [(begin + result["started_perf_counter"], end + result["started_perf_counter"])
                        for begin, end in intervals]
            summary["foreground_window_presented_fps"] = console_fps(target, absolute)
        summary["swap_chains"] = sorted({row["SwapChainAddress"] for row in target})
        # These remain distinct: do not treat CPU frame duration as displayed FPS.
        for column in ["MsBetweenPresents", "FrameTime", "CPUFrameTime", "DisplayedTime"]:
            summary[column] = distribution([number(row.get(column)) for row in target])
    summary["usable"] = (summary.get("frames", 0) > 0 or
                         summary.get("presented_fps", {}).get("count", 0) > 0)
    fps = summary.get("presented_fps", {})
    if result.get("synthetic", False):
        summary["rate_change_observed"] = ((fps.get("min") or 1000) < 40
                                            and (fps.get("max") or 0) > 50)
    return summary


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("directory", type=Path)
    args = parser.parse_args()
    reports = []
    for file in sorted(args.directory.rglob("result.json")):
        result = summarize(json.loads(file.read_text(encoding="utf-8")))
        reports.append({"path": str(file), **result})
    output = args.directory / "summary.json"
    output.write_text(json.dumps(reports, indent=2, allow_nan=False), encoding="utf-8")
    print(json.dumps(reports, indent=2))


if __name__ == "__main__":
    main()
