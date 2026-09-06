
use crate::ast::SourceFile;
use crate::sourcemap::{Generator, SourceIndex};
use super::*;

pub(crate) fn generate_source_map_from_offsets(
    generator: &mut Generator,
    source_index: SourceIndex,
    output: &str,
    src_offsets: &[u32],
    source: &str,
    source_line_starts: &[usize],
    _source_file: &SourceFile,
) {

    let out_chars: Vec<char> = output.chars().collect();
    let mut gen_line: i32 = 0;
    let mut gen_col: i32 = 0;
    let mut prev_src: u32 = UNMAPPED;

    for (i, &src_off) in src_offsets.iter().enumerate() {
        let ch = out_chars.get(i).copied().unwrap_or('\n');

        if ch != '\n' && src_off != UNMAPPED {
            let should_emit = if prev_src == UNMAPPED {
                true
            } else {
                let prev_byte = prev_src as usize;
                if prev_byte < source.len() {
                    let prev_char_len = source[prev_byte..]
                        .chars()
                        .next()
                        .map(|c| c.len_utf8())
                        .unwrap_or(1);
                    src_off != prev_src + prev_char_len as u32
                } else {
                    true
                }
            };
            if should_emit {
                let byte_off = src_off as usize;
                let (src_line, line_start) = offset_to_line(source_line_starts, byte_off);
                let src_col = utf16_column(source, line_start, byte_off);
                let _ = generator.add_source_mapping(
                    gen_line,
                    gen_col,
                    source_index,
                    src_line,
                    src_col,
                );
            }
        }

        if ch == '\n' {
            gen_line += 1;
            gen_col = 0;
            prev_src = UNMAPPED;
        } else {
            gen_col += ch.len_utf16() as i32;
            prev_src = src_off;
        }
    }
}
