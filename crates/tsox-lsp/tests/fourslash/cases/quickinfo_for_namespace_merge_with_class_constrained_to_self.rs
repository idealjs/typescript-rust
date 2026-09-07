use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyQuickInfoAt"]
#[test]
fn quickinfo_for_namespace_merge_with_class_constrained_to_self() {
    let content = r#"declare namespace AMap {
    namespace MassMarks {
        interface Data {
            style?: number;
        }
    }
    class MassMarks<D extends MassMarks.Data = MassMarks.Data> {
        constructor(data: D[] | string);
        clear(): void;
    }
}

interface MassMarksCustomData extends AMap.MassMarks./*1*/Data {
    name: string;
    id: string;
}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "1", "interface AMap.MassMarks<D extends AMap.MassMarks.Data = AMap.MassMarks
}
