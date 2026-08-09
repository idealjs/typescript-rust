# TypeScript-Rust Conformance Test Improvement TODO

## Current Status
- **Pass: 1491/5907 (25.2%)**
- Panic: 0, Lib tests: 1295/1295

## Priority Tasks (by impact)

### P0: TS2304 False Positives (~1366 tests affected)
- [ ] Generic type parameter `T`/`U` in construct/call signatures not resolved (704 occurrences)
  - e.g. `new <T>(x: T) => void` in interface members
  - Root cause: type parameter scope not pushed for construct signatures
- [ ] `extends Object` / `extends Error` global names not resolved in heritage clauses
  - Root cause: `resolve_identifier` fails for globals in expression position
- [ ] `exports` CommonJS global not recognized (370 occurrences)
- [ ] `(`, `{`, `)` punctuation reported as names (parser recovery artifacts)
- [ ] Built-in globals (`Object`, `Date`, `Array`, `Intl`) missing in some contexts

### P1: TS1005 Parser Syntax Errors (~803 tests affected)
- [ ] Private identifier `#prop` not supported in member access
- [ ] Optional chaining with template literals `obj?.foo` backtick recovery
- [ ] `using` / `await using` declarations not parsed
- [ ] Multi-level member access in optional chains `a?.b.c`
- [ ] Type system syntax in expressions (`as const`, `satisfies`, `typeof`)

### P2: TS2322 Type Assignment Compatibility (~368 tests)
- [ ] Enhance `is_assignable_to` for structured types (interfaces, object types)
- [ ] Property name/shape matching for object literal assignment
- [ ] Numeric literal vs number type assignment
- [ ] Union type narrowing in assignments

### P3: TS2454 Variable Used Before Assignment (~428 tests)
- [ ] Extend definite-assignment analysis to `var` declarations
  - Requires precise flow analysis to avoid false positives
- [ ] Handle `for...of` loop binding assignment (`for (var w of []) { ... }`)
- [ ] Handle conditional paths where assignment may not execute

### P4: Other High-Frequency Missing Diagnostics
- [ ] TS1219: Duplicate string index signature (10 tests)
- [ ] TS1036: Statements must be separated (9 tests)
- [ ] TS2300: Duplicate identifier (7 tests)
- [ ] TS2430: Types of construct signatures incompatible (6 tests)
- [ ] TS2353: Object literal property does not exist (5 tests)
- [ ] TS2345: Argument not assignable to parameter (5 tests)

## Completed Work
- [x] TS2564 strictPropertyInitialization check (+22 tests)
- [x] import.meta parser support (MetaProperty)
- [x] async function expression parsing (+10 tests)
- [x] async/await parser support + await context check
- [x] TS-format error baseline generation with source interleaving
- [x] Real baseline comparison against TS reference (--ts-ref-dir)
- [x] /.src/ path convention fix (+63 tests)
- [x] Stack-based type resolution cycle detection
- [x] Heritage clause extends expression TS2304 suppression
