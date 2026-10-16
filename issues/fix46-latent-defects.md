> 版本：v1.0 · 2026-10-06

# fix46 波暴露的潜伏缺陷

## declaration_name_to_string 跨文件文本失配 panic

后果：任何跨文件符号走到 `report_nonexistent_property` 的诊断格式化时，worker 进程 panic，用例记 SKIP，波及面随诊断覆盖面扩大（fix46 整合期一次触发 440+ SKIP）。

机制：`crates/tsox-frontend/src/scanner/mig/m3i.rs:166` 的 `get_text_of_node_from_source_text` 以调用方传入的 `source_text` 切 `node.pos()..node.end()`；`declaration_name_to_string`（诊断格式化路径，栈：`report_nonexistent_property → declaration_name_to_string → get_text_of_node → get_text_of_node_from_source_text`）传入的字符串与节点所属文件不一致（实测 6 字节串被切 213 偏移），越界 panic。触发样本：2dArrays.ts + 属性访问臂走 `get_type_of_expression` 全路径（已回退的 w12/67346603 路由）。

修法方向：`declaration_name_to_string` 按节点所属 SourceFile 取文本，而非沿用当前文件的 source_text。

## TypeOnly 查询模式缺诊断抑制

后果：类型查询路径（对应 Go `checkExpressionEx(CheckModeTypeOnly)`）在 Rust 侧重查表达式时全额上报诊断，Go 在该模式抑制大部分上报；差异使「发射期/查询期取型」产生 Go 不存在的错误，并触发上一条 panic。

fix46 处置：整合分支回退了 w12 的属性访问臂路由 commit（见 949c9c17fe），待生产侧实现 CheckModeTypeOnly 的上报抑制后，该路由改造与 bigintWithLib 例一并回归下波。

## 暖 checker 发射暴露的 lib 类型别名分歧

fix46 波收尾把 .types 发射改为按文件亲和取暖 checker（对齐 Go `GetTypeCheckerForFile`，见 6f65af01352）。冷发射此前掩盖的真实分歧随之显形：check 期缓存的 lib 签名是展开形（`import("lib.es2020.intl").Intl.Locale` 联合），Go 暖缓存保留别名形（`Intl.LocalesArgument`）。样本：bigintWithLib.ts 行 71/73。属类型别名在缓存边界的保全问题，交下波按代表例修。

## 历史修复的跨族纠缠（2026-10-06 主修轮实证）

主 agent 重放 13 个历史已验证修复（fix47/fix48 各波 pass 例的 commit）至保守终态：单例电池 8/8 MATCH，但全量 anchor 2192→2337（+145）；逐项回退试验显示布尔自新净 −11（回退它反升 2348）、其余 12 个集体 +156 且不可分。结论：**单例 anchor 快检验证 ≠ 净正向**，各波的语义修复在布尔拓宽/数组包裹/undefined/实例化机械上互为反向面，任何子集批量重放均引入更大损伤。正确引擎仍是波次循环（全量水位 + 回归归因）；已验证修复的复用必须伴随全量归因，不得按单例通过判定。

## 整合互损三源（2026-10-06 查明）

① fix47 整合 keep-HEAD 分辨丢弃先落侧语义；② fix47b 盲整合 7 个未验证 commit（两套命名会话系统共存）；③ fix48 的 50 落 32 回滚震荡。叠加致 fix47 波 13 个 pass 例中 9 个的修复在后续波中反复得失。LFS 大 CSV 用 git show 只得指针，对其 grep 恒空——曾致「2189 无回归」误判（判定基线必须用工作区实际文件或归档 run 日志）。
