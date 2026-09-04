# LSP Handler 注册表（完整 1:1）与映射规则

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
