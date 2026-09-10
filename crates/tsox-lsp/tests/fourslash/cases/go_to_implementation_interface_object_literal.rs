use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineGoToImplementation"]
#[test]
fn go_to_implementation_interface_object_literal() {
    let content = r#"
// @Filename: /file1.ts
export interface MyInterface { P: number; }

// @Filename: /file2.ts
import { MyInterface } from "./file1";

const x: /*impl*/MyInterface = { P: 2 };
"#;
    let mut s = Session::new_for_test("goToImplementationInterfaceObjectLiteral", content);
    fourslash::unsupported("VerifyBaselineGoToImplementation"); // f.VerifyBaselineGoToImplementation(t, "impl")
}
