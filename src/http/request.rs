use std::{collections::HashMap, str::FromStr as _};

use super::{constants::HTTP1_1, method::Method};

#[derive(PartialEq, Debug)]
pub struct Request {
    pub method: Method,
    pub target: String,
    protocol: &'static str,
    headers: HashMap<String, String>,
}

impl Request {
    pub fn parse<T>(req: T) -> Result<Self, Box<dyn std::error::Error>>
    where
        T: std::io::BufRead,
    {
        let mut lines = req.lines();

        let request_line = if let Some(line) = lines.next() {
            line?
        } else {
            return Err(ParseRequestError.into());
        };
        let [method_raw, target] = match request_line.split(' ').collect::<Vec<&str>>()[..] {
            [method, target, HTTP1_1] => [method, target],
            _ => return Err(ParseRequestError.into()),
        };
        let method = Method::from_str(method_raw)?;

        let mut headers = HashMap::new();
        let host_line = if let Some(line) = lines.next() {
            line?
        } else {
            return Err(ParseRequestError.into());
        };
        if let Some(("Host", domain)) = host_line.split_once(':') {
            headers.insert("Host".into(), domain.trim().into());
        } else {
            return Err(ParseRequestError.into());
        };

        for line in lines {
            match line {
                Ok(l) => {
                    if l == "" {
                        break;
                    }
                    if let Some((k, v)) = l.split_once(':') {
                        headers.insert(k.trim().into(), v.trim().into());
                    }
                }
                Err(e) => return Err(e.into()),
            }
        }

        Ok(Self {
            method,
            headers,
            target: target.into(),
            protocol: HTTP1_1,
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
    use super::*;
    use std::io::Cursor;

    #[test]
    fn parse_succeeds() {
        let request = Cursor::new("GET / HTTP/1.1\r\nHost: example.com\r\n\r\n".as_bytes());
        let mut headers = HashMap::new();
        headers.insert("Host".into(), "example.com".into());
        let want = Request {
            method: Method::GET,
            target: String::from("/"),
            protocol: "HTTP/1.1",
            headers,
        };

        let got = Request::parse(request).unwrap();
        assert_eq!(got, want);
    }

    #[test]
    fn parse_succeeds_with_port_in_host() {
        let request = Cursor::new("GET / HTTP/1.1\r\nHost: localhost:7878\r\n\r\n".as_bytes());
        let mut headers = HashMap::new();
        headers.insert("Host".into(), "localhost:7878".into());
        let want = Request {
            method: Method::GET,
            target: String::from("/"),
            protocol: "HTTP/1.1",
            headers,
        };
        let got = Request::parse(request).unwrap();
        assert_eq!(got, want)
    }

    #[test]
    fn parse_additional_headers() {
        let req_raw = vec![
            "GET / HTTP/1.1",
            "Host: example.com",
            "User-Agent: rust unit test",
            "foo: bar",
            "\r\n",
        ]
        .join("\r\n");
        let request = Cursor::new(req_raw.as_bytes());

        let mut headers = HashMap::new();
        headers.insert("Host".into(), "example.com".into());
        headers.insert("User-Agent".into(), "rust unit test".into());
        headers.insert("foo".into(), "bar".into());

        let want = Request {
            method: Method::GET,
            protocol: HTTP1_1,
            target: "/".into(),
            headers,
        };
        let got = Request::parse(request).unwrap();
        assert_eq!(got, want);
    }

    #[test]
    fn parse_fails_without_host() {
        let request = Cursor::new("GET / HTTP/1.1\r\n\r\n");
        let got = Request::parse(request).unwrap_err();
        assert!(got.is::<ParseRequestError>());
    }
}
