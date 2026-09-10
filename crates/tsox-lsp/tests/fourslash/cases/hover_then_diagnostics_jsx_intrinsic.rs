use tsox_lsp::fourslash::{self, Session};


#[ignore = "generator: // ...then a subsequent diagnostic pull must not invent a TS"]
#[test]
fn hover_then_diagnostics_jsx_intrinsic() {
    let content = r#"// @Filename: /tsconfig.json
{ "compilerOptions": { "strict": true, "jsx": "preserve" } }
// @Filename: /jsx.d.ts
declare namespace JSX {
    interface Element { }
    interface IntrinsicElements {
        div: any;
    }
}
// @Filename: /file.tsx
export default function Home() {
    return <di/*1*/v>hi</div>;
}"#;
    let mut s = Session::new_for_test("hoverThenDiagnosticsJsxIntrinsic", content);
    // TODO: // Hover on the intrinsic element first...
    fourslash::verify_quick_info_at(&mut s, "1", "(property) JSX.IntrinsicElements.div: any", "");
    // TODO: // ...then a subsequent diagnostic pull must not invent a TS2304 for `div`.
    fourslash::verify_no_errors(&mut s, );
}
