# ts2rust-port

TypeScript 到 Rust 的移植工程（tsox 系列 crate，详见 `Cargo.toml` workspace members 与 `AGENTS.md`）。

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
