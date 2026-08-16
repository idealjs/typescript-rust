# triaged.txt 根因分类报告（2026-08-16 生成）

105 个批次分组 → 21 个统一根因分类；条目计数 = baseline 文件数（无重复，合计 2976）。
优先级建议：按 分类条数 ÷ 预估工作量 排序；G5 杂项桶（573 条）需先二次分诊再排期。

## A1 泛型调用推断与约束 — 277 条（11 个批次组）
-  136 ← generic call-site inference & constraints — TS2344 constraint
-   60 ← type-argument inference & type-parameter checks (cont.)
-   33 ← 2026-08-16 batch #6100-#6536 (final partial batch, 437 cases):
-   20 ← 2026-08-16 batch #600-#699: misc checker depth — cast/as-expression
-    8 ← 2026-08-16 batch #800-#899: variance inference — co/contravariant
-    7 ← 2026-08-16 batch #800-#899: misc — TS2717 property-redecl type
-    6 ← generic inference gaps: mixin base `class extends Base`, contextual
-    4 ← variance annotations — measurement/propagation,
-    1 ← 2026-08-15 batch #41-#50: generic array-method instantiation
-    1 ← 2026-08-15 batch #51-#60: WeakMap/WeakRef/FinalizationRegistry generic
-    1 ← 2026-08-16 batch #100-#199: `<<` re-scan for ambiguous generic

## A2 上下文类型/签名实例化 — 45 条（1 个批次组）
-   45 ← 2026-08-16 batch #1200-#1299: contextual SIGNATURE instantiation —

## B1 控制流/收窄/明确赋值 — 539 条（7 个批次组）
-  472 ← 2026-08-16 batch #1300-#1399: control-flow definite-assignment/
-   29 ← narrowing subsystem — switch-discriminant narrowing,
-   20 ← 'in' operator narrowing (right operand to object),
-    9 ← use-before-declaration (final) — class decorators,
-    6 ← 2026-08-16 batch #500-#599: use-before-definite-flow analysis —
-    2 ← 2026-08-16 batch #200-#299: assertion-function narrowing
-    1 ← 2026-08-15 batch #61-#70: TS7023 circular return-type inference for

## E1 JSX 类型检查 — 134 条（4 个批次组）
-   83 ← JSX type-checking subsystem — IntrinsicElements/
-   27 ← 2026-08-16 batch #5100-#6099 (1000-case batch):
-   21 ← 2026-08-16 batch #900-#999: comment placement parser recovery —
-    3 ← 2026-08-16 batch #200-#299: arguments in class-field initializers and

## G1 枚举检查器家族 — 54 条（5 个批次组）
-   24 ← enum checker family (cont.) — literal displays ('E.A'), TS2432
-   12 ← 'typeof' type queries (final) — typeof classes/enums/
-   10 ← 2026-08-16 batch #1000-#1099: computed properties — destructuring
-    7 ← 2026-08-16 batch #1100-#1199: const-enum checker family not ported —
-    1 ← TS2628 assign-to-enum / TS2540 readonly-property assignment checks

## D2 声明产生/emit 基线 — 122 条（4 个批次组）
-   82 ← 2026-08-16 batch #3100-#4099 (1000-case batch):
-   18 ← source-map validation baselines (emit-artifact
-   13 ← declaration-nameability/privacy checks (TS4023 family,
-    9 ← 2026-08-16 batch #1000-#1099: complex generic relations — recursive

## D1 模块解析/导入导出/增强 — 204 条（9 个批次组）
-  159 ← 2026-08-16 batch #4100-#5099 (1000-case batch):
-   19 ← 2026-08-16 batch #400-#499: module augmentation via export= —
-   17 ← triple-slash/tslib/symlink resolution — types refs
-    5 ← 2026-08-16 batch #500-#599: module-resolution caching & cached
-    1 ← 2026-08-15 batch #71-#80: import-alias / namespace export resolution
-    1 ← 2026-08-15 batch #101-#110: import-equals alias VALUE typing under
-    1 ← 2026-08-15 batch #101-#110: node_modules / package.json "exports"
-    1 ← 2026-08-16 batch #300-#399: export= augmentation resolution
-    0 ← 2026-08-15 batch #81-#90: cross-file import-alias resolution (`import X =

## C1 类继承/成员/合并 — 120 条（6 个批次组）
-   63 ← interface/class inheritance member compatibility — override
-   45 ← 2026-08-16 batch #700-#799: class heritage/member subsystem —
-    6 ← 2026-08-16 batch #800-#899: class+namespace (clodule) merge across
-    5 ← 2026-08-16 batch #400-#499: class heritage checks — base-member
-    1 ← 2026-08-15 batch #71-#80: interface `extends` member/call-signature
-    0 ← 2026-08-15 batch #51-#60: abstract-class inheritance diagnostics not yet

## C2 super/this/static 成员检查 — 84 条（5 个批次组）
-   32 ← super-call/property checks — definite-super analysis,
-   24 ← static member checks — static/instance resolution
-   21 ← 'this' typing — this in modules/static methods/outer class
-    5 ← TS17005: super call in a class extending 'null'; TS2417 static-side
-    2 ← TS17009: 'super' must be called before accessing 'this' in a derived

## C3 重载解析与函数签名兼容 — 179 条（8 个批次组）
-   49 ← overload resolution & function-signature compat — TS2391-family
-   35 ← misc final-batch singles — UMD globals, unicode
-   31 ← 2026-08-16 batch #200-#299: array/arithmetic deep-checker cluster.
-   30 ← overload resolution (cont.) — literal-specialization
-   10 ← 2026-08-16 batch #1100-#1199: contextual typing / inference — overload
-   10 ← 2026-08-16 batch #1100-#1199: constructor signature family — TS2769
-    8 ← misc unported diagnostics: TS2552 did-you-mean suggestions, TS2420
-    6 ← 2026-08-16 batch #500-#599: overload-resolution depth — generic

## A3 条件类型/mapped/索引访问/模板 — 189 条（7 个批次组）
-  100 ← inference subsystem (cont.) — inferential typing, indexed-access
-   33 ← recursive-type handling — base-class cycle detection
-   28 ← tagged template & template-literal type checking —
-   14 ← 2026-08-16 batch #1000-#1099: conditional-type evaluation —
-    8 ← 2026-08-16 batch #400-#499: Awaited<T> conditional-type evaluation
-    4 ← 2026-08-16 batch #1100-#1199: type-parameter constraint checking —
-    2 ← 2026-08-16 batch #1100-#1199: relater/inference misc — TS2564 definite

## F1 解析器恢复/扫描 — 29 条（4 个批次组）
-   15 ← 2026-08-16 batch #1000-#1099: misc — missing-semicolon ASI
-   12 ← regular-expression literal scanning — Annex B,
-    1 ← 2026-08-15 batch #41-#50: binary-file recovery order — TS1490 scanner
-    1 ← 2026-08-16 batch #100-#199: parser recovery for contextual `module`

## F2 作用域/遮蔽/let 边界 — 54 条（3 个批次组）
-   38 ← let-scoping edge cases (TS2300-vs-TS2451 choice, 'let' as
-   15 ← scope/shadowing checks — class-property scope,
-    1 ← TS2301: initializer-references-constructor-local check

## G2 迭代器/spread/rest/for-in — 37 条（4 个批次组）
-   13 ← for-in / for-await-of iteration typing, fallthrough analysis
-   12 ← spread/rest typing — spread into index signatures,
-    7 ← 2026-08-16 batch #300-#399: async/generator typing — yield/yield*
-    5 ← yield* / generator contextual typing (final)

## G3 对象字面量检查 — 38 条（1 个批次组）
-   38 ← object-literal checking (cont.) — freshness/spread,

## G4 implicit-any 家族 — 20 条（1 个批次组）
-   20 ← implicit-any family (cont.) — destructuring/contextual

## H1 参数/访问器/严格模式/杂项诊断 — 62 条（7 个批次组）
-   30 ← optional-parameter & parameter checks — default-value
-   13 ← accessor pair typing & modifier checks on class elements
-    6 ← switch checking — case-expression comparability,
-    6 ← var/varargs checks (final) — varArg typing, var-as-ID,
-    4 ← with-statement checking — TS2410 invalid targets,
-    2 ← 2026-08-15 batch #111-#120: TS5101 outFile-deprecation warnings not
-    1 ← strict-mode checks the harness doesn't enable for these cases

## A4 推断杂项 — 72 条（4 个批次组）
-   24 ← Promise inference family — then/catch chaining,
-   20 ← undefined/unknown/inference edges (final) — undefined
-   15 ← 2026-08-16 batch #500-#599: call/inference misc — boxed Boolean
-   13 ← 2026-08-16 batch #400-#499: inference & error-form misc — best-common-

## B2 relater 可赋值性细化错误链 — 108 条（4 个批次组）
-   59 ← 2026-08-16 batch #300-#399: assignment-compat cluster — relater
-   30 ← relater elaborated error chains & excess-property deep checks
-   17 ← union-type relations (final) — subtype reduction,
-    2 ← 2026-08-15 batch #81-#90: TS2352 assertion overlap comparability

## G5 杂项单根因（未二次分诊） — 573 条（5 个批次组）
-  191 ← misc single-root gaps (batch #5100-#6099)
-  153 ← misc single-root gaps (batch #4100-#5099)
-  113 ← misc single-root gaps (batch #3100-#4099)
-   99 ← misc single-root gaps — narrowed during follow-up passes
-   17 ← misc single-root gaps (batch #6100-#6536)

## H2 其余杂项诊断 — 36 条（5 个批次组）
-   21 ← parser error-recovery anchors/ordering — TS1005/TS1144
-    9 ← 2026-08-16 batch #400-#499: bigint literal typing + target gating
-    4 ← 2026-08-16 batch #200-#299: suffixed configs of the triaged groups
-    1 ← 2026-08-15 diff-fix round: TS2749 for default-import aliases requires
-    1 ← 2026-08-15 batch #91-#100: TS2873/2872 truthiness of cast-of-null uses
