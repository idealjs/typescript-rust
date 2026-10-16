# xtask 常驻 worker 致命退出问题

> 版本:v1.0 · 2026-10-15(主体已修复,残留 1 例)

## 后果

池化模式(`TSOX_POOL=1`)下 worker 进程中途死亡,用例记为 SKIP。初版全量 8 例(0.12%)。v6 修复后仅剩 1 例(0.015%),且与每例进程基准的全部判定差异(73 处)均为基线超时例在池化下真实完成,无错误判定。默认路径仍为每例进程模式,池化需显式 `TSOX_POOL=1`。

## 根因(2026-10-15 复盘,三个独立机制)

### 1. 跨例状态污染(bindOnce 引入,已修复 3f0874ba)

初版池化叠加 bind 产物缓存(b9ad9208)后全量 fail 1082→5452:checker 的全局合并(`populate_globals`)与 module augmentation 会把 checker 侧合并符号经 unsafe 指针**原地写回 file symbol 的 members**(Go 对应路径写 per-checker links,移植时选择了直写共享表)。lib file symbol 被 bind 缓存共享后,前一例的合并结果被后例继承。二分锁定:污染源 `acceptSymbolAsWeakType.ts`(WeakSet this 型),受害者 `accessInstanceMemberFromStaticMethod01.ts`。修复:Symbol 手工 Clone(id 经 AtomicU64 保持,links 键不变),各 Program 对 lib file symbol 持私有克隆,其余 bind 产物保持只读共享。

### 2. 虚拟内存上限(已修复 5ed795b5)

重例单例峰值实测 3.86GB(nestedSpreadsAndWidening),贴 4GiB RLIMIT_AS;常驻进程附加 bind/parse 缓存(~数百 MB)与分配器跨例滞留把它推过线,Rust alloc handler 打印 `memory allocation of N bytes failed` 后 SIGABRT。8GiB 对照实验:pool-only crash 4→1。修复:常驻 worker 虚拟内存下限 8GiB(`WorkerSpec::effective_mem_bytes`;RSS 不受影响,跑飞用例仍由超时兜底)。辅助:逐例 `malloc_trim(0)` + `MALLOC_ARENA_MAX=2`。

### 3. 栈溢出(残留 1 例,未修)

`jsxElementType.tsx` 在常驻模式下大栈线程(256MB)溢出(`fatal runtime error: stack overflow, aborting`),RSS 仅 78MB,与内存无关;同例单例模式 PASS。怀疑共享 bind 产物图(OnceLock<Weak> 父链/flow 节点身份)使深递归路径更深,未定位。发生时该 worker 队列中无未完成请求(单飞行协议)。

### 附:观察期消失项

初版 8 例中昨天报告的另外 7 例在 v6 不再复现(任一模式),与协议修正(退出码捕获/stderr 观测/前缀去重)和内存修复的叠加效果有关,未逐例归因。`mappedTypeRecursiveInference.ts` 等基线超时例在常驻模式下 8.6s 完成(单例 30s 超时),池化的暖 lib 状态顺带救回一批贴超时线用例——这是 73 处 SKIP→FAIL/PASS 差异的来源,属数据质量改善。

## 观测设施(本次新增,常驻调试基建)

- 死亡分支 `child.wait()` 收取真实退出码/信号,失败详情归一(消除双层 "worker crashed (worker crashed …)" 前缀)。
- worker 逐例 stderr 上报 `RESIDENT case=<名> rss=<VmRSS> vsize=<VmSize>`,resident stderr inherit 到 xtask 控制台。
- 12 worker 并发写同一 stderr 会交错串行(已知噪音,单 worker 调试时干净)。

## 排查状态

- [x] 跨例污染(机制 1)——已修复并全量验收
- [x] 虚拟内存上限(机制 2)——已修复并全量验收
- [ ] 栈溢出(机制 3,jsxElementType.tsx)——待办:RUST_BACKTRACE=1 抓栈、对比单例/常驻的递归深度差、评估常驻栈扩容(512MB)或该例降级
- 全量验收:池化 vs 每例进程非超时差异 0;wall 118s / CPU 1183s(池化前 137s/1445s,单例进程 565s/6425s)

## 状态

- 默认路径:每例进程模式。
- 池化:`TSOX_POOL=1` 显式启用,判定与基准逐例等价(除超时例完成),残留 1 例栈溢出 SKIP。
