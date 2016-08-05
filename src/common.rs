use std::collections::{HashSet, HashMap};
use std::cmp::Eq;
use std::hash::{Hash, BuildHasher};

/// Splits a string at the first whitespace and returns a tuple of the parts, both trimmed.
pub fn str_head_tail(s: &str) -> (String, String) {
    let mut parts = s.splitn(2, |c: char| c.is_whitespace());
    let first = parts.next().unwrap_or("").trim().to_owned();
    let second = parts.next().unwrap_or("").trim().to_owned();
    (first, second)
}

pub trait Retain<T> {
    fn retain<F>(&mut self, f: F) where F: Fn(&T) -> bool;
}

impl<T, S> Retain<T> for HashSet<T, S>
    where T: Eq + Hash,
          S: BuildHasher
{
    fn retain<F>(&mut self, f: F)
        where F: Fn(&T) -> bool
    {
        // TODO is there a way without using a temp set?
        let to_retain = self.drain().filter(f).collect::<HashSet<T>>();
        for e in to_retain {
            self.insert(e);
        }
    }
}

impl<K, V, S> Retain<K> for HashMap<K, V, S>
    where K: Eq + Hash,
          S: BuildHasher
{
    fn retain<F>(&mut self, f: F)
        where F: Fn(&K) -> bool
    {
        // TODO is there a way without using a temp map?
        let to_retain = self.drain().filter(|&(ref k, _)| f(k)).collect::<HashMap<K, V>>();
        for (k, v) in to_retain {
            self.insert(k, v);
        }
    }
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
mod tests_hash_set_retain {
    use super::Retain;
    use std::collections::HashSet;

    #[test]
    fn retains_matching() {
        let mut v = vec![-100, -3, 0, 1, 2, 3, 100, 101];
        let mut set = v.drain(..).collect::<HashSet<i32>>();
        set.retain(|&i| i > 1);
        let mut expected = vec![2, 3, 100, 101];
        assert_eq!(expected.drain(..).collect::<HashSet<i32>>(), set);
    }

    #[test]
    fn clears_if_all_match() {
        let mut v = vec![-100, -3, 0, 1, 2, 3, 100, 101];
        let mut set = v.drain(..).collect::<HashSet<i32>>();
        set.retain(|&i| i > 1000);
        let mut expected = vec![];
        assert_eq!(expected.drain(..).collect::<HashSet<i32>>(), set);
    }

    #[test]
    fn noop_if_all_match() {
        let mut v = vec![-100, -3, 0, 1, 2, 3, 100, 101];
        let mut set = v.drain(..).collect::<HashSet<i32>>();
        set.retain(|&i| i > -1000);
        let mut expected = vec![-100, -3, 0, 1, 2, 3, 100, 101];
        assert_eq!(expected.drain(..).collect::<HashSet<i32>>(), set);
    }
}

#[cfg(test)]
mod tests_hash_map_retain {
    use super::Retain;
    use std::collections::HashMap;

    #[test]
    fn retains_matching() {
        let mut v = vec![(-100, 0), (-3, 0), (0, 0), (1, 0), (2, 0), (3, 0), (100, 0), (101, 0)];
        let mut map = v.drain(..).collect::<HashMap<i32, i32>>();
        map.retain(|&i| i > 1);
        let mut expected = vec![(2, 0), (3, 0), (100, 0), (101, 0)];
        assert_eq!(expected.drain(..).collect::<HashMap<i32, i32>>(), map);
    }

    #[test]
    fn clears_if_all_match() {
        let mut v = vec![(-100, 0), (-3, 0), (0, 0), (1, 0), (2, 0), (3, 0), (100, 0), (101, 0)];
        let mut map = v.drain(..).collect::<HashMap<i32, i32>>();
        map.retain(|&i| i > 1000);
        let mut expected = vec![];
        assert_eq!(expected.drain(..).collect::<HashMap<i32, i32>>(), map);
    }

    #[test]
    fn noop_if_all_match() {
        let mut v = vec![(-100, 0), (-3, 0), (0, 0), (1, 0), (2, 0), (3, 0), (100, 0), (101, 0)];
        let mut map = v.drain(..).collect::<HashMap<i32, i32>>();
        map.retain(|&i| i > -1000);
        let mut expected = vec![(-100, 0), (-3, 0), (0, 0), (1, 0), (2, 0), (3, 0), (100, 0),
                                (101, 0)];
        assert_eq!(expected.drain(..).collect::<HashMap<i32, i32>>(), map);
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
