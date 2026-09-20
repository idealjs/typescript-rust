# subagent-corpus-repair 变更记录

## v1.0 · 2026-09-20

初次记录。背景：corpus3 轮次多 subagent 并行修复中，subagent 反复被平台配额终止（票据过期、600 秒无活动超时），且接手后需自行跑家族批量定位目标导致修复缓慢。据此拍板：全量验证主 agent 独占、subagent 接明确用例清单、小步提交与交接日志应对配额终止。
