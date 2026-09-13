/// Byte-level line assembly for a single stream (stdout or stderr).
///
/// Unlike `BufRead::lines()`, this treats a bare `\r` (no following `\n`) as
/// an overwrite signal — the buffered content is discarded, not glued onto
/// whatever comes next — which is how terminal progress bars/spinners work.
/// `\r\n` is still treated as one ordinary line terminator.
#[derive(Default)]
pub struct LineAssembler {
    pending: Vec<u8>,
    saw_cr: bool,
}

impl LineAssembler {
    pub fn feed(&mut self, chunk: &[u8], mut emit: impl FnMut(&[u8])) {
        for &b in chunk {
            if self.saw_cr {
                self.saw_cr = false;
                if b == b'\n' {
                    emit(&self.pending);
                    self.pending.clear();
                    continue;
                } else {
                    // Lone \r: the previous content was being overwritten.
                    self.pending.clear();
                }
            }

            match b {
                b'\n' => {
                    emit(&self.pending);
                    self.pending.clear();
                }
                b'\r' => {
                    self.saw_cr = true;
                }
                _ => self.pending.push(b),
            }
        }
    }

    /// Resolve any content left over after an idle gap or at EOF. An
    /// unresolved trailing `\r` (no `\n` arrived in time) is treated as a
    /// lone overwrite and discarded, matching `feed`'s behavior.
    pub fn flush_pending(&mut self) -> Option<Vec<u8>> {
        if self.saw_cr {
            self.pending.clear();
            self.saw_cr = false;
        }

        if self.pending.is_empty() {
            None
        } else {
            Some(std::mem::take(&mut self.pending))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn feed_all(chunks: &[&[u8]]) -> Vec<Vec<u8>> {
        let mut asm = LineAssembler::default();
        let mut lines = Vec::new();
        for chunk in chunks {
            asm.feed(chunk, |line| lines.push(line.to_vec()));
        }
        lines
    }

    #[test]
    fn splits_on_newline() {
        let lines = feed_all(&[b"hello\nworld\n"]);
        assert_eq!(lines, vec![b"hello".to_vec(), b"world".to_vec()]);
    }

    #[test]
    fn crlf_is_one_terminator() {
        let lines = feed_all(&[b"hello\r\nworld\r\n"]);
        assert_eq!(lines, vec![b"hello".to_vec(), b"world".to_vec()]);
    }

    #[test]
    fn crlf_split_across_chunks() {
        let lines = feed_all(&[b"hello\r", b"\nworld\n"]);
        assert_eq!(lines, vec![b"hello".to_vec(), b"world".to_vec()]);
    }

    #[test]
    fn lone_cr_discards_pending() {
        let lines = feed_all(&[b"abc\rdef\n"]);
        assert_eq!(lines, vec![b"def".to_vec()]);
    }

    #[test]
    fn lone_cr_split_across_chunks_discards_pending() {
        let lines = feed_all(&[b"abc\r", b"def\n"]);
        assert_eq!(lines, vec![b"def".to_vec()]);
    }

    #[test]
    fn flush_pending_empty_returns_none() {
        let mut asm = LineAssembler::default();
        assert_eq!(asm.flush_pending(), None);
    }

    #[test]
    fn flush_pending_returns_and_clears_buffered_content() {
        let mut asm = LineAssembler::default();
        asm.feed(b"partial", |_| panic!("no full line yet"));
        assert_eq!(asm.flush_pending(), Some(b"partial".to_vec()));
        assert_eq!(asm.flush_pending(), None);
    }

    #[test]
    fn flush_pending_resolves_unterminated_trailing_cr_as_discard() {
        let mut asm = LineAssembler::default();
        asm.feed(b"stale\r", |_| panic!("no full line yet"));
        // No \n arrived in time: the lone \r means "overwrite", so the
        // stale content before it is discarded, not flushed.
        assert_eq!(asm.flush_pending(), None);
    }
}
