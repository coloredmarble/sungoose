# a dead simple http server lib/framework  

its pretty simple. ***handing you the stream directly and gives you a few helper functions***.  
you can add your own middleware function (Result handling, Custom response system)  
highly inefficient. tho id argue its better than the one i wrote in go

### Usage:
```
use std::{
    io::{self, Write},
    net,
};

use sungoose::{self, stuff::dumbass_format_n_write_header};

const OK_RESPONSE: &[u8] = b"haiiii!! uwu";

fn operator(mut c: sungoose::stuff::Closet) -> Result<(), String> {
    if c.request.method != "GET" {
        // gaslight the client
        // closet, first line, array of (&str,&str) (header fields)
        c.write_all(b"HTTP/1.1 400 bad req\r\n\r\n")
            .map_err(|_| "failed to gaslight client")?;
    }
    dumbass_format_n_write_header(
        &mut c,
        "HTTP/1.1 200 ok",
        &[("Content-Length", OK_RESPONSE.len().to_string().as_str())],
    )
    .map_err(|_| "failed to write OK header!!")?;
    _ = c.write_all(OK_RESPONSE);
    Ok(())
}

fn main() {
    println!("starting sever1!1!!!1");
    sungoose::init_tcp_server_thread_per_req(
        net::TcpListener::bind("0.0.0.0:8080").unwrap(),
        // middleware function (add "?"" symbol return)
        |c| match operator(c) {
            Ok(_) => {}
            Err(e) => {
                eprintln!("error! {e}")
            }
        },
    );
}
```

a okay amount of work is being done to kinda optimize and make it easier to use