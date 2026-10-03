use crate::ast::node::SourceFile;
use crate::ast::utilities_synthesized::position_is_synthesized;
use crate::scanner::mig::w1::compute_line_of_position;
use crate::scanner::{skip_trivia_ex, SkipTriviaOptions};
use tsox_core::core::mig::x12::utf16_len;
use tsox_core::core::text::TextRange;
use tsox_core::stringutil::is_white_space_like;

pub trait Source {
    fn text(&self) -> &str;
    fn file_name(&self) -> &str;
    fn ecma_line_map(&self) -> Vec<i32>;
}

pub fn get_ecma_line_starts(source_file: &SourceFile) -> Vec<i32> { ::tsox_core::fntrace::enter("get_ecma_line_starts"); 
    source_file
        .line_map
        .line_starts
        .iter()
        .map(|&start| start as i32)
        .collect()
}

pub fn range_is_on_single_line(r: TextRange, source_file: &SourceFile) -> bool { ::tsox_core::fntrace::enter("range_is_on_single_line"); 
    range_start_is_on_same_line_as_range_end(r, r, source_file)
}

pub fn range_start_positions_are_on_same_line(
    range1: TextRange,
    range2: TextRange,
    source_file: &SourceFile,
) -> bool { ::tsox_core::fntrace::enter("range_start_positions_are_on_same_line"); 
    positions_are_on_same_line(
        get_start_position_of_range(range1, source_file, false),
        get_start_position_of_range(range2, source_file, false),
        source_file,
    )
}

pub fn range_end_positions_are_on_same_line(
    range1: TextRange,
    range2: TextRange,
    source_file: &SourceFile,
) -> bool { ::tsox_core::fntrace::enter("range_end_positions_are_on_same_line"); 
    positions_are_on_same_line(range1.end() as i64, range2.end() as i64, source_file)
}

pub fn range_start_is_on_same_line_as_range_end(
    range1: TextRange,
    range2: TextRange,
    source_file: &SourceFile,
) -> bool { ::tsox_core::fntrace::enter("range_start_is_on_same_line_as_range_end"); 
    positions_are_on_same_line(
        get_start_position_of_range(range1, source_file, false),
        range2.end() as i64,
        source_file,
    )
}

pub fn range_end_is_on_same_line_as_range_start(
    range1: TextRange,
    range2: TextRange,
    source_file: &SourceFile,
) -> bool { ::tsox_core::fntrace::enter("range_end_is_on_same_line_as_range_start"); 
    positions_are_on_same_line(
        range1.end() as i64,
        get_start_position_of_range(range2, source_file, false),
        source_file,
    )
}

pub fn get_start_position_of_range(r: TextRange, source_file: &SourceFile, include_comments: bool) -> i64 { ::tsox_core::fntrace::enter("get_start_position_of_range"); 
    if position_is_synthesized(r.pos()) {
        return -1;
    }
    skip_trivia_ex(
        &source_file.text,
        r.pos(),
        &SkipTriviaOptions { stop_at_comments: include_comments, ..Default::default() },
        None,
    ) as i64
}

pub fn positions_are_on_same_line(pos1: i64, pos2: i64, source_file: &SourceFile) -> bool { ::tsox_core::fntrace::enter("positions_are_on_same_line"); 
    get_lines_between_positions(source_file, pos1, pos2) == 0
}

pub fn get_lines_between_positions(source_file: &SourceFile, pos1: i64, pos2: i64) -> i64 { ::tsox_core::fntrace::enter("get_lines_between_positions"); 
    if pos1 == pos2 {
        return 0;
    }
    let line_starts = get_ecma_line_starts(source_file);
    let lower = if pos1 < pos2 { pos1 } else { pos2 };
    let is_negative = lower == pos2;
    let upper = if is_negative { pos1 } else { pos2 };
    let lower_line = compute_line_of_position(&line_starts, lower as i32) as i64;
    let upper_line =
        lower_line + compute_line_of_position(&line_starts[lower_line as usize..], upper as i32) as i64;
    if is_negative {
        lower_line - upper_line
    } else {
        upper_line - lower_line
    }
}

pub fn get_lines_between_range_end_and_range_start(
    range1: TextRange,
    range2: TextRange,
    source_file: &SourceFile,
    include_second_range_comments: bool,
) -> i64 { ::tsox_core::fntrace::enter("get_lines_between_range_end_and_range_start"); 
    let range2_start = get_start_position_of_range(range2, source_file, include_second_range_comments);
    get_lines_between_positions(source_file, range1.end() as i64, range2_start)
}

pub fn get_lines_between_position_and_preceding_non_whitespace_character(
    pos: i64,
    stop_pos: i64,
    source_file: &SourceFile,
    include_comments: bool,
) -> i64 { ::tsox_core::fntrace::enter("get_lines_between_position_and_preceding_non_whitespace_character"); 
    let start_pos = skip_trivia_ex(
        &source_file.text,
        pos.max(0) as usize,
        &SkipTriviaOptions { stop_at_comments: include_comments, ..Default::default() },
        None,
    ) as i64;
    let prev_pos = get_previous_non_whitespace_position(start_pos, stop_pos, source_file);
    get_lines_between_positions(
        source_file,
        if prev_pos >= 0 { prev_pos } else { stop_pos },
        start_pos,
    )
}

pub fn get_lines_between_position_and_next_non_whitespace_character(
    pos: i64,
    stop_pos: i64,
    source_file: &SourceFile,
    include_comments: bool,
) -> i64 { ::tsox_core::fntrace::enter("get_lines_between_position_and_next_non_whitespace_character"); 
    let next_pos = skip_trivia_ex(
        &source_file.text,
        pos.max(0) as usize,
        &SkipTriviaOptions { stop_at_comments: include_comments, ..Default::default() },
        None,
    ) as i64;
    get_lines_between_positions(source_file, pos, if stop_pos < next_pos { stop_pos } else { next_pos })
}

pub fn get_previous_non_whitespace_position(pos: i64, stop_pos: i64, source_file: &SourceFile) -> i64 { ::tsox_core::fntrace::enter("get_previous_non_whitespace_position"); 
    let bytes = source_file.text.as_bytes();
    let mut pos = pos;
    while pos >= stop_pos {
        if !is_white_space_like(bytes[pos as usize] as char) {
            return pos;
        }
        pos -= 1;
    }
    -1
}

pub struct LineCharacterCache {
    pub line_map: Vec<i32>,
    pub text: String,
    pub cached_line: i64,
    pub cached_pos: i64,
    pub cached_char: i64,
    pub has_cached: bool,
}

pub fn new_line_character_cache(source: &dyn Source) -> LineCharacterCache { ::tsox_core::fntrace::enter("new_line_character_cache"); 
    LineCharacterCache {
        line_map: source.ecma_line_map(),
        text: source.text().to_string(),
        cached_line: 0,
        cached_pos: 0,
        cached_char: 0,
        has_cached: false,
    }
}

impl LineCharacterCache {
    pub fn get_line_and_character(&mut self, pos: i64) -> (i64, i64) { ::tsox_core::fntrace::enter("get_line_and_character"); 
        let line = compute_line_of_position(&self.line_map, pos as i32) as i64;
        let line_start = self.line_map[line as usize] as i64;
        let end_pos = pos.min(self.text.len() as i64);
        let character;
        if self.has_cached && line == self.cached_line && end_pos >= self.cached_pos {
            character = self.cached_char
                + utf16_len(&self.text[self.cached_pos as usize..end_pos as usize]) as i64;
        } else {
            character = utf16_len(&self.text[line_start as usize..end_pos as usize]) as i64;
        }
        let cached_char = character;
        let character = character + (pos - end_pos);
        self.cached_line = line;
        self.cached_pos = end_pos;
        self.cached_char = cached_char;
        self.has_cached = true;
        (line, character)
    }
}
