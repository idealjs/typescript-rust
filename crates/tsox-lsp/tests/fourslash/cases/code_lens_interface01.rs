use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineCodeLens"]
#[test]
fn code_lens_interface01() {
    let content = r#"
// @module: preserve

// @filename: ./pointable.ts
export interface Pointable {
  getX(): number;
  getY(): number;
}

// @filename: ./classPointable.ts
import { Pointable } from "./pointable";

class Point implements Pointable {
  getX(): number {
    return 0;
  }
  getY(): number {
    return 0;
  }
}

// @filename: ./objectPointable.ts
import { Pointable } from "./pointable";

let x = 0;
let y = 0;
const p: Pointable = {
  getX(): number {
	return x;
  },
  getY(): number {
	return y;
  },
};
"#;
    let mut s = Session::new_for_test("codeLensInterface01", content);
    fourslash::unsupported("VerifyBaselineCodeLens"); // f.VerifyBaselineCodeLens(t, &lsutil.UserPreferences{
}
