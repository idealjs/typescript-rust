# ts2rust-port

TypeScript 到 Rust 的移植工程（tsox 系列 crate，详见 `Cargo.toml` workspace members 与 `AGENTS.md`）。

## 文档导航

- `docs/`：怎么做。设计文档、操作流程（如 [subagent 语料修复工作流](docs/subagent-corpus-repair.md)）
- `decisions/`：已拍板决策（ADR），含否决项留档（如 [subagent 任务边界](decisions/subagent-corpus-repair.md)）
- 各文档的变更记录见同名 `.change.md` 文件

## 环境备忘

### 无头启动 ZCode GUI（Xvfb）

前提：`--ozone-platform=x11` 必须加，且环境里不能有 `WAYLAND_DISPLAY`。ZCode 硬编码偏好 Wayland，连不上时不回退 X11，而是进入无显示连接的僵尸状态（进程存活、日志正常、但永不渲染）。

```bash
Xvfb :99 -screen 0 1920x1080x24 -nolisten tcp &
env -u WAYLAND_DISPLAY -u GDK_BACKEND DISPLAY=:99 /opt/ZCode/zcode --ozone-platform=x11
```

验证窗口已创建：

```bash
DISPLAY=:99 xwininfo -root -tree
```
