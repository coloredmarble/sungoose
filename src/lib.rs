use pipey::Pipey;
// "trait not found" and you can still link me to the fucking trait. rust have cum so far
use std::net;
use std::thread;

pub mod stuff;
use stuff::{hold_conn, Closet, INITIAL_BUFFER_SIZE};
/// blocking server
pub fn init_tcp_server_hot_single(listener: net::TcpListener, mut f: impl FnMut(Closet<'_>)) {
    let mut hold_brick: [u8; INITIAL_BUFFER_SIZE] = [0; INITIAL_BUFFER_SIZE];
    // cant use for_each (fnmut)
    for st in listener.incoming() {
        hold_conn(
            match st {
                Ok(st) => st,
                Err(_) => continue,
            },
            &mut hold_brick,
            &mut f,
        );
    }
}

/// uses threads. alot of em
pub fn init_tcp_server_thread_per_req<F: FnMut(Closet<'_>) + Send + Sync + Copy + 'static>(
    listener: net::TcpListener,
    mut f: F,
) {
    // cant use for_each (fnmut)
    for st in listener.incoming() {
        match st {
            Err(_) => {}
            // make new buffer for every request (dont use this shit)
            Ok(st) => {
                _ = thread::spawn(|| hold_conn(st, &mut [0; INITIAL_BUFFER_SIZE], &mut f))
            }
        }
    }
}