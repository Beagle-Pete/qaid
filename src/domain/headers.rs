use crate::{APIError, domain::BadHeaderInfo, utils};

#[derive(Debug, PartialEq, Clone)]
pub struct Headers(Vec<String>);

impl Headers {
    pub fn parse(headers: Vec<String>) -> Result<Self, APIError> {

        // Look for empty headers
        let empty_headers = headers.iter().enumerate()
            .filter_map(|(index, header)| {
                let header = header.trim().to_owned();
                if header.is_empty() {
                    Some(index)
                } else {
                    None
                }
            })
            .collect::<Vec<usize>>();

        // Look for duplicate headers
        let dup_headers = {
            let headers_lower: Vec<String> = headers.iter()
                .map(|header| header.to_lowercase())
                .collect();
            
            utils::get_duplicates(&headers_lower)
        };
        
        if !empty_headers.is_empty() || !dup_headers.is_empty() {
            let bad_header_info = BadHeaderInfo{
                empty: empty_headers,
                duplicate: dup_headers,
            };
            return Err(APIError::BadHeaders(bad_header_info));
        }
        
        Ok(Self(headers))
    }
}

impl AsRef<Vec<String>> for Headers {
    fn as_ref(&self) -> &Vec<String> {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn good_header_should_return_ok() {
        let headers_vec = vec!["one".to_owned(), "two".to_owned(), "three".to_owned(), "four".to_owned()];
        let headers = Headers::parse(headers_vec);
        assert!(headers.is_ok());
    }

    #[test]
    fn headers_with_empty_or_duplicate_headers_should_return_an_err() {
        let headers_vec = vec!["one".to_owned(), "one".to_owned(), "three".to_owned(), "four".to_owned()];
        let headers = Headers::parse(headers_vec);
        let expected_err = BadHeaderInfo {
            empty: vec![],
            duplicate: vec!["one".to_owned()]
        };
        assert_eq!(headers, Err(APIError::BadHeaders(expected_err)));

        let headers_vec = vec!["one".to_owned(), "".to_owned(), "three".to_owned(), "four".to_owned()];
        let headers = Headers::parse(headers_vec);
        let expected_err = BadHeaderInfo {
            empty: vec![1],
            duplicate: vec![]
        };
        assert_eq!(headers, Err(APIError::BadHeaders(expected_err)));

        let headers_vec = vec!["one".to_owned(), "".to_owned(), "   ".to_owned(), "four".to_owned(), "four".to_owned()];
        let headers = Headers::parse(headers_vec);
        let expected_err = BadHeaderInfo {
            empty: vec![1, 2],
            duplicate: vec!["four".to_owned()]
        };
        assert_eq!(headers, Err(APIError::BadHeaders(expected_err)));
    }
}