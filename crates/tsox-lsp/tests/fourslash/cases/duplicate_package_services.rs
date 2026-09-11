use tsox_lsp::fourslash::{self, Session};


#[test]
fn duplicate_package_services() {
    let content = r#"// @noImplicitReferences: true
// @Filename: /node_modules/a/index.d.ts
import [|X/*useAX*/|] from "x";
export function a(x: X): void;
// @Filename: /node_modules/a/node_modules/x/index.d.ts
export default class /*defAX*/X {
    private x: number;
}
// @Filename: /node_modules/a/node_modules/x/package.json
{ "name": "x", "version": "1.2.3" }
// @Filename: /node_modules/b/index.d.ts
import [|X/*useBX*/|] from "x";
export const b: X;
// @Filename: /node_modules/b/node_modules/x/index.d.ts
export default class /*defBX*/X {
    private x: number;
}
// @Filename: /node_modules/b/node_modules/x/package.json
{ "name": "x", "version": "1.2.3" }
// @Filename: /src/a.ts
import { a } from "a";
import { b } from "b";
a(/*error*/b);"#;
    let mut s = Session::new_for_test("duplicatePackageServices", content);
    fourslash::go_to_file(&mut s, "/src/a.ts");
    fourslash::verify_number_of_errors_in_current_file(&mut s, 0);
    // TODO: f.VerifyBaselineFindAllReferences(t, "useAX", "defAX", "useBX")
    // TODO: f.VerifyBaselineGoToDefinition(t, true, "useAX", "useBX")
}
