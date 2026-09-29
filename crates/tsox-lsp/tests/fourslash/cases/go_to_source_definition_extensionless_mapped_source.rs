use tsox_lsp::fourslash::Session;


#[test]
fn go_to_source_definition_extensionless_mapped_source() {
    let content = r#"// @Filename: /lib/helper.d.ts
export declare function helper(): string;
//# sourceMappingURL=helper.d.ts.map
// @Filename: /lib/helper.d.ts.map
{"version":3,"file":"helper.d.ts","sourceRoot":"","sources":["helper"],"names":[],"mappings":"AAAA,wBAAgB,MAAM,IAAI,MAAM,CAAC"}
// @Filename: /lib/helper
export function helper(): string { return ""; }
// @Filename: /index.ts
import { /*usage*/helper } from "./lib/helper";
helper();"#;
    let _s = Session::new_for_test("goToSourceDefinitionExtensionlessMappedSource", content);
    // TODO: f.VerifyBaselineGoToSourceDefinition(t, "usage")
}
