use tsox_lsp::fourslash::Session;


#[test]
fn go_to_implementation_interface_object_literal() {
    let content = r#"
// @Filename: /file1.ts
export interface MyInterface { P: number; }

// @Filename: /file2.ts
import { MyInterface } from "./file1";

const x: /*impl*/MyInterface = { P: 2 };
"#;
    let _s = Session::new_for_test("goToImplementationInterfaceObjectLiteral", content);
    // TODO: f.VerifyBaselineGoToImplementation(t, "impl")
}
