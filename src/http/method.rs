#[derive(PartialEq, Debug)]
pub enum Method {
    GET,
}

impl std::str::FromStr for Method {
    type Err = ParseMethodError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "GET" => Ok(Method::GET),
            _ => Err(ParseMethodError {
                method: s.to_string(),
            }),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct ParseMethodError {
    method: String,
}

impl std::fmt::Display for ParseMethodError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "could not parse http method from {}", self.method)
    }
}

impl std::error::Error for ParseMethodError {}

#[cfg(test)]
mod test {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn parses_get_request() {
        let method_raw = "GET";
        let want = Method::GET;
        let got = Method::from_str(method_raw).unwrap();
        assert_eq!(got, want);
    }

    #[test]
    fn parse_fails_on_lowercase_verb() {
        let method_raw = "get";
        let want = ParseMethodError {
            method: method_raw.to_string(),
        };
        let got = Method::from_str(method_raw).unwrap_err();
        assert_eq!(got, want);
    }
}
