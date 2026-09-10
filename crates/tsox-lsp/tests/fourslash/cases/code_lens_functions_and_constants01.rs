use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineCodeLens"]
#[test]
fn code_lens_functions_and_constants01() {
    let content = r#"
// @module: preserve

// @filename: ./exports.ts

let callCount = 0;
export function foo(n: number): void {
  callCount++;
  if (n > 0) {
	foo(n - 1);
  }
  else {
    console.log("function was called " + callCount + " times");
  }
}

foo(5);

export const bar = 123;

// @filename: ./importer.ts
import { foo, bar } from "./exports";

foo(5);
console.log(bar);
"#;
    let mut s = Session::new_for_test("codeLensFunctionsAndConstants01", content);
    fourslash::unsupported("VerifyBaselineCodeLens"); // f.VerifyBaselineCodeLens(t, &lsutil.UserPreferences{
}
