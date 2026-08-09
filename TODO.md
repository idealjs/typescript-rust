# TypeScript-Rust Conformance Test Improvement TODO

## Current Status
- **Pass: 1616/5907 (27.3%)**
- Panic: 0, Lib tests: 1295/1295
- Total session progress: **+125 tests** (from 1491 to 1616)

## Remaining Priority Tasks

### P0: Parser Syntax Features (highest ROI, still significant)
- [ ] `using` / `await using` declarations in complex contexts (class methods)
- [ ] More complex computed property name patterns in object literals
- [ ] Nullish coalescing `??` in complex expressions (panic on some tests)
- [ ] Regex pattern parsing edge cases

### P1: TS2304 False Positives (~1200 tests affected)
- [x] Generic type parameters in FunctionType/ConstructorType (fixed)
- [x] Punctuation parser-recovery artifacts filtered (fixed)
- [x] CommonJS globals (exports, require, module) added
- [ ] Heritage clause `extends GlobalName` resolution (partially fixed)
- [ ] Built-in globals missing in some expression contexts

### P2: TS2322 Type Assignment Compatibility (~364 tests missing)
- [ ] Enhance `is_assignable_to` for structured types (mirror Go relater.go)
- [ ] Property name/shape matching for object literal assignment
- [ ] Union/intersection type narrowing in assignments
- [ ] Excess property checks for fresh object literals

### P3: TS2454 Variable Used Before Assignment
- [x] For-of/in loop variable definite assignment (fixed)
- [ ] Precise control flow analysis for conditional branches
- [ ] Var declaration definite assignment

### P4: Message/Position Alignment
- [x] TS1121 octal literal `{0}` placeholder fix
- [x] TS2339 `typeof` prefix for constructor types
- [ ] TS2564 computed property name empty string
- [ ] Systematic position offset (Δline=-1 pattern in ~25 tests)

## Completed Work

### This Session (+125 tests)
1. Object literal method/accessor shorthand parsing (+53 tests)
2. Constructor parameter properties `constructor(public x: number)` (+25 tests)
3. Private identifier `#prop` scanning and parsing (+16 tests)
4. Class static block `static { ... }` (+10 tests)
5. CommonJS globals (exports, require, module, etc.) (+12 tests)
6. FunctionType/ConstructorType type param TS2304 suppression (+4 tests)
7. TS1121 octal literal placeholder fix (+3 tests)
8. Optional chaining element access `a?.[0]` (+2 tests)
9. Punctuation parser-recovery TS2304 filtering
10. TS2339 `typeof ClassName` prefix for constructor types
11. For-of/in definite assignment analysis
12. Logical assignment operators (already supported, verified)

### Previous Sessions
- import.meta MetaProperty parsing
- TS2564 strictPropertyInitialization check
- Heritage clause extends expression TS2304 suppression
- async function expression parsing
- /.src/ path convention fix
- TS-format error baseline generation
- Real baseline comparison against TS reference
- Stack-based type resolution cycle detection
