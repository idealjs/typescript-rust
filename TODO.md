# TypeScript-Rust Conformance Test Improvement TODO

## Current Status
- **Pass: 1600/5907 (27.1%)**
- Panic: 0, Lib tests: 1295/1295
- Total session progress: **+109 tests** (from 1491 to 1600)

## Remaining Priority Tasks

### P0: Parser Syntax Features (highest ROI)
- [ ] `using` / `await using` declarations in complex contexts (class methods, for loops)
- [ ] Decorators `@decorator` syntax on classes/methods/properties
- [ ] Private identifier `#prop` in member access and declarations
- [ ] `??=` `||=` `&&=` logical assignment operators
- [ ] More complex computed property name patterns

### P1: TS2304 False Positives (still ~1200 tests)
- [x] Generic type parameters in FunctionType/ConstructorType (fixed)
- [x] Punctuation parser-recovery artifacts filtered (fixed)
- [x] CommonJS globals (exports, require, module) added
- [ ] Heritage clause `extends GlobalName` resolution
- [ ] Built-in globals missing in some expression contexts

### P2: TS2322 Type Assignment Compatibility (~364 tests missing)
- [ ] Enhance `is_assignable_to` for structured types
- [ ] Property name/shape matching for object literal assignment
- [ ] Union/intersection type narrowing in assignments

### P3: TS2454 Variable Used Before Assignment (~202 tests missing)
- [ ] Extend definite-assignment analysis to `var` declarations
- [ ] Handle `for...of` loop binding assignment

### P4: Message/Position Alignment
- [x] TS1121 octal literal `{0}` placeholder fix
- [x] TS2339 `typeof` prefix for constructor types
- [ ] TS2564 computed property name empty string
- [ ] Systematic position offset (Δline=-1 pattern in ~25 tests)

## Completed Work

### This Session (+109 tests)
1. Object literal method/accessor shorthand parsing (+53 tests)
2. Constructor parameter properties `constructor(public x: number)` (+25 tests)
3. Class static block `static { ... }` (+10 tests)
4. CommonJS globals (exports, require, module, etc.) (+12 tests)
5. FunctionType/ConstructorType type param TS2304 suppression (+4 tests)
6. TS1121 octal literal placeholder fix (+3 tests)
7. Optional chaining element access `a?.[0]` (+2 tests)
8. Punctuation parser-recovery TS2304 filtering
9. TS2339 `typeof ClassName` prefix for constructor types

### Previous Sessions
- import.meta MetaProperty parsing
- TS2564 strictPropertyInitialization check
- Heritage clause extends expression TS2304 suppression
- async function expression parsing
- /.src/ path convention fix
- TS-format error baseline generation
- Real baseline comparison against TS reference
- Stack-based type resolution cycle detection
