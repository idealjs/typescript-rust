use super::*;
use crate::tspath::ComparePathsOptions;

fn make_generator() -> Generator {
    Generator::new("main.js", "/", "/", ComparePathsOptions::default())
}

fn raw_map(g: &mut Generator) -> RawSourceMap {
    g.raw_source_map()
}

#[test]
fn empty() {
    let mut g = make_generator();
    let map = raw_map(&mut g);
    assert_eq!(
        map,
        RawSourceMap {
            version: 3,
            file: "main.js".to_string(),
            source_root: "/".to_string(),
            sources: vec![],
            names: vec![],
            mappings: "".to_string(),
            sources_content: vec![],
        }
    );
}

#[test]
fn empty_serialized() {
    let mut g = make_generator();
    let actual = g.to_json();
    let expected =
        r#"{"version":3,"file":"main.js","sourceRoot":"/","sources":[],"names":[],"mappings":""}"#;
    assert_eq!(actual, expected);
}

#[test]
fn add_source() {
    let mut g = make_generator();
    let source_index = g.add_source("/main.ts");
    let map = raw_map(&mut g);
    assert_eq!(source_index, 0);
    assert_eq!(
        map,
        RawSourceMap {
            version: 3,
            file: "main.js".to_string(),
            source_root: "/".to_string(),
            sources: vec!["main.ts".to_string()],
            names: vec![],
            mappings: "".to_string(),
            sources_content: vec![],
        }
    );
}

#[test]
fn set_source_content() {
    let mut g = make_generator();
    let source_index = g.add_source("/main.ts");
    g.set_source_content(source_index, "foo").unwrap();
    let map = raw_map(&mut g);
    assert_eq!(source_index, 0);
    assert_eq!(
        map,
        RawSourceMap {
            version: 3,
            file: "main.js".to_string(),
            source_root: "/".to_string(),
            sources: vec!["main.ts".to_string()],
            names: vec![],
            mappings: "".to_string(),
            sources_content: vec![Some("foo".to_string())],
        }
    );
}

#[test]
fn set_source_content_for_second_source_only() {
    let mut g = make_generator();
    g.add_source("/skipped.ts");
    let source_index = g.add_source("/main.ts");
    g.set_source_content(source_index, "foo").unwrap();
    let map = raw_map(&mut g);
    assert_eq!(source_index, 1);
    assert_eq!(
        map,
        RawSourceMap {
            version: 3,
            file: "main.js".to_string(),
            source_root: "/".to_string(),
            sources: vec!["skipped.ts".to_string(), "main.ts".to_string()],
            names: vec![],
            mappings: "".to_string(),
            sources_content: vec![None, Some("foo".to_string())],
        }
    );
}

#[test]
fn set_source_content_source_index_out_of_range() {
    let mut g = make_generator();
    assert_eq!(
        g.set_source_content(-1, "").unwrap_err(),
        "sourceIndex is out of range"
    );
    assert_eq!(
        g.set_source_content(0, "").unwrap_err(),
        "sourceIndex is out of range"
    );
}

#[test]
fn set_source_content_for_second_source_only_serialized() {
    let mut g = make_generator();
    g.add_source("/skipped.ts");
    let source_index = g.add_source("/main.ts");
    g.set_source_content(source_index, "foo").unwrap();
    let actual = g.to_json();
    let expected = r#"{"version":3,"file":"main.js","sourceRoot":"/","sources":["skipped.ts","main.ts"],"names":[],"mappings":"","sourcesContent":[null,"foo"]}"#;
    assert_eq!(actual, expected);
}

#[test]
fn add_name() {
    let mut g = make_generator();
    let name_index = g.add_name("foo");
    let map = raw_map(&mut g);
    assert_eq!(name_index, 0);
    assert_eq!(
        map,
        RawSourceMap {
            version: 3,
            file: "main.js".to_string(),
            source_root: "/".to_string(),
            sources: vec![],
            names: vec!["foo".to_string()],
            mappings: "".to_string(),
            sources_content: vec![],
        }
    );
}

#[test]
fn add_generated_mapping() {
    let mut g = make_generator();
    g.add_generated_mapping(0, 0).unwrap();
    let map = raw_map(&mut g);
    assert_eq!(map.mappings, "A");
}

#[test]
fn add_generated_mapping_on_second_line_only() {
    let mut g = make_generator();
    g.add_generated_mapping(1, 0).unwrap();
    let map = raw_map(&mut g);
    assert_eq!(map.mappings, ";A");
}

#[test]
fn add_source_mapping() {
    let mut g = make_generator();
    let source_index = g.add_source("/main.ts");
    g.add_source_mapping(0, 0, source_index, 0, 0).unwrap();
    let map = raw_map(&mut g);
    assert_eq!(map.mappings, "AAAA");
}

#[test]
fn add_source_mapping_next_generated_character() {
    let mut g = make_generator();
    let source_index = g.add_source("/main.ts");
    g.add_source_mapping(0, 0, source_index, 0, 0).unwrap();
    g.add_source_mapping(0, 1, source_index, 0, 0).unwrap();
    let map = raw_map(&mut g);
    assert_eq!(map.mappings, "AAAA,CAAA");
}

#[test]
fn add_source_mapping_next_generated_and_source_character() {
    let mut g = make_generator();
    let source_index = g.add_source("/main.ts");
    g.add_source_mapping(0, 0, source_index, 0, 0).unwrap();
    g.add_source_mapping(0, 1, source_index, 0, 1).unwrap();
    let map = raw_map(&mut g);
    assert_eq!(map.mappings, "AAAA,CAAC");
}

#[test]
fn add_source_mapping_next_generated_line() {
    let mut g = make_generator();
    let source_index = g.add_source("/main.ts");
    g.add_source_mapping(0, 0, source_index, 0, 0).unwrap();
    g.add_source_mapping(1, 0, source_index, 0, 0).unwrap();
    let map = raw_map(&mut g);
    assert_eq!(map.mappings, "AAAA;AAAA");
}

#[test]
fn add_source_mapping_previous_source_character() {
    let mut g = make_generator();
    let source_index = g.add_source("/main.ts");
    g.add_source_mapping(0, 0, source_index, 0, 1).unwrap();
    g.add_source_mapping(0, 1, source_index, 0, 0).unwrap();
    let map = raw_map(&mut g);
    assert_eq!(map.mappings, "AAAC,CAAD");
}

#[test]
fn add_named_source_mapping() {
    let mut g = make_generator();
    let source_index = g.add_source("/main.ts");
    let name_index = g.add_name("foo");
    g.add_named_source_mapping(0, 0, source_index, 0, 0, name_index)
        .unwrap();
    let map = raw_map(&mut g);
    assert_eq!(map.mappings, "AAAAA");
    assert_eq!(map.names, vec!["foo".to_string()]);
}

#[test]
fn add_named_source_mapping_with_previous_name() {
    let mut g = make_generator();
    let source_index = g.add_source("/main.ts");
    let name_index1 = g.add_name("foo");
    let name_index2 = g.add_name("bar");
    g.add_named_source_mapping(0, 0, source_index, 0, 0, name_index2)
        .unwrap();
    g.add_named_source_mapping(0, 1, source_index, 0, 0, name_index1)
        .unwrap();
    let map = raw_map(&mut g);
    assert_eq!(map.mappings, "AAAAC,CAAAD");
    assert_eq!(map.names, vec!["foo".to_string(), "bar".to_string()]);
}

#[test]
fn add_generated_mapping_generated_line_cannot_backtrack() {
    let mut g = make_generator();
    g.add_generated_mapping(1, 0).unwrap();
    assert_eq!(
        g.add_generated_mapping(0, 0).unwrap_err(),
        "generatedLine cannot backtrack"
    );
}

#[test]
fn add_generated_mapping_generated_character_cannot_be_negative() {
    let mut g = make_generator();
    g.add_generated_mapping(0, 0).unwrap();
    assert_eq!(
        g.add_generated_mapping(0, -1).unwrap_err(),
        "generatedCharacter cannot be negative"
    );
}

#[test]
fn add_source_mapping_generated_line_cannot_backtrack() {
    let mut g = make_generator();
    let source_index = g.add_source("/main.ts");
    g.add_source_mapping(1, 0, source_index, 0, 0).unwrap();
    assert_eq!(
        g.add_source_mapping(0, 0, source_index, 0, 0).unwrap_err(),
        "generatedLine cannot backtrack"
    );
}

#[test]
fn add_source_mapping_generated_character_cannot_be_negative() {
    let mut g = make_generator();
    let source_index = g.add_source("/main.ts");
    g.add_source_mapping(0, 0, source_index, 0, 0).unwrap();
    assert_eq!(
        g.add_source_mapping(0, -1, source_index, 0, 0).unwrap_err(),
        "generatedCharacter cannot be negative"
    );
}

#[test]
fn add_source_mapping_source_index_is_out_of_range() {
    let mut g = make_generator();
    assert_eq!(
        g.add_source_mapping(0, 0, -1, 0, 0).unwrap_err(),
        "sourceIndex is out of range"
    );
    assert_eq!(
        g.add_source_mapping(0, 0, 0, 0, 0).unwrap_err(),
        "sourceIndex is out of range"
    );
}

#[test]
fn add_source_mapping_source_line_cannot_be_negative() {
    let mut g = make_generator();
    let source_index = g.add_source("/main.ts");
    assert_eq!(
        g.add_source_mapping(0, 0, source_index, -1, 0).unwrap_err(),
        "sourceLine cannot be negative"
    );
}

#[test]
fn add_source_mapping_source_character_cannot_be_negative() {
    let mut g = make_generator();
    let source_index = g.add_source("/main.ts");
    assert_eq!(
        g.add_source_mapping(0, 0, source_index, 0, -1).unwrap_err(),
        "sourceCharacter cannot be negative"
    );
}

#[test]
fn add_named_source_mapping_generated_line_cannot_backtrack() {
    let mut g = make_generator();
    let source_index = g.add_source("/main.ts");
    let name_index = g.add_name("foo");
    g.add_named_source_mapping(1, 0, source_index, 0, 0, name_index)
        .unwrap();
    assert_eq!(
        g.add_named_source_mapping(0, 0, source_index, 0, 0, name_index)
            .unwrap_err(),
        "generatedLine cannot backtrack"
    );
}

#[test]
fn add_named_source_mapping_generated_character_cannot_be_negative() {
    let mut g = make_generator();
    let source_index = g.add_source("/main.ts");
    let name_index = g.add_name("foo");
    g.add_named_source_mapping(0, 0, source_index, 0, 0, name_index)
        .unwrap();
    assert_eq!(
        g.add_named_source_mapping(0, -1, source_index, 0, 0, name_index)
            .unwrap_err(),
        "generatedCharacter cannot be negative"
    );
}

#[test]
fn add_named_source_mapping_source_index_is_out_of_range() {
    let mut g = make_generator();
    let name_index = g.add_name("foo");
    assert_eq!(
        g.add_named_source_mapping(0, 0, -1, 0, 0, name_index)
            .unwrap_err(),
        "sourceIndex is out of range"
    );
    assert_eq!(
        g.add_named_source_mapping(0, 0, 0, 0, 0, name_index)
            .unwrap_err(),
        "sourceIndex is out of range"
    );
}

#[test]
fn add_named_source_mapping_source_line_cannot_be_negative() {
    let mut g = make_generator();
    let name_index = g.add_name("foo");
    let source_index = g.add_source("/main.ts");
    assert_eq!(
        g.add_named_source_mapping(0, 0, source_index, -1, 0, name_index)
            .unwrap_err(),
        "sourceLine cannot be negative"
    );
}

#[test]
fn add_named_source_mapping_source_character_cannot_be_negative() {
    let mut g = make_generator();
    let name_index = g.add_name("foo");
    let source_index = g.add_source("/main.ts");
    assert_eq!(
        g.add_named_source_mapping(0, 0, source_index, 0, -1, name_index)
            .unwrap_err(),
        "sourceCharacter cannot be negative"
    );
}

#[test]
fn add_named_source_mapping_name_index_is_out_of_range() {
    let mut g = make_generator();
    let source_index = g.add_source("/main.ts");
    assert_eq!(
        g.add_named_source_mapping(0, 0, source_index, 0, 0, -1)
            .unwrap_err(),
        "nameIndex is out of range"
    );
    assert_eq!(
        g.add_named_source_mapping(0, 0, source_index, 0, 0, 0)
            .unwrap_err(),
        "nameIndex is out of range"
    );
}

#[test]
fn decoder_empty() {
    let decoder = MappingsDecoder::new("");
    let (mappings, err) = decoder.collect_all();
    assert!(mappings.is_empty());
    assert!(err.is_none());
}

#[test]
fn decoder_single_generated_mapping() {
    let decoder = MappingsDecoder::new("A");
    let (mappings, err) = decoder.collect_all();
    assert!(err.is_none());
    assert_eq!(mappings.len(), 1);
    assert_eq!(
        mappings[0],
        Mapping {
            generated_line: 0,
            generated_character: 0,
            source_index: MISSING_SOURCE,
            source_line: MISSING_LINE_OR_COLUMN,
            source_character: MISSING_UTF16_COLUMN,
            name_index: MISSING_NAME,
        }
    );
}

#[test]
fn decoder_single_source_mapping() {
    let decoder = MappingsDecoder::new("AAAA");
    let (mappings, err) = decoder.collect_all();
    assert!(err.is_none());
    assert_eq!(mappings.len(), 1);
    assert!(mappings[0].is_source_mapping());
    assert_eq!(mappings[0].source_index, 0);
    assert_eq!(mappings[0].source_line, 0);
    assert_eq!(mappings[0].source_character, 0);
}

#[test]
fn decoder_two_lines() {
    let decoder = MappingsDecoder::new("AAAA;AAAA");
    let (mappings, err) = decoder.collect_all();
    assert!(err.is_none());
    assert_eq!(mappings.len(), 2);
    assert_eq!(mappings[0].generated_line, 0);
    assert_eq!(mappings[1].generated_line, 1);
}

#[test]
fn decoder_roundtrip() {
    let mut g = make_generator();
    let source_index = g.add_source("/main.ts");
    let name_index = g.add_name("foo");
    g.add_source_mapping(0, 0, source_index, 0, 0).unwrap();
    g.add_source_mapping(0, 5, source_index, 0, 3).unwrap();
    g.add_named_source_mapping(1, 0, source_index, 1, 0, name_index)
        .unwrap();
    let map = raw_map(&mut g);

    let decoder = MappingsDecoder::new(&map.mappings);
    let (mappings, err) = decoder.collect_all();
    assert!(err.is_none());
    assert_eq!(mappings.len(), 3);

    assert_eq!(mappings[0].generated_line, 0);
    assert_eq!(mappings[0].generated_character, 0);
    assert_eq!(mappings[0].source_line, 0);
    assert_eq!(mappings[0].source_character, 0);

    assert_eq!(mappings[1].generated_line, 0);
    assert_eq!(mappings[1].generated_character, 5);
    assert_eq!(mappings[1].source_line, 0);
    assert_eq!(mappings[1].source_character, 3);

    assert_eq!(mappings[2].generated_line, 1);
    assert_eq!(mappings[2].generated_character, 0);
    assert_eq!(mappings[2].source_line, 1);
    assert_eq!(mappings[2].source_character, 0);
    assert_eq!(mappings[2].name_index, 0);
}

#[test]
fn try_get_source_mapping_url_finds_comment() {
    let text = "var x = 1;\n//# sourceMappingURL=app.js.map\n";

    let line_starts = vec![0, 11, 42];
    assert_eq!(try_get_source_mapping_url(text, &line_starts), "app.js.map");
}

#[test]
fn try_get_source_mapping_url_no_comment() {
    let text = "var x = 1;\n";
    let line_starts = vec![0, 11];
    assert_eq!(try_get_source_mapping_url(text, &line_starts), "");
}
