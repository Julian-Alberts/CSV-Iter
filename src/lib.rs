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
    seperator: char,
}

impl<R, H> CSVIter<R, H>
where
    R: Read,
{
    fn new_internal(data: R, header: H, seperator: char) -> Self {
        CSVIter {
            header: Rc::new(header),
            data,
            seperator,
        }
    }
}

impl<R> CSVIter<R, NoHeader>
where
    R: Read,
{
    pub fn new_without_header(data: R, field_seperator: char) -> Self {
        CSVIter::new_internal(data, NoHeader, field_seperator)
    }
}

impl<R> CSVIter<R, WithHeader>
where
    R: BufRead,
{
    pub fn new_with_header(mut data: R, field_seperator: char) -> std::io::Result<Self> {
        let header = if let Some(header) = Row::new(&mut data, Rc::new(NoHeader), field_seperator)?
        {
            WithHeader::new(header)
        } else {
            WithHeader::default()
        };
        Ok(CSVIter::new_internal(data, header, field_seperator))
    }
}

impl<R, H> Iterator for CSVIter<R, H>
where
    R: BufRead,
{
    type Item = std::io::Result<Row<H>>;

    fn next(&mut self) -> Option<Self::Item> {
        Row::new(&mut self.data, self.header.clone(), self.seperator).transpose()
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
        assert_eq!(row.get("a"), Some("1"));
        assert_eq!(row.get("b"), Some("2"));
        assert_eq!(row.get("c"), Some("3"));
        let row = iter.next().unwrap().unwrap();
        assert_eq!(row.get("a"), Some("4"));
        assert_eq!(row.get("b"), Some("5"));
        assert_eq!(row.get("c"), Some("6"));
        let row = iter.next().unwrap().unwrap();
        assert_eq!(row.get("a"), Some("7"));
        assert_eq!(row.get("b"), Some("8"));
        assert_eq!(row.get("c"), Some("9"));
        assert!(iter.next().is_none());
    }
}
