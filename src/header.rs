use std::collections::HashMap;

use crate::Row;

#[derive(Debug, PartialEq, Eq)]
pub struct NoHeader;

#[derive(Debug, PartialEq, Eq, Default)]
pub struct WithHeader {
    header: HashMap<String, usize>,
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
}
