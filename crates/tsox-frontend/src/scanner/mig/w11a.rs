use super::super::token_to_string::Scanner;
use tsox_core::stringutil::is_white_space_like;

impl Scanner {
    pub(crate) fn token_start(&self) -> usize { ::tsox_core::fntrace::enter("token_start"); 
        self.token_pos
    }

    pub(crate) fn token_full_start(&self) -> usize { ::tsox_core::fntrace::enter("token_full_start"); 
        self.full_start_pos
    }
}

pub(crate) fn strip_leading_jsdoc_comment(line: &str) -> String { ::tsox_core::fntrace::enter("strip_leading_jsdoc_comment"); 
    let line = line.trim_start_matches(|c| is_white_space_like(c));
    let line = if let Some(rest) = line.strip_prefix('*') {
        rest
    } else {
        line
    };
    line.trim_start_matches(|c| is_white_space_like(c)).to_string()
}
