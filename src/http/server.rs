use super::thread_pool::ThreadPool;
use std::fs;
use std::io::{prelude::*, BufReader};
use std::net::{TcpListener, TcpStream, ToSocketAddrs};

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
    println!("Shutting down.");

    Ok(())
}

fn handle_connection(mut stream: TcpStream) -> Result<(), std::io::Error> {
    let buf_reader = BufReader::new(&mut stream);
    if let Some(request_line) = buf_reader.lines().next() {
        // TODO: requests are more than a single line
        // TODO: parse into a request object
        let (status_line, filename) = match &request_line?[..] {
            "GET / HTTP/1.1" => ("HTTP/1.1 200 OK", "hello.html"),
            _ => ("HTTP/1.1 404 NOT FOUND", "404.html"),
        };

        let contents = fs::read_to_string(format!("www/{filename}"))?;
        let length = contents.len();

        // TODO: build response object
        let response = format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}");

        stream.write_all(response.as_bytes())?;
    };

    Ok(())
}

fn parse_request(request_line: &str) -> Option<Request> {
    let [method_raw, target, protocol] = match request_line.split(' ').collect::<Vec<&str>>()[..] {
        [method, target, protocol] => [method, target, protocol],
        // TODO: return a parse error instead of None
        _ => return None,
    };

    let method = match method_raw {
        "GET" => HttpMethod::GET,
        // TODO: return a parse error instead of None
        _ => return None,
    };

    Some(Request {
        method,
        target: String::from(target),
        protocol: String::from(protocol),
    })
}

#[derive(PartialEq, Debug)]
struct Request {
    method: HttpMethod,
    target: String,
    protocol: String,
}

#[derive(PartialEq, Debug)]
enum HttpMethod {
    GET,
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn parses_get_request() {
        let request = "GET foo bar";
        let want = HttpMethod::GET;

        if let Some(parsed_request) = parse_request(request) {
            assert_eq!(want, parsed_request.method);
        } else {
            panic!("did not parse correctly");
        };
    }
}
