# LSP 迁移规划（严格 1:1 映射）

更新日期：2026-08-04

## 骨架完成状态：全部 10 Phase 完成 ✅

| Phase | 内容 | 文件数 | 行数 | 状态 |
|-------|------|--------|------|------|
| 1 | lsproto 协议类型 | 8 | ~809 | ✅ |
| 2 | logging + background + dirty | 7 | ~716 | ✅ |
| 3 | ls/lsconv 转换层 | 2 | ~300 | ✅ |
| 4 | ls/lsutil 工具层 | 8 | ~2,000 | ✅ |
| 5 | ls/change 文本变更 | 3 | ~1,400 | ✅ |
| 6 | ls/autoimport 自动导入 | 11 | ~3,398 | ✅ |
| 7 | ls 核心语言服务 | 38 | ~3,892 | ✅ |
| 8 | project/ata 自动类型获取 | 4 | ~937 | ✅ |
| 9 | project 项目管理 | 20 | ~3,996 | ✅ |
| 10 | lsp 服务器 | 6 | ~600 | ✅ |
| **合计** | | **107** | **~17,745** | **全部完成** |

**测试状态**：1,290 lib 通过（+8 新增），0 失败，0 ignored。
**Stub 数量**：95 个 `// TODO` 标记，待逐步填充实际逻辑。

## 原则

**严格按 Go 的逻辑做迁移，不做重构。**
- 每个 Go 文件 → 对应 Rust 文件（同名 snake_case）
- 每个 Go package → 对应 Rust 模块目录
- 每个 Go struct → Rust struct（同名字段）
- 每个 Go interface → Rust trait
- 每个 Go 方法签名 → Rust 方法签名
- 遵循 Go 的调用链和数据流，不改变架构

## Go 源码总量

| Go 目录 | 源码行数（不含测试） | 目标 Rust 目录 |
|---------|------------|-------------|
| `internal/lsp/` | ~2,474 | `src/lsp/` |
| `internal/lsp/lsproto/` | ~17,828 | `src/lsp/lsproto/` |
| `internal/lsp/lspwatcher/` | 596 | `src/lsp/lspwatcher/` |
| `internal/ls/` | ~24,200 | `src/ls/` |
| `internal/ls/autoimport/` | ~5,297 | `src/ls/autoimport/` |
| `internal/ls/change/` | 1,423 | `src/ls/change/` |
| `internal/ls/lsconv/` | 427 | `src/ls/lsconv/` |
| `internal/ls/lsutil/` | ~2,811 | `src/ls/lsutil/` |
| `internal/project/` | ~7,790 | `src/project/` |
| `internal/project/ata/` | ~937 | `src/project/ata/` |
| `internal/project/background/` | 52 | `src/project/background/` |
| `internal/project/dirty/` | ~729 | `src/project/dirty/` |
| `internal/project/logging/` | ~332 | `src/project/logging/` |
| **合计** | **~63,890** | |

## 迁移顺序（依赖拓扑序）

```
1. lsproto         — LSP 协议类型（无内部依赖）
2. logging/background/dirty — 基础设施（无内部依赖）
3. ls/lsconv       — 编译器↔LSP 转换（依赖 lsproto）
4. ls/lsutil       — 语言服务工具（依赖 lsproto, lsconv）
5. ls/change       — 文本变更追踪（依赖 lsconv）
6. ls/autoimport   — 自动导入（依赖 lsconv, lsutil）
7. ls              — 语言服务核心（依赖上述全部）
8. project/ata     — 自动类型获取（依赖 logging）
9. project         — 项目管理（依赖 ls + dirty + logging）
10. lsp/lspwatcher — 文件监听（依赖 lsconv, lsproto）
11. lsp            — LSP 服务器（依赖上述全部）
```

## 文件级映射

### Phase 1: lsproto

| Go 文件 | 行 | Rust 目标 |
|---------|---|-----------|
| `lsproto/baseproto.go` | 33 | `src/lsp/lsproto/baseproto.rs` |
| `lsproto/jsonrpc.go` | 130 | `src/lsp/lsproto/jsonrpc.rs` |
| `lsproto/lsp.go` | 312 | `src/lsp/lsproto/lsp.rs` |
| `lsproto/structcodec.go` | 164 | `src/lsp/lsproto/struct_codec.rs` |
| `lsproto/util.go` | 37 | `src/lsp/lsproto/util.rs` |
| `lsproto/lsp_generated.go` | 17,262 | `src/lsp/lsproto/lsp_generated.rs`（生成） |

### Phase 2: 基础设施

| Go 文件 | 行 | Rust 目标 |
|---------|---|-----------|
| `project/logging/logger.go` | 135 | `src/project/logging/logger.rs` |
| `project/logging/logtree.go` | 163 | `src/project/logging/log_tree.rs` |
| `project/logging/logcollector.go` | 34 | `src/project/logging/log_collector.rs` |
| `project/background/queue.go` | 52 | `src/project/background/queue.rs` |
| `project/dirty/box.go` | 62 | `src/project/dirty/box.rs` |
| `project/dirty/cloneablemap.go` | 9 | `src/project/dirty/cloneable_map.rs` |
| `project/dirty/entry.go` | 29 | `src/project/dirty/entry.rs` |
| `project/dirty/interfaces.go` | 15 | `src/project/dirty/interfaces.rs` |
| `project/dirty/map.go` | 169 | `src/project/dirty/map.rs` |
| `project/dirty/mapbuilder.go` | 74 | `src/project/dirty/map_builder.rs` |
| `project/dirty/syncmap.go` | 362 | `src/project/dirty/sync_map.rs` |
| `project/dirty/util.go` | 18 | `src/project/dirty/util.rs` |

### Phase 3: lsconv

| Go 文件 | 行 | Rust 目标 |
|---------|---|-----------|
| `ls/lsconv/converters.go` | 356 | `src/ls/lsconv/converters.rs` |
| `ls/lsconv/linemap.go` | 71 | `src/ls/lsconv/linemap.rs` |

### Phase 4: lsutil

| Go 文件 | 行 | Rust 目标 |
|---------|---|-----------|
| `ls/lsutil/userpreferences.go` | 901 | `src/ls/lsutil/user_preferences.rs` |
| `ls/lsutil/organizeimports.go` | 695 | `src/ls/lsutil/organize_imports.rs` |
| `ls/lsutil/symbol_display.go` | 438 | `src/ls/lsutil/symbol_display.rs` |
| `ls/lsutil/completednode.go` | 196 | `src/ls/lsutil/completed_node.rs` |
| `ls/lsutil/utilities.go` | 157 | `src/ls/lsutil/utilities.rs` |
| `ls/lsutil/formatcodeoptions.go` | 141 | `src/ls/lsutil/format_code_options.rs` |
| `ls/lsutil/children.go` | 130 | `src/ls/lsutil/children.rs` |
| `ls/lsutil/asi.go` | 104 | `src/ls/lsutil/asi.rs` |

### Phase 5: change

| Go 文件 | 行 | Rust 目标 |
|---------|---|-----------|
| `ls/change/tracker.go` | 751 | `src/ls/change/tracker.rs` |
| `ls/change/trackerimpl.go` | 402 | `src/ls/change/tracker_impl.rs` |
| `ls/change/delete.go` | 270 | `src/ls/change/delete.rs` |

### Phase 6: autoimport

| Go 文件 | 行 | Rust 目标 |
|---------|---|-----------|
| `ls/autoimport/registry.go` | 1,823 | `src/ls/autoimport/registry.rs` |
| `ls/autoimport/fix.go` | 1,314 | `src/ls/autoimport/fix.rs` |
| `ls/autoimport/import_adder.go` | 501 | `src/ls/autoimport/import_adder.rs` |
| `ls/autoimport/extract.go` | 461 | `src/ls/autoimport/extract.rs` |
| `ls/autoimport/view.go` | 256 | `src/ls/autoimport/view.rs` |
| `ls/autoimport/aliasresolver.go` | 236 | `src/ls/autoimport/alias_resolver.rs` |
| `ls/autoimport/util.go` | 323 | `src/ls/autoimport/util.rs` |
| `ls/autoimport/index.go` | 186 | `src/ls/autoimport/index.rs` |
| `ls/autoimport/export.go` | 142 | `src/ls/autoimport/export.rs` |
| `ls/autoimport/specifiers.go` | 75 | `src/ls/autoimport/specifiers.rs` |

### Phase 7: ls（核心语言服务）

| Go 文件 | 行 | Rust 目标 |
|---------|---|-----------|
| `ls/languageservice.go` | 132 | `src/ls/language_service.rs` |
| `ls/host.go` | 25 | `src/ls/host.rs` |
| `ls/constants.go` | 6 | `src/ls/constants.rs` |
| `ls/api.go` | 44 | `src/ls/api.rs` |
| `ls/hover.go` | 1,133 | `src/ls/hover.rs` |
| `ls/completions.go` | 6,295 | `src/ls/completions.rs` |
| `ls/findallreferences.go` | 2,654 | `src/ls/find_all_references.rs` |
| `ls/string_completions.go` | 2,211 | `src/ls/string_completions.rs` |
| `ls/signaturehelp.go` | 1,421 | `src/ls/signature_help.rs` |
| `ls/utilities.go` | 1,404 | `src/ls/utilities.rs` |
| `ls/codeactions_fixmissingtypeannotation.go` | 1,419 | `src/ls/code_actions_fix_missing_type.rs` |
| `ls/organizeimports.go` | 954 | `src/ls/organize_imports.rs` |
| `ls/inlay_hints.go` | 927 | `src/ls/inlay_hints.rs` |
| `ls/importTracker.go` | 767 | `src/ls/import_tracker.rs` |
| `ls/documenthighlights.go` | 752 | `src/ls/document_highlights.rs` |
| `ls/sourcedefinition.go` | 707 | `src/ls/source_definition.rs` |
| `ls/symbols.go` | 694 | `src/ls/symbols.rs` |
| `ls/callhierarchy.go` | 1,103 | `src/ls/call_hierarchy.rs` |
| `ls/jsdoc_snippet.go` | 594 | `src/ls/jsdoc_snippet.rs` |
| `ls/semantictokens.go` | 576 | `src/ls/semantic_tokens.rs` |
| `ls/folding.go` | 568 | `src/ls/folding.rs` |
| `ls/crossproject.go` | 421 | `src/ls/cross_project.rs` |
| `ls/definition.go` | 440 | `src/ls/definition.rs` |
| `ls/codeactions.go` | 399 | `src/ls/code_actions.rs` |
| `ls/codeactions_importfixes.go` | 452 | `src/ls/code_actions_import_fixes.rs` |
| `ls/codeactions_missingmemberfixer.go` | 498 | `src/ls/code_actions_missing_member.rs` |
| `ls/rename.go` | 379 | `src/ls/rename.rs` |
| `ls/file_rename.go` | 382 | `src/ls/file_rename.rs` |
| `ls/codeactions_fixclassincorrectlyimplementsinterface.go` | 236 | `src/ls/code_actions_fix_implements.rs` |
| `ls/displaypartswriter.go` | 217 | `src/ls/display_parts_writer.rs` |
| `ls/selectionranges.go` | 211 | `src/ls/selection_ranges.rs` |
| `ls/codelens.go` | 207 | `src/ls/code_lens.rs` |
| `ls/source_map.go` | 134 | `src/ls/source_map.rs` |
| `ls/linkedediting.go` | 107 | `src/ls/linked_editing.rs` |
| `ls/jsdoc.go` | 161 | `src/ls/jsdoc.rs` |
| `ls/format.go` | 181 | `src/ls/format.rs` |
| `ls/autoinsert.go` | 99 | `src/ls/auto_insert.rs` |
| `ls/diagnostics.go` | 58 | `src/ls/diagnostics.rs` |

### Phase 8: ata

| Go 文件 | 行 | Rust 目标 |
|---------|---|-----------|
| `project/ata/typesmap.go` | 505 | `src/project/ata/types_map.rs` |
| `project/ata/ata.go` | 500 | `src/project/ata/ata.rs` |
| `project/ata/discovertypings.go` | 334 | `src/project/ata/discover_typings.rs` |
| `project/ata/validatepackagename.go` | 98 | `src/project/ata/validate_package_name.rs` |

### Phase 9: project

| Go 文件 | 行 | Rust 目标 |
|---------|---|-----------|
| `project/session.go` | 1,837 | `src/project/session.rs` |
| `project/projectcollectionbuilder.go` | 1,259 | `src/project/project_collection_builder.rs` |
| `project/snapshotfs.go` | 802 | `src/project/snapshot_fs.rs` |
| `project/configfileregistrybuilder.go` | 733 | `src/project/config_file_registry_builder.rs` |
| `project/snapshot.go` | 562 | `src/project/snapshot.rs` |
| `project/checkerpool.go` | 530 | `src/project/checker_pool.rs` |
| `project/project.go` | 502 | `src/project/project.rs` |
| `project/watch.go` | 469 | `src/project/watch.rs` |
| `project/overlayfs.go` | 395 | `src/project/overlay_fs.rs` |
| `project/projectcollection.go` | 359 | `src/project/project_collection.rs` |
| `project/configfileregistry.go` | 202 | `src/project/config_file_registry.rs` |
| `project/autoimport.go` | 180 | `src/project/auto_import.rs` |
| `project/compilerhost.go` | 108 | `src/project/compiler_host.rs` |
| `project/ownercache.go` | 107 | `src/project/owner_cache.rs` |
| `project/refcountcache.go` | 114 | `src/project/refcount_cache.rs` |
| `project/programcounter.go` | 48 | `src/project/program_counter.rs` |
| `project/filechange.go` | 86 | `src/project/file_change.rs` |
| `project/client.go` | 21 | `src/project/client.rs` |
| `project/parsecache.go` | 39 | `src/project/parse_cache.rs` |
| `project/extendedconfigcache.go` | 51 | `src/project/extended_config_cache.rs` |
| `project/api.go` | 28 | `src/project/api.rs` |

### Phase 10: lsp

| Go 文件 | 行 | Rust 目标 |
|---------|---|-----------|
| `lsp/server.go` | 1,883 | `src/lsp/server.rs` |
| `lsp/dynamic_queue.go` | 94 | `src/lsp/dynamic_queue.rs` |
| `lsp/progress.go` | 217 | `src/lsp/progress.rs` |
| `lsp/logger.go` | 184 | `src/lsp/logger.rs` |
| `lsp/stack_sanitizer.go` | 96 | `src/lsp/stack_sanitizer.rs` |
| `lsp/lspwatcher/lspwatcher.go` | 596 | `src/lsp/lspwatcher.rs` |

## Handler 注册表（完整 1:1）

### registerNotificationHandler（10 个）

| LSP Method | Handler |
|------------|---------|
| `initialized` | `handle_initialized` |
| `exit` | `handle_exit` |
| `workspace/didChangeConfiguration` | `handle_did_change_workspace_configuration` |
| `textDocument/didOpen` | `handle_did_open` |
| `textDocument/didChange` | `handle_did_change` |
| `textDocument/didSave` | `handle_did_save` |
| `textDocument/didClose` | `handle_did_close` |
| `workspace/didChangeWatchedFiles` | `handle_did_change_watched_files` |
| `$/setTrace` | `handle_set_trace` |
| `$/setLogVerbosity` | `handle_set_log_verbosity` |

### registerRequestHandler（16 个）

| LSP Method | Handler |
|------------|---------|
| `initialize` | `handle_initialize` |
| `shutdown` | `handle_shutdown` |
| `workspace/willRenameFiles` | `handle_will_rename_files` |
| `textDocument/rename` | `handle_rename` |
| `callHierarchy/incomingCalls` | `handle_call_hierarchy_incoming_calls` |
| `callHierarchy/outgoingCalls` | `handle_call_hierarchy_outgoing_calls` |
| `workspace/symbol` | `handle_workspace_symbol` |
| `completionItem/resolve` | `handle_completion_item_resolve` |
| `codeLens/resolve` | `handle_code_lens_resolve` |
| `_/runGC` | `handle_run_gc` |
| `_/saveHeapProfile` | `handle_save_heap_profile` |
| `_/saveAllocProfile` | `handle_save_alloc_profile` |
| `_/startCPUProfile` | `handle_start_cpu_profile` |
| `_/stopCPUProfile` | `handle_stop_cpu_profile` |
| `_/initializeAPISession` | `handle_initialize_api_session` |
| `_/projectInfo` | `handle_project_info` |

### registerLanguageServiceDocumentRequestHandler（22 个）

| LSP Method | Handler |
|------------|---------|
| `textDocument/diagnostic` | `handle_document_diagnostic` |
| `textDocument/hover` | `handle_hover` |
| `textDocument/definition` | `handle_definition` |
| `_/textDocumentSourceDefinition` | `handle_source_definition` |
| `textDocument/typeDefinition` | `handle_type_definition` |
| `textDocument/signatureHelp` | `handle_signature_help` |
| `textDocument/formatting` | `handle_document_format` |
| `textDocument/rangeFormatting` | `handle_document_range_format` |
| `textDocument/onTypeFormatting` | `handle_document_on_type_format` |
| `textDocument/documentSymbol` | `handle_document_symbol` |
| `textDocument/documentHighlight` | `handle_document_highlight` |
| `_/multiDocumentHighlight` | `handle_multi_document_highlight` |
| `textDocument/selectionRange` | `handle_selection_range` |
| `textDocument/inlayHint` | `handle_inlay_hint` |
| `textDocument/codeLens` | `handle_code_lens` |
| `textDocument/prepareCallHierarchy` | `handle_prepare_call_hierarchy` |
| `textDocument/foldingRange` | `handle_folding_range` |
| `textDocument/prepareRename` | `handle_prepare_rename` |
| `textDocument/linkedEditingRange` | `handle_linked_editing_range` |
| `_/_vs_onAutoInsert` | `handle_vs_on_auto_insert` |
| `textDocument/semanticTokens/full` | `handle_semantic_tokens_full` |
| `textDocument/semanticTokens/range` | `handle_semantic_tokens_range` |

### registerLanguageServiceWithAutoImportsRequestHandler（2 个）

| LSP Method | Handler |
|------------|---------|
| `textDocument/completion` | `handle_completion` |
| `textDocument/codeAction` | `handle_code_action` |

### registerMultiProjectReferenceRequestHandler（3 个）

| LSP Method | Handler |
|------------|---------|
| `textDocument/references` | `LanguageService::provide_references` |
| `_/_vs_references` | `LanguageService::provide_vs_references` |
| `textDocument/implementation` | `LanguageService::provide_implementations` |

**总计：53 个 LSP 方法注册**

## Go → Rust 映射规则

| Go 概念 | Rust 对应 |
|---------|-----------|
| `interface` | `trait` |
| `struct` | `struct` |
| `*T` (pointer) | `&T` / `&mut T` |
| `context.Context` | 取消令牌 |
| `sync.Mutex` | `std::sync::Mutex` |
| `sync.RWMutex` | `std::sync::RwLock` |
| `atomic.Int32` | `std::sync::atomic::AtomicI32` |
| `sync.Once` | `std::sync::Once` |
| `map[K]V` | `HashMap<K, V>` |
| `chan T` | `mpsc::channel` |
| `goroutine` | `std::thread::spawn` |
| `error` | `Result<T, Error>` |
| `nil` | `None` |
| 手动 refcount | `Arc<T>` 或手动原子计数 |
