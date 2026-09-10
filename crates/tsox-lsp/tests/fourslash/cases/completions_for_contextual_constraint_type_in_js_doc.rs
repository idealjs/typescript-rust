use tsox_lsp::fourslash::{self, Session};


#[ignore = "generator: //"]
#[test]
fn completions_for_contextual_constraint_type_in_js_doc() {
    let content = r#"
// @allowJs: true
// @filename: a.ts
export interface Blah<T extends { a: "hello" | "world" }> {
}

// @filename: b.js
/** @import * as a from "./a" */

/** @type {a.Blah<{ a: /*1*/ }>} */
let x;

// @filename: c.js
/** @import * as a from "./a" */

/** @type {a.Blah<{ a: /*2*/ }>} */
"#;
    let mut s = Session::new_for_test("completionsForContextualConstraintTypeInJsDoc", content);
    // TODO: // These examples both would panic in retrieving the symbols
    // TODO: // of property signature nodes within JSDoc types.
    // TODO: // In both cases, we'd have a JSDoc property signature that has no symbol.
    // TODO: //
    // TODO: // The two cases differ in whether or not there is a variable declaration
    // TODO: // following the `@type` comment. These are important to test differently
    // TODO: // because of how JSDoc re-parsing would construct nodes in the tree.
    // TODO: //
    // TODO: // Getting the symbol of the reparsed node is a sufficient fix for marker 1.
    // TODO: // However, that would not fix the case at marker 2 because
    // TODO: // there is no variable to attach the `@type` annotation, so the node basically
    // TODO: // doesn't exist for subsequent passes like the binder.
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, f.Markers(), &fourslash.CompletionsExpectedList{
}
