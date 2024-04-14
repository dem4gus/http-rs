use super::thread_pool::ThreadPool;
use std::error::Error;
use std::fs;
use std::io::{prelude::*, BufReader};
use std::net::{TcpListener, TcpStream, ToSocketAddrs};
use std::str::FromStr;

// TODO: make a struct to hold all this

pub fn new<A>(listen_addr: A, workers: usize) -> Result<(), std::io::Error>
where
    A: ToSocketAddrs,
{
    let listener = TcpListener::bind(listen_addr)?;

    let pool = ThreadPool::new(workers);

    for conn in listener.incoming() {
        match conn {
            Ok(stream) => {
                pool.execute(|| {
                    // TODO: different handler funcs for paths
                    match handle_connection(stream) {
                        Ok(()) => {
                            println!("connection closed successfully");
                        }
                        Err(e) => {
                            eprintln!("error handling connection: {:?}", e);
                        }
                    };
                });
            }
            Err(e) => {
                eprintln!("error creating connection: {:?}", e);
            }
        }
    }
    // TODO: graceful shutdown on SIGINT
    println!("Shutting down.");

    Ok(())
}

fn handle_connection(mut stream: TcpStream) -> Result<(), Box<dyn Error>> {
    let buf_reader = BufReader::new(&mut stream);
    let request = Request::parse(buf_reader)?;
    // TODO: create a response object
    let (status_line, filename) = match request.target.as_str() {
        "/" => ("HTTP/1.1 200 OK", "hello.html"),
        _ => ("HTTP/1.1 404 Not Found", "404.html"),
    };

    let contents = fs::read_to_string(format!("www/{filename}"))?;
    let length = contents.len();

    // TODO: build response object
    let response = format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}");

    stream.write_all(response.as_bytes())?;
    // TODO: requests are more than a single line

    Ok(())
}

const HTTP1_1: &'static str = "HTTP/1.1";

#[derive(PartialEq, Debug)]
struct Request {
    method: Method,
    target: String,
    protocol: &'static str,
    host: String,
}

impl Request {
    fn parse<T>(req: T) -> Result<Self, Box<dyn Error>>
    where
        T: BufRead,
    {
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

#[derive(PartialEq, Debug)]
enum Method {
    GET,
}

impl FromStr for Method {
    type Err = ParseRequestError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "GET" => Ok(Method::GET),
            _ => Err(ParseRequestError),
        }
    }
}

#[derive(Debug, PartialEq)]
struct ParseRequestError;

impl std::fmt::Display for ParseRequestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "could not parse request")
    }
}

impl Error for ParseRequestError {}

#[cfg(test)]
mod test_http_request {
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
