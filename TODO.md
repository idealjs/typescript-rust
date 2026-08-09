# TypeScript-Rust Conformance Test Improvement TODO

## Current Status
- **Pass: 1586/5907 (26.9%)**
- Panic: 0, Lib tests: 1295/1295
- Session progress: +95 tests (from 1491)

## Priority Tasks (by impact)

### P0: Parser Syntax Features (still highest ROI)
- [ ] `using` / `await using` declarations (ES2024)
- [ ] Index signature parsing in interfaces: `[key: string]: T`
- [ ] Optional chaining with element access: `a?.[0]`
- [ ] `exports` CommonJS global recognition
- [ ] Private identifier `#prop` in member access
- [ ] Decorators `@decorator` syntax

### P1: TS2304 False Positives (reduced but still significant)
- [x] Generic type parameter `T`/`U` in FunctionType/ConstructorType (fixed)
- [x] Punctuation parser-recovery artifacts filtered (fixed)
- [ ] `extends Object` / global names in heritage clauses
- [ ] `exports` / `require` CommonJS globals

### P2: TS2322 Type Assignment Compatibility (~364 tests missing)
- [ ] Enhance `is_assignable_to` for structured types
- [ ] Property name/shape matching for object literal assignment
- [ ] Numeric literal vs number type assignment

### P3: TS2454 Variable Used Before Assignment (~202 tests missing)
- [ ] Extend definite-assignment analysis to `var` declarations
- [ ] Handle `for...of` loop binding assignment

### P4: Message/Position Alignment (~57 tests close to passing)
- [x] TS1121 octal literal `{0}` placeholder fix
- [x] TS2339 `typeof` prefix for constructor types
- [ ] TS2564 computed property name empty string
- [ ] Systematic position offset investigation (Δline=-1 pattern)

## Completed Work (this session)
- [x] Object literal method/accessor shorthand parsing (+53 tests)
- [x] Constructor parameter properties `constructor(public x: number)` (+25 tests)
- [x] Class static block `static { ... }` (+10 tests)
- [x] TS1121 octal literal placeholder fix (+3 tests)
- [x] FunctionType/ConstructorType type param TS2304 suppression (+4 tests)
- [x] Punctuation parser-recovery TS2304 filtering
- [x] TS2339 `typeof ClassName` prefix for constructor types
- [x] import.meta MetaProperty parsing
- [x] TS2564 strictPropertyInitialization check
- [x] Heritage clause extends expression TS2304 suppression
- [x] async function expression parsing
- [x] /.src/ path convention fix
