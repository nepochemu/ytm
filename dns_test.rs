use std::net::ToSocketAddrs;
fn main() {
    println!("{:?}", "www.google.com:80".to_socket_addrs());
}