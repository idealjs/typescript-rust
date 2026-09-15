use tsox_lsp::fourslash::{self, Session};


#[test]
fn completions_literal_overload() {
    let content = r#"// @allowJs: true
// @Filename: /a.tsx
interface Events {
  "": any;
  drag: any;
  dragenter: any;
}
declare function addListener<K extends keyof Events>(type: K, listener: (ev: Events[K]) => any): void;

declare function ListenerComponent<K extends keyof Events>(props: { type: K, onWhatever: (ev: Events[K]) => void }): JSX.Element;

addListener("/*ts*/");
(<ListenerComponent type="/*tsx*/" />);
// @Filename: /b.js
addListener("/*js*/");"#;
    let mut s = Session::new_for_test("completionsLiteralOverload", content);
    for m in ["ts", "tsx", "js"] {
        fourslash::verify_completions_exact_at(&mut s, Some(m), &["", "drag", "dragenter"]);
    }
    fourslash::insert(&mut s, "drag");
    for m in ["ts", "tsx", "js"] {
        fourslash::verify_completions_exact_at(&mut s, Some(m), &["", "drag", "dragenter"]);
    }
}
