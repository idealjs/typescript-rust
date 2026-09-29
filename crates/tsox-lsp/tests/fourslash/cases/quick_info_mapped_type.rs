use tsox_lsp::fourslash::{self, Session};


#[ignore = "go: t.Skip('Known failing fourslash test')"]
#[test]
fn quick_info_mapped_type() {
    let content = r#"interface I {
  /** m documentation */ m(): void;
}
declare const o: { [K in keyof I]: number };
o.m/*0*/;

declare const p: { [K in keyof I]: I[K] };
p.m/*1*/;

declare const q: Pick<I, "m">;
q.m/*2*/;"#;
    let mut s = Session::new_for_test("quickInfoMappedType", content);
    fourslash::verify_quick_info_at(&mut s, "0", "(property) m: number", "m documentation");
    fourslash::verify_quick_info_at(&mut s, "1", "(method) m(): void", "m documentation");
    fourslash::verify_quick_info_at(&mut s, "2", "(method) m(): void", "m documentation");
}
