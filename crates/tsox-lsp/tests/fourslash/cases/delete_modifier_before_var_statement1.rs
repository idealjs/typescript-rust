use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.DeleteAtCaret"]
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
    let mut s = Session::new(content);
    fourslash::unsupported("GoToFileNumber"); // f.GoToFileNumber(t, 0)
    fourslash::unsupported("GoToPosition"); // f.GoToPosition(t, 0)
    fourslash::unsupported("DeleteAtCaret"); // f.DeleteAtCaret(t, 100)
    fourslash::unsupported("GoToPosition"); // f.GoToPosition(t, 198)
    fourslash::unsupported("DeleteAtCaret"); // f.DeleteAtCaret(t, 16)
    fourslash::unsupported("GoToPosition"); // f.GoToPosition(t, 198)
    fourslash::insert(&mut s, "Item(): string; ");
}
