use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyOutliningSpans"]
#[test]
fn outlining_spans_switch_cases() {
    let content = r#"switch (undefined)[| {
 case 0:[|
   console.log(1)
   console.log(2)
   break;
   console.log(3);|]
 case 1:[|
   break;|]
 case 2:[|
   break;
   console.log(3);|]
 case 3:[|
   console.log(4);|]
 
 case 4:
 case 5:
 case 6:[|


   console.log(5);|]
 
 case 7:[| console.log(6);|]

 case 8:[| [|{
   console.log(8);
   break;
 }|]
 console.log(8);|]

 default:[|
   console.log(7);
   console.log(8);|]
}|]"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyOutliningSpans"); // f.VerifyOutliningSpans(t)
}
