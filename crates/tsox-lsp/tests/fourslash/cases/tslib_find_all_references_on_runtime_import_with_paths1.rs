use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineFindAllReferences"]
#[test]
fn tslib_find_all_references_on_runtime_import_with_paths1() {
    let content = r#"// @Filename: project/src/foo.ts
import * as x from /**/"tslib";
// @Filename: project/src/bar.ts
export default "";
// @Filename: project/src/bal.ts

// @Filename: project/src/dir/tslib.d.ts
export function __importDefault(...args: any): any;
export function __importStar(...args: any): any;
// @Filename: project/tsconfig.json
{
    "compilerOptions": {
        "moduleResolution": "node",
        "module": "es2020",
        "importHelpers": true,
        "moduleDetection": "force",
        "paths": {
            "tslib": ["./src/dir/tslib"]
        }
    }
}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineFindAllReferences"); // f.VerifyBaselineFindAllReferences(t, "")
}
