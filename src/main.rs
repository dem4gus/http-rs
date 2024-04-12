use http_rs::ThreadPool;
use std::{
    fs,
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream},
    thread,
    time::Duration,
};

const DEFAULT_ADDR_IP4: &str = "127.0.0.1";
const DEFAULT_PORT: u16 = 7878;
const DEFAULT_WORKERS: usize = 8;

fn main() -> Result<(), std::io::Error> {
    // TODO: pull out all of this to a module
    // ideally it will be called like
    //
    // http::register_handler(uri: string, hander: func)
    // http::listen_and_serve(socket: impl ToSocketAddr)

    let socket_ipv4 = format!("{DEFAULT_ADDR_IP4}:{DEFAULT_PORT}");
    let listener = TcpListener::bind(socket_ipv4)?;

    let pool = ThreadPool::new(DEFAULT_WORKERS);

    for conn in listener.incoming() {
        match conn {
            Ok(stream) => {
                pool.execute(|| {
                    // FIXME: handle error
                    handle_connection(stream).unwrap();
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
            "GET /sleep HTTP/1.1" => {
                thread::sleep(Duration::from_secs(10));
                ("HTTP/1.1 200 OK", "hello.html")
            }
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
