use std::borrow::Cow;
use std::sync::LazyLock;

use regex::Regex;

static ANSI_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\x1b\[[0-9;?]*[a-zA-Z]").unwrap());

/// Strips ANSI CSI escape sequences (color, cursor movement, line-clear,
/// etc.) — what color/progress libraries emit. Used only for the logged
/// copy of a line; the live terminal tee stays raw.
pub fn strip(input: &str) -> Cow<'_, str> {
    ANSI_RE.replace_all(input, "")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_sgr_color_codes() {
        assert_eq!(strip("\x1b[31mred\x1b[0m"), "red");
    }

    #[test]
    fn leaves_plain_text_untouched() {
        assert_eq!(strip("plain text"), "plain text");
    }
}
