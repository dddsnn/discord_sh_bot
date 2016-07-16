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
}

#[cfg(test)]
mod tests_split_whitespace_with_rest {
    use super::SplitWhitespaceWithRest;

    #[test]
    fn one_space() {
        let mut s = SplitWhitespaceWithRest::new("asd sDf DfG");
        assert_eq!(Some("asd sDf DfG"), s.rest());
        assert_eq!(Some("asd"), s.next());
        assert_eq!(Some("sDf DfG"), s.rest());
        assert_eq!(Some("sDf"), s.next());
        assert_eq!(Some("DfG"), s.rest());
        assert_eq!(Some("DfG"), s.next());
        assert_eq!(None, s.rest());
        assert_eq!(None, s.next());
        assert_eq!(None, s.rest());
    }

    #[test]
    fn multi_spaces() {
        let mut s = SplitWhitespaceWithRest::new("asd   sDf\t DfG");
        assert_eq!(Some("asd   sDf\t DfG"), s.rest());
        assert_eq!(Some("asd"), s.next());
        assert_eq!(Some("sDf\t DfG"), s.rest());
        assert_eq!(Some("sDf"), s.next());
        assert_eq!(Some("DfG"), s.rest());
        assert_eq!(Some("DfG"), s.next());
        assert_eq!(None, s.rest());
        assert_eq!(None, s.next());
        assert_eq!(None, s.rest());
    }

    #[test]
    fn with_surrounding_space() {
        let mut s = SplitWhitespaceWithRest::new("   asd   sDf\t DfG\t \t");
        assert_eq!(Some("asd   sDf\t DfG\t \t"), s.rest());
        assert_eq!(Some("asd"), s.next());
        assert_eq!(Some("sDf\t DfG\t \t"), s.rest());
        assert_eq!(Some("sDf"), s.next());
        assert_eq!(Some("DfG\t \t"), s.rest());
        assert_eq!(Some("DfG"), s.next());
        assert_eq!(None, s.rest());
        assert_eq!(None, s.next());
        assert_eq!(None, s.rest());
    }
}
