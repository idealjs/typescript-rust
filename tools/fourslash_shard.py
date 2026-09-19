#!/usr/bin/env python3
"""fourslash 用例分片内存护栏运行器。

目的：全量放开用例后出现内存增长/OOM，定位是「全局累积泄露」还是
「特定用例大分配」，且不让宿主进程被 OOM kill。

手段：
1. 测试二进制直跑（绕过 cargo），RUST_TEST_THREADS=1 串行，
   出问题时的「当前用例」从日志可精确定位；
2. preexec 里 setrlimit(RLIMIT_AS)：虚拟内存超限时测试二进制自己
   malloc 失败退出，主脚本与终端不受影响（提前限制）；
3. 每 50ms 采样 VmRSS 记录每片峰值，超过 soft 阈值先记录、超过 hard
   限制由内核兜底；
4. 被限杀的片自动对半二分，最终定位到单个用例；
5. 附带累积性检查：单用例重复 N 次看 RSS 斜率。

用法：
  python3 tools/fourslash_shard.py            # 全量分片 + 汇总
  python3 tools/fourslash_shard.py --bisect   # 只对 oom 片二分
  python3 tools/fourslash_shard.py --probe t,50  # 单用例重复泄露检查
"""

import argparse
import glob
import os
import re
import resource
import subprocess
import sys
import time
from concurrent.futures import ThreadPoolExecutor

BIN_GLOB = "target/debug/deps/fourslash-*"
OUT_DIR = os.environ.get("TSOX_FOURSLASH_OUT", "/tmp/fourslash_shards")
SHARD_SIZE = 200
AS_LIMIT = 6 * 1024**3        # RLIMIT_AS：6GiB 虚拟内存（提前限制）
KILL_RSS = 4 * 1024**3        # 采样软阈值 4GiB：主动 SIGKILL 并记录
POLL_SEC = 0.05


def test_binary():
    override = os.environ.get("TSOX_FOURSLASH_BIN")
    if override:
        return override
    bins = [b for b in glob.glob(BIN_GLOB) if os.access(b, os.X_OK)]
    if not bins:
        sys.exit("找不到测试二进制，先 cargo build -p tsox-lsp --tests")
    return max(bins, key=os.path.getmtime)


def list_tests(binary):
    out = subprocess.run([binary, "--list", "--format", "terse"],
                         capture_output=True, text=True, check=True).stdout
    names = []
    for line in out.splitlines():
        m = re.match(r"^(cases::\S+): test$", line.strip())
        if m:
            names.append(m.group(1))
    return names


def run_shard(binary, names, log_path, as_limit=AS_LIMIT, kill_rss=KILL_RSS):
    """跑一片，返回 (peak_rss, killed, elapsed)。日志含逐用例进度。"""
    start = time.time()

    def preexec():
        resource.setrlimit(resource.RLIMIT_AS, (as_limit, as_limit))

    proc = subprocess.Popen(
        [binary] + names + ["--exact", "--test-threads=1", "--nocapture"],
        stdout=open(log_path, "w"), stderr=subprocess.STDOUT,
        preexec_fn=preexec)
    peak = 0
    killed = False
    while proc.poll() is None:
        try:
            with open(f"/proc/{proc.pid}/status") as fh:
                for line in fh:
                    if line.startswith("VmRSS:"):
                        rss = int(line.split()[1]) * 1024
                        peak = max(peak, rss)
                        if rss > kill_rss:
                            proc.kill()
                            killed = True
                        break
        except (FileNotFoundError, ProcessLookupError):
            break
        time.sleep(POLL_SEC)
    proc.wait()
    killed = killed or proc.returncode in (-9, 134, -6)
    # 栈溢出等中途崩溃：进程异常退出或日志缺 test result 汇总行
    with open(log_path) as fh:
        has_result = "test result:" in fh.read()
    return peak, killed or not has_result, time.time() - start


def last_running_test(log_path):
    """libtest 先打印 `test NAME ... ` 再执行：取最后一条未完结名"""
    cur = None
    pat = re.compile(r"^test (cases::\S+) \.\.\. ")
    try:
        with open(log_path, errors="replace") as fh:
            for line in fh:
                m = pat.match(line)
                if m:
                    cur = m.group(1)
    except FileNotFoundError:
        pass
    return cur


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--bisect", action="store_true", help="只二分此前 oom 的片")
    ap.add_argument("--probe", default=None,
                    help="累积性检查：test子串,次数（如 foo,50）")
    ap.add_argument("--shard-size", type=int, default=SHARD_SIZE)
    ap.add_argument("--parallel", type=int, default=1,
                    help="并行运行的分片进程数（每片内部仍单线程）")
    args = ap.parse_args()

    binary = test_binary()
    os.makedirs(OUT_DIR, exist_ok=True)

    if args.probe:
        name, n = args.probe.rsplit(",", 1)
        names = [t for t in list_tests(binary) if name in t]
        if not names:
            sys.exit(f"未找到用例 {name}")
        peak0, _, _ = run_shard(binary, names[:1],
                                f"{OUT_DIR}/probe_once.log", kill_rss=1 << 60)
        peakn, _, _ = run_shard(binary, names[:1] * int(n),
                                f"{OUT_DIR}/probe_{n}.log", kill_rss=1 << 60)
        verdict = "疑似累积泄露" if peakn > peak0 * 2 + (256 << 20) else "未见累积增长"
        print(f"单次 peak={peak0 >> 20}MiB，{n} 次 peak={peakn >> 20}MiB（{verdict}）")
        return

    list_log = os.path.join(OUT_DIR, "tests.txt")
    if not (os.path.exists(list_log) and os.path.getmtime(list_log) > os.path.getmtime(binary)):
        names = list_tests(binary)
        open(list_log, "w").write("\n".join(names))
    names = open(list_log).read().split()
    print(f"用例 {len(names)} 个，片大小 {args.shard_size}")

    manifest = os.path.join(OUT_DIR, "shards.txt")
    if args.bisect and os.path.exists(manifest):
        shards = [(i, l.strip().split()) for i, l in enumerate(open(manifest))
                  if "oom" in l]
    else:
        shards = [(i, names[i:i + args.shard_size])
                  for i in range(0, len(names), args.shard_size)]
        open(manifest, "w").write("")

    done_shards = set()
    if os.path.exists(manifest):
        for line in open(manifest):
            parts = line.rstrip("\n").split("\t")
            if len(parts) >= 4 and parts[3] == "ok":
                done_shards.add(int(parts[0]))
    mf = open(manifest, "a")
    suspects = []
    pending = [(idx, shard) for idx, shard in shards if idx not in done_shards]

    def run_one(item):
        idx, shard = item
        log = f"{OUT_DIR}/shard_{idx:04d}.log"
        peak, killed, elapsed = run_shard(binary, shard, log)
        return idx, shard, peak, killed, elapsed

    with ThreadPoolExecutor(max_workers=args.parallel) as pool:
        for idx, shard, peak, killed, elapsed in pool.map(run_one, pending):
            status = "oom" if killed else "ok"
            print(f"片 {idx:04d} [{len(shard)} 用例] peak={peak >> 20}MiB "
                  f"{elapsed:.0f}s {status}")
            mf.write(f"{idx}\t{len(shard)}\t{peak >> 20}MiB\t{status}\t"
                     f"{shard[0]}\n")
            mf.flush()
            if killed:
                suspects.append((idx, shard))
    mf.close()

    # 对被限杀的片对半二分，直至单片
    while suspects:
        idx, shard = suspects.pop()
        if len(shard) == 1:
            print(f"定位：{shard[0]}")
            continue
        mid = len(shard) // 2
        for half in (shard[:mid], shard[mid:]):
            log = f"{OUT_DIR}/bisect_{idx}_{len(half)}.log"
            peak, killed, _ = run_shard(binary, half, log)
            if killed:
                print(f"  二分 [{len(half)}] peak={peak >> 20}MiB oom：{half[0]}")
                suspects.append((idx, half))
            else:
                print(f"  二分 [{len(half)}] peak={peak >> 20}MiB ok")


if __name__ == "__main__":
    main()
