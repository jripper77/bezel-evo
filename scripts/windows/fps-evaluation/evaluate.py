"""Windows-only, isolated PresentMon 2.6 evaluation; never installs a service.

Uses the matching extracted SDK header and DLL. Outputs raw observations, not
a claim of benchmark accuracy. A D3D11 workload is optional when --pid is given.
"""
import argparse
import ctypes as ct
import json
import math
import os
from pathlib import Path
import re
import subprocess
import threading
import time
from types import SimpleNamespace
import uuid


class QueryElement(ct.Structure):
    _fields_ = [("metric", ct.c_int), ("stat", ct.c_int),
                ("device", ct.c_uint32), ("index", ct.c_uint32),
                ("offset", ct.c_uint64), ("size", ct.c_uint64)]


class Version(ct.Structure):
    _fields_ = [("major", ct.c_uint16), ("minor", ct.c_uint16),
                ("patch", ct.c_uint16), ("tag", ct.c_char * 22),
                ("hash", ct.c_char * 8), ("config", ct.c_char * 4)]


def enum_values(header, enum):
    """Read integer enum IDs from the matching official SDK, never from main."""
    text = re.sub(r"//[^\n]*|/\*.*?\*/", "", header, flags=re.S)
    body = re.search(r"enum\s+" + re.escape(enum) + r"\s*\{([^}]+)\}", text).group(1)
    result, value = {}, 0
    for item in body.split(","):
        if not item.strip():
            continue
        parts = item.strip().split("=")
        if len(parts) == 2:
            value = int(parts[1].strip(), 0)
        result[parts[0].strip()] = value
        value += 1
    return result


def clean_number(value):
    return value if math.isfinite(value) else None


class Api:
    def __init__(self, directory, header, pipe):
        self.session = ct.c_void_p()
        self.query = ct.c_void_p()
        self.tracked = None
        self.directory_guard = os.add_dll_directory(str(directory))
        self.dll = ct.CDLL(str(directory / "PresentMonAPI2.dll"))
        self.status = {v: k for k, v in enum_values(header, "PM_STATUS").items()}
        self.metrics = enum_values(header, "PM_METRIC")
        self.stats = enum_values(header, "PM_STAT")
        signatures = {
            "pmGetApiVersion": [ct.POINTER(Version)],
            "pmOpenSessionWithPipe": [ct.POINTER(ct.c_void_p), ct.c_char_p],
            "pmCloseSession": [ct.c_void_p],
            "pmStartTrackingProcess": [ct.c_void_p, ct.c_uint32],
            "pmStopTrackingProcess": [ct.c_void_p, ct.c_uint32],
            "pmSetEtwFlushPeriod": [ct.c_void_p, ct.c_uint32],
            "pmRegisterDynamicQuery": [ct.c_void_p, ct.POINTER(ct.c_void_p),
                ct.POINTER(QueryElement), ct.c_uint64, ct.c_double, ct.c_double],
            "pmPollDynamicQuery": [ct.c_void_p, ct.c_uint32, ct.POINTER(ct.c_ubyte),
                ct.POINTER(ct.c_uint32)],
            "pmFreeDynamicQuery": [ct.c_void_p],
        }
        for name, args in signatures.items():
            func = getattr(self.dll, name)
            func.argtypes, func.restype = args, ct.c_int
        version = Version()
        try:
            self.call("pmGetApiVersion", ct.byref(version))
            self.version = f"{version.major}.{version.minor}.{version.patch}"
            self.call("pmOpenSessionWithPipe", ct.byref(self.session), pipe.encode())
        except Exception:
            if self.session:
                self.dll.pmCloseSession(self.session)
            self.directory_guard.close()
            raise

    def call(self, name, *args):
        code = getattr(self.dll, name)(*args)
        if code:
            raise RuntimeError(f"{name}: {self.status.get(code, code)}")

    def track(self, pid):
        self.call("pmSetEtwFlushPeriod", self.session, 100)
        self.call("pmStartTrackingProcess", self.session, pid)
        self.tracked = pid
        self.names = ["SWAP_CHAIN_ADDRESS", "PRESENTED_FPS", "DISPLAYED_FPS",
                      "APPLICATION_FPS", "CPU_FRAME_TIME"]
        self.elements = (QueryElement * len(self.names))(*[
            QueryElement(self.metrics["PM_METRIC_" + name],
                         self.stats["PM_STAT_MID_POINT" if i == 0 else "PM_STAT_AVG"],
                         0, 0, 0, 0) for i, name in enumerate(self.names)])
        self.call("pmRegisterDynamicQuery", self.session, ct.byref(self.query),
                  self.elements, len(self.names), 1000.0, 0.0)
        self.stride = max(e.offset + e.size for e in self.elements)
        if self.stride > 4096 or any(e.size != 8 for e in self.elements):
            raise RuntimeError("Unexpected metric ABI; refusing to read query blobs")
        self.blob = (ct.c_ubyte * (self.stride * 64))()

    def poll(self):
        count = ct.c_uint32(64)
        self.call("pmPollDynamicQuery", self.query, self.tracked,
                  self.blob, ct.byref(count))
        rows = []
        if count.value > 64:
            raise RuntimeError("Swap-chain count exceeded the supplied capacity")
        for i in range(count.value):
            row = {}
            for name, element in zip(self.names, self.elements):
                address = ct.addressof(self.blob) + i * self.stride + element.offset
                row[name.lower()] = (hex(ct.c_uint64.from_address(address).value)
                    if name == "SWAP_CHAIN_ADDRESS" else clean_number(ct.c_double.from_address(address).value))
            rows.append(row)
        return rows

    def close(self):
        if self.query:
            self.dll.pmFreeDynamicQuery(self.query)
        if self.tracked is not None:
            self.dll.pmStopTrackingProcess(self.session, self.tracked)
        if self.session:
            self.dll.pmCloseSession(self.session)
        self.directory_guard.close()


class ProcessMemory(ct.Structure):
    _fields_ = [("cb", ct.c_uint32), ("faults", ct.c_uint32)] + [
        (name, ct.c_size_t) for name in ["peak_rss", "rss", "peak_paged", "paged",
        "peak_nonpaged", "nonpaged", "pagefile", "peak_pagefile", "private"]]


def resources(process):
    kernel = ct.WinDLL("kernel32", use_last_error=True)
    kernel.GetProcessTimes.argtypes = [ct.c_void_p] + [ct.POINTER(ct.c_uint64)] * 4
    memory = ct.WinDLL("psapi", use_last_error=True)
    memory.GetProcessMemoryInfo.argtypes = [ct.c_void_p, ct.POINTER(ProcessMemory), ct.c_uint32]
    creation, exit_time, system, user = (ct.c_uint64() for _ in range(4))
    counters = ProcessMemory()
    counters.cb = ct.sizeof(counters)
    handle = int(process._handle)
    if not kernel.GetProcessTimes(handle, ct.byref(creation), ct.byref(exit_time),
                                  ct.byref(system), ct.byref(user)):
        return None
    if not memory.GetProcessMemoryInfo(handle, ct.byref(counters), ct.sizeof(counters)):
        return None
    return {"cpu_seconds": (system.value + user.value) / 1e7,
            "rss_bytes": counters.rss, "private_bytes": counters.private}


def system_cpu():
    kernel = ct.WinDLL("kernel32", use_last_error=True)
    kernel.GetSystemTimes.argtypes = [ct.POINTER(ct.c_uint64)] * 3
    idle, system, user = (ct.c_uint64() for _ in range(3))
    if not kernel.GetSystemTimes(ct.byref(idle), ct.byref(system), ct.byref(user)):
        return None
    return {"idle": idle.value, "total": system.value + user.value}


def target_is_foreground(pid):
    user = ct.WinDLL("user32", use_last_error=True)
    user.GetForegroundWindow.restype = ct.c_void_p
    user.GetWindowThreadProcessId.argtypes = [ct.c_void_p, ct.POINTER(ct.c_uint32)]
    owner = ct.c_uint32()
    user.GetWindowThreadProcessId(user.GetForegroundWindow(), ct.byref(owner))
    return owner.value == pid


def stop_owned(process, graceful=False):
    if process is None or process.poll() is not None:
        return "already-exited"
    if graceful and process.stdin:
        try:
            process.stdin.write("%ping\n%quit\n")
            process.stdin.flush()
        except OSError:
            pass
    try:
        process.wait(timeout=10)
        return "graceful"
    except subprocess.TimeoutExpired:
        process.terminate()  # Only our own disposable probe child, never Bezel/the game.
        process.wait(timeout=10)
        return "terminated"


def run_once(args, mode, output):
    output.mkdir(parents=True, exist_ok=False)
    suffix = uuid.uuid4().hex[:10]
    session_name = "BezelFpsEval-" + suffix
    pipe = "\\\\.\\pipe\\" + session_name
    service_dir = args.extracted / "Intel" / "PresentMonSharedService"
    workload = capture = api = target = None
    result = {"mode": mode, "seconds": args.seconds, "pid": args.pid,
              "synthetic": args.pid is None, "session_name": session_name,
              "samples": [], "resources": []}
    handles = []
    try:
        if not args.pid:
            work_out = open(output / "workload.json", "w", encoding="utf-8")
            handles.append(work_out)
            workload = subprocess.Popen([str(args.workload), str(args.seconds + 8)],
                stdout=work_out, creationflags=subprocess.CREATE_NO_WINDOW)
            result["pid"] = workload.pid
        pid = result["pid"]
        if args.pid:
            kernel = ct.WinDLL("kernel32", use_last_error=True)
            kernel.OpenProcess.argtypes = [ct.c_uint32, ct.c_int, ct.c_uint32]
            kernel.OpenProcess.restype = ct.c_void_p
            handle = kernel.OpenProcess(0x1000, 0, pid)  # Query counters only.
            if handle:
                target = SimpleNamespace(_handle=handle)
            else:
                result["target_counters_error"] = ct.get_last_error()
        err = open(output / "capture.stderr.log", "w", encoding="utf-8")
        handles.append(err)
        stdout = open(output / "capture.stdout.log", "w", encoding="utf-8")
        handles.append(stdout)
        if mode == "api":
            command = [str(service_dir / "PresentMonService.exe"),
                "--etw-session-name", session_name, "--control-pipe", pipe,
                "--shm-name-prefix", "Local\\" + session_name,
                "--enable-test-control", "--enable-stdio-log",
                "--disable-colorized-stdio-log", "--disable-ipc-log",
                "--timed-stop", str(int((args.seconds + 25) * 1000))]
            if args.connect:
                connection = json.loads(args.connect.read_text(encoding="utf-8"))
                pipe = connection["pipe"]
                if not re.fullmatch(r"\\\\\.\\pipe\\BezelFpsEval-[0-9a-f]+", pipe):
                    raise ValueError("Not an evaluation service pipe")
                result["session_name"] = pipe.rsplit("\\", 1)[-1]
                result["external_service_pid"] = connection["pid"]
            else:
                service_temp = output / "service-temp"
                service_temp.mkdir()
                capture = subprocess.Popen(command, stdin=subprocess.PIPE, stdout=stdout,
                    stderr=err, text=True, creationflags=subprocess.CREATE_NO_WINDOW,
                    env={**os.environ, "TEMP": str(service_temp), "TMP": str(service_temp)})
            header = (args.extracted / "Intel/PresentMon/SDK/PresentMonAPI.h").read_text(encoding="utf-8-sig")
            for attempt in range(30):
                if capture and capture.poll() is not None:
                    raise RuntimeError(f"Service exited with {capture.returncode}")
                try:
                    api = Api(service_dir, header, pipe)
                    break
                except RuntimeError:
                    if attempt == 29:
                        raise
                    time.sleep(0.25)
            result["api_version"] = api.version
            result["etw_flush_ms"] = 100
            api.track(pid)
        elif mode == "console":
            command = [str(args.console), "--process_id", str(pid),
                "--session_name", session_name, "--output_stdout", "--no_console_stats",
                "--qpc_time_ms", "--timed", str(int(args.seconds)), "--terminate_after_timed"]
            if args.track_frame_type:
                command.append("--track_frame_type")
            result["track_frame_type"] = args.track_frame_type
            capture = subprocess.Popen(command, stdout=subprocess.PIPE, stderr=err,
                text=True, creationflags=subprocess.CREATE_NO_WINDOW)
            # Timestamp at the reader, not when the UI later drains a queue.
            def read_lines():
                for line in capture.stdout:
                    stdout.write(line)
                    result["samples"].append({"arrived": time.perf_counter(), "csv": line.rstrip()})
            reader = threading.Thread(target=read_lines)
            reader.start()
        elif mode != "baseline":
            raise ValueError(mode)
        start = time.perf_counter()
        result["started_perf_counter"] = start
        while time.perf_counter() - start < args.seconds:
            elapsed = time.perf_counter() - start
            if api:
                result["samples"].append({"elapsed": elapsed, "chains": api.poll()})
            processes = [("workload", workload), ("capture", capture),
                         ("target", target),
                         ("reader", SimpleNamespace(_handle=-1))]
            result["resources"].append({"elapsed": elapsed, **{
                name: resources(proc) for name, proc in processes if proc},
                "system_cpu": system_cpu(), "target_foreground": target_is_foreground(pid)})
            if capture and capture.poll() is not None:
                result["capture_exit"] = capture.returncode
                break
            time.sleep(0.25)
        if mode == "console":
            capture.wait(timeout=10)
            reader.join(timeout=5)
            if reader.is_alive():
                raise RuntimeError("Console reader did not finish")
        result["capture_exit"] = capture.poll() if capture else None
        if capture and capture.returncode not in (None, 0):
            raise RuntimeError(f"Capture exited with {capture.returncode}; see capture.stderr.log")
        if mode == "api" and not any((chain.get("presented_fps") or 0) > 0
                for sample in result["samples"] for chain in sample.get("chains", [])):
            raise RuntimeError("API returned no positive frame-rate samples")
        if mode == "console" and len(result["samples"]) < 2:
            raise RuntimeError("Console returned no frame rows")
        if workload and workload.poll() is not None:
            raise RuntimeError(f"Workload exited prematurely: {workload.returncode}")
    except Exception as exc:
        result["error"] = str(exc)
    finally:
        if api:
            api.close()
        result["capture_stop"] = stop_owned(capture, graceful=mode == "api")
        result["workload_stop"] = stop_owned(workload)
        result["workload_exit"] = workload.returncode if workload else None
        if target:
            kernel = ct.WinDLL("kernel32", use_last_error=True)
            kernel.CloseHandle.argtypes = [ct.c_void_p]
            kernel.CloseHandle(target._handle)
        for handle in handles:
            handle.close()
        result["capture_exit"] = capture.returncode if capture else None
        if capture and capture.returncode != 0:
            result.setdefault("error", f"Capture cleanup exited with {capture.returncode}; see capture.stderr.log")
        (output / "result.json").write_text(json.dumps(result, indent=2, allow_nan=False), encoding="utf-8")
    return {"mode": mode, "output": str(output), "error": result.get("error"),
            "sample_count": len(result["samples"]), "capture_exit": result["capture_exit"]}


def serve(args):
    """An elevated, time-limited collector for a separate normal-user client."""
    directory = args.output / "server"
    directory.mkdir(parents=True, exist_ok=False)
    suffix = uuid.uuid4().hex[:10]
    name = "BezelFpsEval-" + suffix
    pipe = "\\\\.\\pipe\\" + name
    connection = args.output / "connection.json"
    stop = args.output / "connection.stop"
    service = args.extracted / "Intel/PresentMonSharedService/PresentMonService.exe"
    observations = []
    with open(directory / "stdout.log", "w", encoding="utf-8") as out, \
         open(directory / "stderr.log", "w", encoding="utf-8") as err:
        process = subprocess.Popen([str(service), "--etw-session-name", name,
            "--control-pipe", pipe, "--shm-name-prefix", "Local\\" + name,
            "--enable-test-control", "--enable-stdio-log", "--disable-colorized-stdio-log",
            "--disable-ipc-log", "--timed-stop", "120000"], stdin=subprocess.PIPE,
            stdout=out, stderr=err, text=True, creationflags=subprocess.CREATE_NO_WINDOW)
        try:
            connection.write_text(json.dumps({"pipe": pipe, "pid": process.pid}), encoding="utf-8")
            start = time.perf_counter()
            while time.perf_counter() - start < 110 and not stop.exists():
                observations.append({"elapsed": time.perf_counter() - start,
                                     "capture": resources(process)})
                if process.poll() is not None:
                    break
                time.sleep(0.25)
        finally:
            how = stop_owned(process, graceful=True)
            (directory / "resources.json").write_text(json.dumps({
                "resources": observations, "started_perf_counter": start,
                "exit": process.returncode, "stop": how}, indent=2), encoding="utf-8")
        return process.returncode


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--extracted", type=Path, required=True)
    parser.add_argument("--console", type=Path, required=True)
    parser.add_argument("--workload", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--pid", type=int)
    parser.add_argument("--serve", action="store_true")
    parser.add_argument("--connect", type=Path)
    parser.add_argument("--seconds", type=float, default=30)
    parser.add_argument("--repetitions", type=int, default=1)
    parser.add_argument("--modes", default="baseline,console,api")
    parser.add_argument("--track-frame-type", action="store_true",
                        help="Enable console frame-type instrumentation for a bounded diagnostic run")
    args = parser.parse_args()
    if not 5 <= args.seconds <= 600 or not 1 <= args.repetitions <= 10:
        parser.error("Use 5..600 seconds and 1..10 repetitions")
    if os.name != "nt":
        parser.error("Windows only")
    args.output.mkdir(parents=True, exist_ok=True)
    if args.serve:
        raise SystemExit(serve(args))
    if args.connect and (args.modes != "api" or args.repetitions != 1):
        parser.error("External evaluation service requires --modes api --repetitions 1")
    try:
        for repetition in range(args.repetitions):
            for mode in args.modes.split(","):
                summary = run_once(args, mode, args.output / f"{repetition + 1}-{mode}")
                print(json.dumps(summary), flush=True)
                if summary["error"]:
                    raise SystemExit(1)
    finally:
        if args.connect:
            args.connect.with_suffix(".stop").write_text("client complete\n", encoding="utf-8")


if __name__ == "__main__":
    main()
