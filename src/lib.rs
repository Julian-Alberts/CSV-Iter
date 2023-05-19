mod header;
mod row;

pub use header::*;
pub use row::*;
use std::{
    io::{BufRead, Read},
    rc::Rc,
};

pub struct CSVIter<R, H = NoHeader>
where
    R: Read,
{
    header: Rc<H>,
    data: R,
    separator: char,
}

impl<R, H> CSVIter<R, H>
where
    R: Read,
{
    fn new_internal(data: R, header: H, separator: char) -> Self {
        CSVIter {
            header: Rc::new(header),
            data,
            separator,
        }
    }
}

impl<R> CSVIter<R, NoHeader>
where
    R: Read,
{
    pub fn new_without_header(data: R, field_separator: char) -> Self {
        CSVIter::new_internal(data, NoHeader, field_separator)
    }
}

impl<R> CSVIter<R, WithHeader>
where
    R: BufRead,
{
    pub fn new_with_header(mut data: R, field_separator: char) -> std::io::Result<Self> {
        let header = Row::new(&mut data, Rc::new(NoHeader), field_separator)?;
        let header = if let Some(header_row) = header
        {
            WithHeader::new(header_row)
        } else {
            WithHeader::default()
        };
        Ok(CSVIter::new_internal(data, header, field_separator))
    }

    pub fn width(&self) -> usize {
        self.header.width()
    }
}

impl<R, H> Iterator for CSVIter<R, H>
where
    R: BufRead,
{
    type Item = std::io::Result<Row<H>>;

    fn next(&mut self) -> Option<Self::Item> {
        Row::new(&mut self.data, self.header.clone(), self.separator).transpose()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_iter_without_header() {
        let data = "1,2,3\n4,5,6\n7,8,9";
        let mut iter = CSVIter::new_without_header(data.as_bytes(), ',');
        assert_eq!(iter.header, Rc::new(NoHeader));
        assert_eq!(iter.data, data.as_bytes());
        let row = iter.next().unwrap().unwrap();
        assert_eq!(row.get(0), Some("1"));
        assert_eq!(row.get(1), Some("2"));
        assert_eq!(row.get(2), Some("3"));
        let row = iter.next().unwrap().unwrap();
        assert_eq!(row.get(0), Some("4"));
        assert_eq!(row.get(1), Some("5"));
        assert_eq!(row.get(2), Some("6"));
        let row = iter.next().unwrap().unwrap();
        assert_eq!(row.get(0), Some("7"));
        assert_eq!(row.get(1), Some("8"));
        assert_eq!(row.get(2), Some("9"));
        assert!(iter.next().is_none());
    }

    #[test]
    fn create_iter_with_header() {
        let data = "a,b,c\n1,2,3\n4,5,6\n7,8,9";
        let mut iter = CSVIter::new_with_header(data.as_bytes(), ',').unwrap();
        assert_eq!(iter.data, data[6..].as_bytes());
        let row = iter.next();
        let row = row.unwrap().unwrap();
        assert_eq!(row.get_by_key("a"), Some("1"));
        assert_eq!(row.get_by_key("b"), Some("2"));
        assert_eq!(row.get_by_key("c"), Some("3"));
        let row = iter.next().unwrap().unwrap();
        assert_eq!(row.get_by_key("a"), Some("4"));
        assert_eq!(row.get_by_key("b"), Some("5"));
        assert_eq!(row.get_by_key("c"), Some("6"));
        let row = iter.next().unwrap().unwrap();
        assert_eq!(row.get_by_key("a"), Some("7"));
        assert_eq!(row.get_by_key("b"), Some("8"));
        assert_eq!(row.get_by_key("c"), Some("9"));
        assert!(iter.next().is_none());
        assert_eq!(iter.width(), 3)
    }

    #[test]
    fn create_iter_with_header_default() {
        let mut iter = CSVIter::new_with_header("".as_bytes(), ',').unwrap();
        assert_eq!(iter.width(), 0);
        assert!(iter.next().is_none());
    }

    #[test]
    fn invalid_csv() {
        let data = b"test,invalid\"value\nvalid,invalid\"\"value2";
        let csv = CSVIter::new_without_header(&data[..], ',');
        
        for entry in csv {
            let Err(e) = entry else {
                panic!("Expected error");
            };
            assert_eq!(e.kind(), std::io::ErrorKind::InvalidData);
        }
    }
}
