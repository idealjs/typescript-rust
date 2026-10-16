use tsox_lsp::fourslash::Session;


#[test]
fn go_to_source_definition_type_only_import_falls_back_to_declaration() {
    // TODO: // When source definition is invoked on a type-only symbol (e.g. an
    // TODO: // corresponding declaration. Source definition should fall back to the
    // TODO: // .d.ts declaration rather than jumping to the first line of the .js file.
    let content = r#"// @moduleResolution: bundler
// @Filename: /home/src/workspaces/project/node_modules/pkg/package.json
{ "name": "pkg", "main": "./index.js", "types": "./index.d.ts" }
// @Filename: /home/src/workspaces/project/node_modules/pkg/index.d.ts
export interface /*targetDecl*/Config {
    name: string;
    value: number;
}
export declare function create(config: Config): void;
// @Filename: /home/src/workspaces/project/node_modules/pkg/index.js
export function create(config) { return config; }
// @Filename: /home/src/workspaces/project/index.ts
import { /*importConfig*/Config, create } from "pkg";
const c: Config = { name: "test", value: 1 };
create(c);"#;
    let _s = Session::new_for_test("goToSourceDefinitionTypeOnlyImportFallsBackToDeclaration", content);
    // TODO: f.VerifyBaselineGoToSourceDefinition(t, "importConfig")
}

#[test]
fn go_to_source_definition_type_only_usage_falls_back_to_declaration() {
    // TODO: // When source definition is invoked at a usage site of a type-only symbol,
    // TODO: // the checker path finds the .d.ts declarations but mapDeclarationToSource
    // TODO: // finds nothing in the .js file. The result should fall back to regular
    // TODO: // definition (the .d.ts declaration).
    let content = r#"// @moduleResolution: bundler
// @Filename: /home/src/workspaces/project/node_modules/pkg/package.json
{ "name": "pkg", "main": "./index.js", "types": "./index.d.ts" }
// @Filename: /home/src/workspaces/project/node_modules/pkg/index.d.ts
export interface /*targetDecl*/Config {
    name: string;
}
export declare function create(config: Config): void;
// @Filename: /home/src/workspaces/project/node_modules/pkg/index.js
export function create(config) { return config; }
// @Filename: /home/src/workspaces/project/index.ts
import { Config, create } from "pkg";
const c: /*usageSite*/Config = { name: "test" };
create(c);"#;
    let _s = Session::new_for_test("goToSourceDefinitionTypeOnlyUsageFallsBackToDeclaration", content);
    // TODO: f.VerifyBaselineGoToSourceDefinition(t, "usageSite")
}

#[test]
fn go_to_source_definition_value_import_still_works() {
    // TODO: // Value imports (functions, classes, variables) should still navigate
    // TODO: // to the .js implementation, not regress to .d.ts.
    let content = r#"// @moduleResolution: bundler
// @Filename: /home/src/workspaces/project/node_modules/pkg/package.json
{ "name": "pkg", "main": "./index.js", "types": "./index.d.ts" }
// @Filename: /home/src/workspaces/project/node_modules/pkg/index.d.ts
export declare function /*dtsCreate*/create(): void;
// @Filename: /home/src/workspaces/project/node_modules/pkg/index.js
export function /*targetCreate*/create() {}
// @Filename: /home/src/workspaces/project/index.ts
import { /*importCreate*/create } from "pkg";
create();"#;
    let _s = Session::new_for_test("goToSourceDefinitionValueImportStillWorks", content);
    // TODO: f.VerifyBaselineGoToSourceDefinition(t, "importCreate")
}
