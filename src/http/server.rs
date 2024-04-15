use super::{request::Request, thread_pool::ThreadPool};
use std::error::Error;
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
    let response = format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}\r\n\r\n");

    stream.write_all(response.as_bytes())?;

    Ok(())
}
