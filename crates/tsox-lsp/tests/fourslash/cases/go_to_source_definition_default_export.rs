use tsox_lsp::fourslash::Session;


#[test]
fn go_to_source_named_and_default_export() {
    // TODO: // findDeclarationNodesByName correctly finds both named exports and
    // TODO: // default-exported classes/functions via the AST visitor.
    let content = r#"// @moduleResolution: bundler
// @Filename: /home/src/workspaces/project/node_modules/pkg/package.json
{ "name": "pkg", "main": "./index.js", "types": "./index.d.ts" }
// @Filename: /home/src/workspaces/project/node_modules/pkg/index.d.ts
export default class Widget {}
export declare function helper(): void;
// @Filename: /home/src/workspaces/project/node_modules/pkg/index.js
export default class /*targetWidget*/Widget {}
export function /*targetHelper*/helper() {}
// @Filename: /home/src/workspaces/project/index.ts
import /*importDefault*/Widget, { /*importHelper*/helper } from "pkg";
Widget;
helper();"#;
    let _s = Session::new_for_test("goToSourceNamedAndDefaultExport", content);
    // TODO: f.VerifyBaselineGoToSourceDefinition(t, "importDefault", "importHelper")
}

#[test]
fn go_to_source_default_import_not_first_statement() {
    // TODO: // Default import navigates to the actual export default declaration,
    // TODO: // not the first statement of the file, when the default export is not first.
    let content = r#"// @moduleResolution: bundler
// @Filename: /home/src/workspaces/project/node_modules/pkg/package.json
{ "name": "pkg", "main": "./index.js", "types": "./index.d.ts" }
// @Filename: /home/src/workspaces/project/node_modules/pkg/index.d.ts
export declare const version: string;
export default class Widget {}
// @Filename: /home/src/workspaces/project/node_modules/pkg/index.js
export const version = "1.0";
export default class /*targetWidget*/Widget {}
// @Filename: /home/src/workspaces/project/index.ts
import /*importDefault*/Widget from "pkg";
Widget;"#;
    let _s = Session::new_for_test("goToSourceDefaultImportNotFirstStatement", content);
    // TODO: f.VerifyBaselineGoToSourceDefinition(t, "importDefault")
}

#[test]
fn go_to_source_unnamed_default_export() {
    let content = r#"// @moduleResolution: bundler
// @Filename: /home/src/workspaces/project/node_modules/pkg/package.json
{ "name": "pkg", "main": "./index.js", "types": "./index.d.ts" }
// @Filename: /home/src/workspaces/project/node_modules/pkg/index.d.ts
export default function(): string;
// @Filename: /home/src/workspaces/project/node_modules/pkg/index.js
export default /*targetDefault*/function() { return "ok"; }
// @Filename: /home/src/workspaces/project/index.ts
import /*importDefault*/myFunc from "pkg";
myFunc/*usage*/();"#;
    let _s = Session::new_for_test("goToSourceUnnamedDefaultExport", content);
    // TODO: f.VerifyBaselineGoToSourceDefinition(t, "importDefault", "usage")
}

#[test]
fn go_to_source_empty_names_entry_fallback() {
    // TODO: // getCandidateSourceDeclarationNames returns empty names,
    // TODO: // so mapDeclarationToSourceDefinitions falls through to entry declarations.
    let content = r#"// @moduleResolution: bundler
// @Filename: /home/src/workspaces/project/node_modules/pkg/package.json
{ "name": "pkg", "main": "./index.js", "types": "./index.d.ts" }
// @Filename: /home/src/workspaces/project/node_modules/pkg/index.d.ts
declare const _default: { run(): void };
export default _default;
// @Filename: /home/src/workspaces/project/node_modules/pkg/index.js
export default { run() {} };
// @Filename: /home/src/workspaces/project/index.ts
import /*defaultImport*/pkg from "pkg";
pkg.run();"#;
    let _s = Session::new_for_test("goToSourceEmptyNamesEntryFallback", content);
    // TODO: f.VerifyBaselineGoToSourceDefinition(t, "defaultImport")
}

#[test]
fn go_to_source_export_assignment_default() {
    // TODO: // ExportAssignment/default path in findDeclarationNodesByName
    // TODO: // and getCandidateSourceDeclarationNames.
    let content = r#"// @moduleResolution: bundler
// @Filename: /home/src/workspaces/project/node_modules/pkg/package.json
{ "name": "pkg", "main": "./index.js", "types": "./index.d.ts" }
// @Filename: /home/src/workspaces/project/node_modules/pkg/index.d.ts
declare const _default: { run(): void };
export default _default;
// @Filename: /home/src/workspaces/project/node_modules/pkg/index.js
/*target*/export default { run() {} };
// @Filename: /home/src/workspaces/project/index.ts
import pkg from "pkg";
pkg/*usage*/;"#;
    let _s = Session::new_for_test("goToSourceExportAssignmentDefault", content);
    // TODO: f.VerifyBaselineGoToSourceDefinition(t, "usage")
}

#[test]
fn go_to_source_export_assignment() {
    // TODO: // findDeclarationNodesByName finds export assignment (export = ...)
    // TODO: // when searching for "default".
    let content = r#"// @moduleResolution: bundler
// @Filename: /home/src/workspaces/project/node_modules/legacy/package.json
{ "name": "legacy", "main": "./index.js", "types": "./index.d.ts" }
// @Filename: /home/src/workspaces/project/node_modules/legacy/index.d.ts
declare function legacyFn(): string;
export = legacyFn;
// @Filename: /home/src/workspaces/project/node_modules/legacy/index.js
function /*targetFn*/legacyFn() { return "ok"; }
module.exports = legacyFn;
// @Filename: /home/src/workspaces/project/index.ts
import /*importName*/legacyFn from "legacy";
legacyFn();"#;
    let _s = Session::new_for_test("goToSourceExportAssignment", content);
    // TODO: f.VerifyBaselineGoToSourceDefinition(t, "importName")
}

#[test]
fn go_to_source_export_assignment_expression() {
    let content = r#"// @moduleResolution: bundler
// @Filename: /home/src/workspaces/project/node_modules/pkg/package.json
{ "name": "pkg", "main": "./index.js", "types": "./index.d.ts" }
// @Filename: /home/src/workspaces/project/node_modules/pkg/index.d.ts
export default function createThing(): { value: number };
// @Filename: /home/src/workspaces/project/node_modules/pkg/index.js
export default function createThing() { return { value: 42 }; }
// @Filename: /home/src/workspaces/project/index.ts
import /*defaultName*/createThing from "pkg";
createThing/*callDefault*/();"#;
    let _s = Session::new_for_test("goToSourceExportAssignmentExpression", content);
    // TODO: f.VerifyBaselineGoToSourceDefinition(t, "defaultName", "callDefault")
}
