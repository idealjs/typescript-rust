# TypeScript-Rust Conformance Test Improvement TODO

## Current Status
- **Pass: 1787/5907 (30.2%)**
- Panic: 0, Lib tests: 1295/1295
- Total session progress: **+296 tests** (from 1491 to 1787)

## Remaining Priority Tasks

### P0: Parser Syntax Features
- [ ] `using` / `await using` in complex class contexts (partially works)
- [ ] Decorator emit/semantic checking (parsing works)
- [ ] Regex pattern parsing edge cases

### P1: TS2304 False Positives (~1000 tests remaining)
- [x] Generic type parameters in FunctionType/ConstructorType
- [x] Punctuation parser-recovery artifacts
- [x] CommonJS globals (exports, require, module)
- [x] ES built-in globals (Object, Array, Date, Math, etc.)
- [ ] `import` keyword being treated as identifier (331 false positives)
- [ ] Type parameter `T`/`U` scope in some contexts

### P2: TS2322 Type Assignment Compatibility (~977 tests missing)
- [ ] Assignment type checking (deferred — needs precise type inference)
- [ ] Structured type property matching
- [ ] Union/intersection narrowing in assignments
- [ ] Excess property checks for fresh object literals

### P3: TS2454 Variable Used Before Assignment
- [x] For-of/in loop variable definite assignment
- [ ] Precise control flow analysis for conditional branches

### P4: Message/Position Alignment
- [x] TS1121 octal literal `{0}` placeholder fix
- [x] TS2339 `typeof` prefix for constructor types
- [x] TS2564 computed property name source text
- [ ] Systematic position offset (Δline=-1 pattern)

## Completed Work

### This Session (+296 tests)
1. Object literal method/accessor shorthand parsing (+53 tests)
2. Constructor parameter properties (+25 tests)
3. Private identifier `#prop` scanning and parsing (+16 tests)
4. Class static block `static { ... }` (+10 tests)
5. CommonJS globals (+12 tests)
6. **Class get/set accessors + generators (+ES globals) (+171 tests)**
7. FunctionType/ConstructorType type param TS2304 suppression (+4 tests)
8. TS1121 octal literal placeholder fix (+3 tests)
9. Optional chaining element access `a?.[0]` (+2 tests)
10. Punctuation parser-recovery TS2304 filtering
11. TS2339 `typeof ClassName` prefix
12. TS2564 computed property name fix
13. For-of/in definite assignment
14. Logical assignment operators (verified)

### Previous Sessions
- import.meta MetaProperty parsing
- TS2564 strictPropertyInitialization check
- Heritage clause extends expression TS2304 suppression
- async function expression parsing
- /.src/ path convention fix
- TS-format error baseline generation
- Real baseline comparison against TS reference
- Stack-based type resolution cycle detection
