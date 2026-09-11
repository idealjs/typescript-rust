use tsox_lsp::fourslash::{self, Session};


#[test]
fn import_name_code_fix_types_versions() {
    let content = r#"// @module: commonjs
// @checkJs: true
// @Filename: /node_modules/unified/package.json
{
  "name": "unified",
  "types": "types/ts3.444/index.d.ts",
  "typesVersions": {
    ">=4.0": {
      "types/ts3.444/*": [
        "types/ts4.0/*"
      ]
    }
  }
}
// @Filename: /node_modules/unified/types/ts3.444/index.d.ts
export declare const x: number;
// @Filename: /node_modules/unified/types/ts4.0/index.d.ts
export declare const x: number;
// @Filename: /foo.js
import {} from "unified";
// @Filename: /index.js
x/**/"#;
    let mut s = Session::new_for_test("importNameCodeFix_typesVersions", content);
    // TODO: f.VerifyImportFixModuleSpecifiers(t, "", []string{"unified", "unified/types/ts3.444/index.js"}, &lsu
}
