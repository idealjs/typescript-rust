use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyOutliningSpans"]
#[test]
fn get_outlining_for_type_literal() {
    let content = r#"type A =[| {
    a: number;
}|]

type B =[| {
   a:[| {
       a1:[| {
           a2:[| {
               x: number;
               y: number;
           }|]
       }|]
   }|],
   b:[| {
       x: number;
   }|],
   c:[| {
       x: number;
   }|]
}|]"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyOutliningSpans"); // f.VerifyOutliningSpans(t)
}
