use std::collections::{HashMap, hash_map::Keys};

use crate::Row;

#[derive(Debug, PartialEq, Eq)]
pub struct NoHeader;

#[derive(Debug, PartialEq, Eq, Default)]
pub struct WithHeader {
    header: HashMap<String, usize>,
}

pub struct HeaderIter<'a> {
    header: Keys<'a, String, usize>
}

impl WithHeader {
    pub(super) fn new(header: Row<NoHeader>) -> Self {
        Self {
            header: header
                .data
                .into_iter()
                .enumerate()
                .map(|(i, s)| (s, i))
                .collect(),
        }
    }

    pub fn get_index(&self, key: &str) -> Option<usize> {
        self.header.get(key).copied()
    }

    pub fn iter(&self) -> HeaderIter {
        let header = self.header.keys();
        HeaderIter { header }
    }

    pub fn width(&self) -> usize {
        self.header.len()
    }
}

impl <'a> Iterator for HeaderIter<'a> {
    type Item = &'a str;
    fn next(&mut self) -> Option<Self::Item> {
        self.header.next().map(String::as_str)
    }
}

#[cfg(test)]
mod tests {

    use std::rc::Rc;

    use super::*;

    #[test]
    fn create_header() {
        let data = "a,b,c".to_string();
        let mut data: &[u8] = data.as_bytes();
        let header = Row::new(&mut data, Rc::new(NoHeader), ',')
            .unwrap()
            .unwrap();
        assert_eq!(
            WithHeader::new(header),
            WithHeader {
                header: HashMap::from_iter([
                    ("a".to_string(), 0),
                    ("b".to_string(), 1),
                    ("c".to_string(), 2),
                ]),
            }
        );
    }

    #[test]
    fn header_iter() {
        let header = WithHeader {
            header: HashMap::from_iter([
                "a".to_string(),
                "b".to_string(),
                "c".to_string()
            ].into_iter().enumerate().map(|(a,b)|(b,a)))
        };

        let mut expected_keys = vec!["a","b","c"];

        for key in header.iter() {
            if expected_keys.contains(&key) {
                expected_keys.retain(|ek| *ek != key)
            } else {
                panic!("found unexpected key: \"{}\"", key)
            }
        }
        assert!(expected_keys.is_empty())
    }

    #[test]
    fn header_width() {
        let header = WithHeader {
            header: HashMap::from_iter([
                "a".to_string(),
                "b".to_string(),
                "c".to_string()
            ].into_iter().enumerate().map(|(a,b)|(b,a)))
        };
        assert_eq!(header.width(), 3)
    }
}
