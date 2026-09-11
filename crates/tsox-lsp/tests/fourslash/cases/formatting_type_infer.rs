use tsox_lsp::fourslash::{self, Session};


#[test]
fn formatting_type_infer() {
    let content = r#"
/*L1*/type C<T> = T extends Array<infer U> ? U : never;

/*L2*/  type   C  <  T  >   =   T   extends   Array   <   infer     U  >  ?   U   :   never  ; 

/*L3*/type C<T> = T extends Array<infer U> ? U : T;

/*L4*/  type   C  <  T  >   =   T   extends   Array   <   infer     U  >  ?   U   :   T  ;  

/*L5*/type Foo<T> = T extends { a: infer U, b: infer U } ? U : never;

/*L6*/  type   Foo  <  T  > = T   extends   {   a  :   infer   U  ,   b  :   infer   U   }   ?   U   :   never  ;  

/*L7*/type Bar<T> = T extends { a: (x: infer U) => void, b: (x: infer U) => void } ? U : never;

/*L8*/  type   Bar  <  T  >   =   T   extends   {   a  :   (x  :  infer  U  ) =>   void  ,   b  :   (x  :   infer   U  )   =>   void   }    ?   U   :   never  ;
"#;
    let mut s = Session::new_for_test("formattingTypeInfer", content);
    fourslash::format_document(&mut s, "");
    fourslash::go_to_marker(&mut s, "L1");
    fourslash::verify_current_line_content(&mut s, r#"type C<T> = T extends Array<infer U> ? U : never;"#);
    fourslash::go_to_marker(&mut s, "L2");
    fourslash::verify_current_line_content(&mut s, r#"type C<T> = T extends Array<infer U> ? U : never;"#);
    fourslash::go_to_marker(&mut s, "L3");
    fourslash::verify_current_line_content(&mut s, r#"type C<T> = T extends Array<infer U> ? U : T;"#);
    fourslash::go_to_marker(&mut s, "L4");
    fourslash::verify_current_line_content(&mut s, r#"type C<T> = T extends Array<infer U> ? U : T;"#);
    fourslash::go_to_marker(&mut s, "L5");
    fourslash::verify_current_line_content(&mut s, r#"type Foo<T> = T extends { a: infer U, b: infer U } ? U : never;"#);
    fourslash::go_to_marker(&mut s, "L6");
    fourslash::verify_current_line_content(&mut s, r#"type Foo<T> = T extends { a: infer U, b: infer U } ? U : never;"#);
    fourslash::go_to_marker(&mut s, "L7");
    fourslash::verify_current_line_content(&mut s, r#"type Bar<T> = T extends { a: (x: infer U) => void, b: (x: infer U) => void } ? U : never;"#);
    fourslash::go_to_marker(&mut s, "L8");
    fourslash::verify_current_line_content(&mut s, r#"type Bar<T> = T extends { a: (x: infer U) => void, b: (x: infer U) => void } ? U : never;"#);
}
