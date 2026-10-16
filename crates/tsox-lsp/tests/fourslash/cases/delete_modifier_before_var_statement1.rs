use tsox_lsp::fourslash::{self, Session};


#[test]
fn delete_modifier_before_var_statement1() {
    let content = r#"

/////////////////////////////
/// Windows Script Host APIS
/////////////////////////////

declare var ActiveXObject: { new (s: string): any; };

interface ITextWriter {
    WriteLine(s): void;
}

declare var WScript: {
    Echo(s): void;
    StdErr: ITextWriter;
    Arguments: { length: number; Item(): string; };
    ScriptFullName: string;
    Quit(): number;
}
"#;
    let mut s = Session::new_for_test("deleteModifierBeforeVarStatement1", content);
    fourslash::go_to_position(&mut s, 0);
    fourslash::delete_at_caret(&mut s, 100);
    fourslash::go_to_position(&mut s, 198);
    fourslash::delete_at_caret(&mut s, 16);
    fourslash::go_to_position(&mut s, 198);
    fourslash::insert(&mut s, "Item(): string; ");
}
