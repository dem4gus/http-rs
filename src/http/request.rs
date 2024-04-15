use std::str::FromStr as _;

use super::{constants::HTTP1_1, method::Method};

#[derive(PartialEq, Debug)]
pub struct Request {
    pub method: Method,
    pub target: String,
    protocol: &'static str,
    host: String,
}

impl Request {
    pub fn parse<T>(req: T) -> Result<Self, Box<dyn std::error::Error>>
    where
        T: std::io::BufRead,
    {
        // BUG: response is served before request has ended (double CRLF)
        let mut lines = req.lines();

        let request_line = if let Some(line) = lines.next() {
            line?
        } else {
            return Err(ParseRequestError.into());
        };
        let [method_raw, target, protocol] = match request_line.split(' ').collect::<Vec<&str>>()[..]
        {
            [method, target, protocol] => [method, target, protocol],
            _ => return Err(ParseRequestError.into()),
        };
        let method = Method::from_str(method_raw)?;
        if protocol != HTTP1_1 {
            return Err(ParseRequestError.into());
        }

        let host_line = if let Some(line) = lines.next() {
            line?
        } else {
            return Err(ParseRequestError.into());
        };
        let host = match host_line.split_once(':') {
            Some(("Host", domain)) => domain.trim(),
            _ => return Err(ParseRequestError.into()),
        };

        Ok(Self {
            method,
            target: String::from(target),
            protocol: HTTP1_1,
            host: String::from(host),
        })
    }
}

#[derive(Debug, PartialEq)]
struct ParseRequestError;

impl std::fmt::Display for ParseRequestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "could not parse request")
    }
}

impl std::error::Error for ParseRequestError {}

#[cfg(test)]
mod test {
    use std::io::Cursor;

    use super::*;
    #[test]
    fn parse_succeeds() {
        let request = Cursor::new("GET / HTTP/1.1\r\nHost: example.com\r\n\r\n".as_bytes());
        let want = Request {
            method: Method::GET,
            target: String::from("/"),
            protocol: "HTTP/1.1",
            host: String::from("example.com"),
        };

        let got = Request::parse(request).unwrap();
        assert_eq!(got, want);
    }

    #[test]
    fn parse_succeeds_with_port_in_host() {
        let request = Cursor::new("GET / HTTP/1.1\r\nHost: localhost:7878\r\n\r\n".as_bytes());
        let want = Request {
            method: Method::GET,
            target: String::from("/"),
            protocol: "HTTP/1.1",
            host: String::from("localhost:7878"),
        };
        let got = Request::parse(request).unwrap();
        assert_eq!(got, want)
    }

    #[test]
    fn parse_fails_without_host() {
        let request = Cursor::new("GET / HTTP/1.1\r\n\r\n");
        let got = Request::parse(request).unwrap_err();
        assert!(got.is::<ParseRequestError>());
    }
}
