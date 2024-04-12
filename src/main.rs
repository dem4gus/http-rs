mod http;

use http::server;

const DEFAULT_ADDR_IP4: &str = "127.0.0.1";
const DEFAULT_PORT: u16 = 7878;
const DEFAULT_WORKERS: usize = 8;

fn main() -> Result<(), std::io::Error> {
    let listen_addr = format!("{DEFAULT_ADDR_IP4}:{DEFAULT_PORT}");
    server::new(listen_addr, DEFAULT_WORKERS)
}
