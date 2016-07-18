/// Splits a string at the first whitespace and returns a tuple of the parts, both trimmed.
pub fn str_head_tail(s: &str) -> (String, String) {
    let mut parts = s.splitn(2, |c: char| c.is_whitespace());
    let first = parts.next().unwrap_or("").trim().to_owned();
    let second = parts.next().unwrap_or("").trim().to_owned();
    (first, second)
}

pub struct SplitWhitespaceWithRest<'a> {
    string: &'a str,
    start_idx: usize,
}

impl<'a> Iterator for SplitWhitespaceWithRest<'a> {
    type Item = &'a str;

    // Invariant: self.start_idx is index of first byte of next token, or first byte after the
    // string.
    fn next(&mut self) -> Option<&'a str> {
        if self.start_idx == self.string.len() {
            return None;
        }
        // Offset from the start index to the next whitespace.
        let end_offset = self.string[self.start_idx..]
            .find(char::is_whitespace)
            .unwrap_or(self.string.len() - self.start_idx);
        let end_idx = self.start_idx + end_offset;
        let next_piece = &self.string[self.start_idx..end_idx];
        // Offset from the end index to the next non-whitespace (i.e. the beginning of the next
        // token).
        let next_start_offset = self.string[end_idx..]
            .find(|c: char| !c.is_whitespace())
            .unwrap_or(self.string.len() - end_idx);
        self.start_idx = end_idx + next_start_offset;
        Some(next_piece)
    }
}

impl<'a> SplitWhitespaceWithRest<'a> {
    pub fn new(string: &'a str) -> Self {
        let start_idx = string.find(|c: char| !c.is_whitespace())
            .unwrap_or(string.len());
        SplitWhitespaceWithRest {
            string: string,
            start_idx: start_idx,
        }
    }

    pub fn rest(&self) -> Option<&'a str> {
        if self.start_idx == self.string.len() {
            None
        } else {
            Some(&self.string[self.start_idx..])
        }
    }

    /// Rewinds the iterator, so that the next call to next() will return the same token again. The
    /// exception is the case where the previous call to next() returned None: in that case, the
    /// next call will return the last token of the string.
    pub fn rewind(&mut self) {
        let prev_end_idx = self.string[..self.start_idx]
            .rfind(|c: char| !c.is_whitespace())
            .unwrap_or(self.start_idx);
        let prev_start_idx = self.string[..prev_end_idx]
            .rfind(char::is_whitespace)
            .map(|i| i + 1)
            .unwrap_or(0);
        self.start_idx = prev_start_idx;
    }
}

#[cfg(test)]
mod tests_split_whitespace_with_rest {
    use super::SplitWhitespaceWithRest;

    #[test]
    fn next_rest_one_space() {
        let mut s = SplitWhitespaceWithRest::new("asd ßDf x DfG");
        assert_eq!(Some("asd ßDf x DfG"), s.rest());
        assert_eq!(Some("asd"), s.next());
        assert_eq!(Some("ßDf x DfG"), s.rest());
        assert_eq!(Some("ßDf"), s.next());
        assert_eq!(Some("x DfG"), s.rest());
        assert_eq!(Some("x"), s.next());
        assert_eq!(Some("DfG"), s.rest());
        assert_eq!(Some("DfG"), s.next());
        assert_eq!(None, s.rest());
        assert_eq!(None, s.next());
        assert_eq!(None, s.rest());
    }

    #[test]
    fn next_rest_multi_spaces() {
        let mut s = SplitWhitespaceWithRest::new("asd   ßDf x\t DfG");
        assert_eq!(Some("asd   ßDf x\t DfG"), s.rest());
        assert_eq!(Some("asd"), s.next());
        assert_eq!(Some("ßDf x\t DfG"), s.rest());
        assert_eq!(Some("ßDf"), s.next());
        assert_eq!(Some("x\t DfG"), s.rest());
        assert_eq!(Some("x"), s.next());
        assert_eq!(Some("DfG"), s.rest());
        assert_eq!(Some("DfG"), s.next());
        assert_eq!(None, s.rest());
        assert_eq!(None, s.next());
        assert_eq!(None, s.rest());
    }

    #[test]
    fn next_rest_with_surrounding_space() {
        let mut s = SplitWhitespaceWithRest::new(" asd   ßDf x\t DfG\t \t");
        assert_eq!(Some("asd   ßDf x\t DfG\t \t"), s.rest());
        assert_eq!(Some("asd"), s.next());
        assert_eq!(Some("ßDf x\t DfG\t \t"), s.rest());
        assert_eq!(Some("ßDf"), s.next());
        assert_eq!(Some("x\t DfG\t \t"), s.rest());
        assert_eq!(Some("x"), s.next());
        assert_eq!(Some("DfG\t \t"), s.rest());
        assert_eq!(Some("DfG"), s.next());
        assert_eq!(None, s.rest());
        assert_eq!(None, s.next());
        assert_eq!(None, s.rest());
    }

    #[test]
    fn rewind_one_space() {
        let mut s = SplitWhitespaceWithRest::new("asd ßDf x DfG");
        s.next();
        s.rewind();
        assert_eq!(Some("asd"), s.next());
        s.next();
        s.rewind();
        assert_eq!(Some("ßDf"), s.next());
        s.next();
        s.rewind();
        assert_eq!(Some("x"), s.next());
        s.next();
        s.rewind();
        assert_eq!(Some("DfG"), s.next());
        assert_eq!(None, s.next());
        s.rewind();
        s.rewind();
        s.rewind();
        s.rewind();
        assert_eq!(Some("asd"), s.next());
    }

    #[test]
    fn rewind_multi_spaces() {
        let mut s = SplitWhitespaceWithRest::new("asd   ßDf x\t DfG");
        s.next();
        s.rewind();
        assert_eq!(Some("asd"), s.next());
        s.next();
        s.rewind();
        assert_eq!(Some("ßDf"), s.next());
        s.next();
        s.rewind();
        assert_eq!(Some("x"), s.next());
        s.next();
        s.rewind();
        assert_eq!(Some("DfG"), s.next());
        assert_eq!(None, s.next());
        s.rewind();
        s.rewind();
        s.rewind();
        s.rewind();
        assert_eq!(Some("asd"), s.next());
    }

    #[test]
    fn rewind_with_surrounding_space() {
        let mut s = SplitWhitespaceWithRest::new(" asd   ßDf x\t DfG\t \t");
        s.next();
        s.rewind();
        assert_eq!(Some("asd"), s.next());
        s.next();
        s.rewind();
        assert_eq!(Some("ßDf"), s.next());
        s.next();
        s.rewind();
        assert_eq!(Some("x"), s.next());
        s.next();
        s.rewind();
        assert_eq!(Some("DfG"), s.next());
        assert_eq!(None, s.next());
        s.rewind();
        s.rewind();
        s.rewind();
        s.rewind();
        assert_eq!(Some("asd"), s.next());
    }
}
