use std::{io::BufRead, rc::Rc};

use crate::{NoHeader, WithHeader};

pub struct Row<H = NoHeader> {
    pub(crate) data: Vec<String>,
    header: Rc<H>,
}

impl<H> Row<H> {
    pub(super) fn new(
        data: &mut impl BufRead,
        header: Rc<H>,
        field_seperator: char,
    ) -> std::io::Result<Option<Self>> {
        let Some(data) = parse_row(data, field_seperator)? else {
            return Ok(None);
        };
        Ok(Some(Self { data, header }))
    }
}

impl Row<NoHeader> {
    pub fn get(&self, index: usize) -> Option<&str> {
        self.data.get(index).as_ref().map(|s| s.as_str())
    }
}

impl Row<WithHeader> {
    pub fn get(&self, key: &str) -> Option<&str> {
        let index = self.header.get_index(key)?;
        self.data.get(index).as_ref().map(|s| s.as_str())
    }
}

fn parse_row(
    data: &mut impl BufRead,
    field_seperator: char,
) -> std::io::Result<Option<Vec<String>>> {
    let mut values = Vec::new();
    let mut buf = String::new();
    if data.read_line(&mut buf)? == 0 {
        return Ok(None);
    };
    let mut chars = buf.chars();

    let mut value_is_masked = false;
    let mut is_masked_active = false;
    let mut is_first_char = true;
    let mut value_buf = String::with_capacity(512);
    let mut last_was_seperator = false;

    while let Some(c) = chars.next() {
        match (c, value_is_masked, is_first_char, is_masked_active) {
            // If '"' is the first char mark the value as masked
            ('"', false, true, false) => {
                value_is_masked = true;
                is_masked_active = true;
            }
            // If '"' is not the first char and the value is not masked
            ('"', false, false, false) => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Invalid CSV data: Unexpected '\"'",
                ));
            }
            // If the current value is masked: flip is_masked_active on every '"'
            ('"', true, false, _) => {
                is_masked_active = !is_masked_active;
                if is_masked_active {
                    value_buf.push('"');
                }
            }
            // If we find a unmasked newline this row is done
            ('\n', false, _, _) => {
                break;
            }
            // If we find a masked newline we need to load the next line
            ('\n', true, _, _) => {
                value_buf.push('\n');
                buf.clear();
                data.read_line(&mut buf)?;
                chars = buf.chars();
            }
            (c, _, _, false) if c == field_seperator => {
                let mut value = value_buf.clone();
                value.shrink_to_fit();
                values.push(value);

                value_buf.clear();
                is_first_char = true;
                value_is_masked = false;
                is_masked_active = false;
                last_was_seperator = true;
                continue;
            }
            (c, _, _, _) => {
                value_buf.push(c);
            }
        }
        is_first_char = false;
        last_was_seperator = false;
    }

    if !value_buf.is_empty() || last_was_seperator {
        values.push(value_buf);
    }

    Ok(Some(values))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_value_without_header() {
        let row = Row {
            data: vec!["1".to_string(), "2".to_string(), "3".to_string()],
            header: Rc::new(NoHeader),
        };
        assert_eq!(row.get(0), Some("1"));
        assert_eq!(row.get(1), Some("2"));
        assert_eq!(row.get(2), Some("3"));
        assert_eq!(row.get(3), None);
    }

    #[test]
    fn parse_single_row_simple() {
        let data = "field1,field2,field3".to_string();
        let mut data = data.as_bytes();
        let row = parse_row(&mut data, ',');
        assert!(row.is_ok());
        assert_eq!(
            row.unwrap(),
            Some(vec![
                "field1".to_string(),
                "field2".to_string(),
                "field3".to_string()
            ])
        );
    }

    #[test]
    fn parse_single_row_simple_end_with_newline() {
        let data = "field1,field2,field3\n".to_string();
        let mut data = data.as_bytes();
        let row = parse_row(&mut data, ',');
        assert!(row.is_ok());
        assert_eq!(
            row.unwrap(),
            Some(vec![
                "field1".to_string(),
                "field2".to_string(),
                "field3".to_string()
            ])
        );
    }

    #[test]
    fn parse_single_row_masked() {
        let data = r#"field1,"joined,field","quotes""in field""#.to_string();
        let mut data = data.as_bytes();
        let row = parse_row(&mut data, ',');
        assert!(row.is_ok());
        assert_eq!(
            row.unwrap(),
            Some(vec![
                "field1".to_string(),
                "joined,field".to_string(),
                r#"quotes"in field"#.to_string()
            ])
        );
    }

    #[test]
    fn parse_single_row_empty_value() {
        let data = "field1,,field3".to_string();
        let mut data = data.as_bytes();
        let row = parse_row(&mut data, ',');
        assert!(row.is_ok());
        assert_eq!(
            row.unwrap(),
            Some(vec![
                "field1".to_string(),
                "".to_string(),
                "field3".to_string()
            ])
        );
    }

    #[test]
    fn parse_single_row_empty_value_at_end() {
        let data = "field1,field2,".to_string();
        let mut data = data.as_bytes();
        let row = parse_row(&mut data, ',');
        assert!(row.is_ok());
        assert_eq!(
            row.unwrap(),
            Some(vec![
                "field1".to_string(),
                "field2".to_string(),
                "".to_string()
            ])
        );
    }

    #[test]
    fn parse_multiline_row() {
        let data = "field1,\"fie\nld2\",\"r1\nr2\"".to_string();
        let mut data = data.as_bytes();
        let row = parse_row(&mut data, ',');
        assert!(row.is_ok());
        assert_eq!(
            row.unwrap(),
            Some(vec![
                "field1".to_string(),
                "fie\nld2".to_string(),
                "r1\nr2".to_string()
            ])
        );
    }
}
