/// Splits a string at the first whitespace and returns a tuple of the parts, both trimmed.
pub fn str_head_tail(s: &str) -> (String, String) {
    let mut parts = s.splitn(2, |c: char| c.is_whitespace());
    let first = parts.next().unwrap_or("").trim().to_owned();
    let second = parts.next().unwrap_or("").trim().to_owned();
    (first, second)
}
