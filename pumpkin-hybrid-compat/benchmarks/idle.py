"""Measure idle Pumpkin instances using the same native binary on Linux."""

import argparse
import json
import os
from pathlib import Path
import platform
import re
import statistics
import subprocess
import time


READY = {
    "vanilla": ("Server is now running",),
    "patchbukkit": ("PatchBukkit background initialization complete", "Successfully enabled."),
    "fabric": ("PatchBukkit background initialization complete", "Successfully enabled.",
               "Teleport Commands registered by Fabric Mixins."),
}
ANSI = re.compile(r"\x1b\[[0-9;]*[mK]")
RATE = re.compile(r"TPS: ([\d.]+) MSPT: ([\d.]+)ms")
PERCENTILES = re.compile(r"Percentiles: P50: ([\d.]+)ms P95: ([\d.]+)ms "
                         r"P99: ([\d.]+)ms\. Sample: (\d+)")


def parse_ticks(text):
    text = ANSI.sub("", text)
    rate = RATE.search(text)
    percentiles = PERCENTILES.search(text)
    if not rate or not percentiles:
        return None
    tps, mspt = map(float, rate.groups())
    p50, p95, p99 = map(float, percentiles.groups()[:3])
    return {"tps": tps, "mspt": mspt, "p50_ms": p50, "p95_ms": p95,
            "p99_ms": p99, "ticks": int(percentiles[4])}


def wait_for(process, log, search, seconds, start_at=0):
    deadline = time.monotonic() + seconds
    with log.open("rb") as stream:
        while time.monotonic() < deadline:
            if process.poll() is not None:
                raise RuntimeError(f"Pumpkin exited with {process.returncode}. Log: {log}")
            stream.seek(start_at)
            result = search(stream.read().decode("utf-8", errors="replace"))
            if result:
                return result
            time.sleep(0.1)
    raise RuntimeError(f"Timed out waiting for Pumpkin. Log: {log}")


def memory_kib(pid):
    fields = {}
    for line in Path(f"/proc/{pid}/status").read_text().splitlines():
        name, _, value = line.partition(":")
        if name in ("VmRSS", "VmHWM"):
            fields[name] = int(value.split()[0])
    return fields["VmRSS"], fields["VmHWM"]


def cpu_ticks(pid):
    fields = Path(f"/proc/{pid}/stat").read_text().rsplit(")", 1)[1].split()
    # The fields following the process name start at field 3. CPU time is 14 and 15.
    return int(fields[11]) + int(fields[12])


def send(process, command):
    process.stdin.write(command + "\n")
    process.stdin.flush()


def stop(process):
    if process.poll() is not None:
        return
    try:
        send(process, "stop")
        process.wait(timeout=25)
    except (BrokenPipeError, subprocess.TimeoutExpired):
        process.terminate()
        try:
            process.wait(timeout=10)
        except subprocess.TimeoutExpired:
            process.kill()
            process.wait()


def measure(binary, profile, instance, log, settle, seconds):
    with log.open("xb") as stdout:
        start = time.monotonic()
        with subprocess.Popen([binary], cwd=instance, text=True, stdin=subprocess.PIPE,
                              stdout=stdout, stderr=subprocess.STDOUT,
                              env={**os.environ, "RUST_LOG": "info"}) as process:
            try:
                wait_for(process, log, lambda text: all(marker in text for marker in READY[profile]), 180)
                ready_time = time.monotonic() - start
                time.sleep(settle)
                rss_samples = []
                start = time.monotonic()
                cpu_before = cpu_ticks(process.pid)
                for second in range(1, seconds + 1):
                    time.sleep(max(0, start + second - time.monotonic()))
                    rss, peak = memory_kib(process.pid)
                    rss_samples.append(rss)
                elapsed = time.monotonic() - start
                cpu_seconds = (cpu_ticks(process.pid) - cpu_before) / os.sysconf("SC_CLK_TCK")
                offset = log.stat().st_size
                send(process, "tps")
                send(process, "tick query")
                ticks = wait_for(process, log, parse_ticks, 10, offset)
                result = {**ticks, "profile": profile, "ready_seconds": ready_time,
                          "median_rss_mib": statistics.median(rss_samples) / 1024,
                          "peak_rss_mib": peak / 1024,
                          "cpu_percent_of_one_core": 100 * cpu_seconds / elapsed}
            finally:
                stop(process)
            if process.returncode != 0:
                raise RuntimeError(f"Pumpkin stopped with {process.returncode}. Log: {log}")
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("binary", type=Path)
    parser.add_argument("--logs", type=Path, required=True)
    for profile in READY:
        parser.add_argument("--" + profile, type=Path)
    parser.add_argument("--settle", type=int, default=15)
    parser.add_argument("--seconds", type=int, default=30)
    parser.add_argument("--rounds", type=int, default=3)
    options = parser.parse_args()
    profiles = [(name, getattr(options, name)) for name in READY if getattr(options, name)]
    if not profiles or options.settle < 0 or options.seconds < 1 or options.rounds < 1:
        parser.error("Specify at least one profile, positive seconds/rounds and a nonnegative settle")
    if len({instance.resolve() for _, instance in profiles}) != len(profiles):
        parser.error("Profiles require separate instance directories")
    options.logs.mkdir(parents=True, exist_ok=True)
    binary = options.binary.resolve(strict=True)
    records = []
    for round_number in range(options.rounds):
        offset = round_number % len(profiles)
        for name, instance in profiles[offset:] + profiles[:offset]:
            log = options.logs / f"{round_number + 1}-{name}.log"
            result = measure(binary, name, instance, log, options.settle, options.seconds)
            records.append({**result, "round": round_number + 1})
    print(json.dumps({"linux": platform.release(), "seconds": options.seconds,
                      "settle": options.settle, "records": records}, indent=2))


if __name__ == "__main__":
    main()
