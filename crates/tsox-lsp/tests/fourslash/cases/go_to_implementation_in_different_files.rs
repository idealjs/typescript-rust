use tsox_lsp::fourslash::{self, Session};


#[test]
fn go_to_implementation_in_different_files() {
    let content = r#"// @lib: es5
// @Filename: /home/src/workspaces/project/bar.ts
import {Foo} from './foo'

class [|A|] implements Foo {
    func() {}
}

class [|B|] implements Foo {
    func() {}
}
// @Filename: /home/src/workspaces/project/foo.ts
export interface /**/Foo {
    func();
}"#;
    let mut s = Session::new_for_test("goToImplementation_inDifferentFiles", content);
    // TODO: f.MarkTestAsStradaServer()
    // TODO: f.VerifyBaselineGoToImplementation(t, "")
}
