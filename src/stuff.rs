use pipey::Pipey;
use std::collections::HashMap;
use std::io::{self, Read};
use std::net;
use std::str;

const HTTP_HEADER_END_DELIMITER: u32 = u32::from_be_bytes(*b"\r\n\r\n");
pub const INITIAL_BUFFER_SIZE: usize = 1 << 12;

pub struct HttpReq<'a> {
    pub method: &'a str,
    pub raw_url: &'a str,
    pub http_ver: &'a str,
    // keep ref valid
    pub header_map: HashMap<&'a str, &'a str>,
}

pub struct Closet<'a> {
    pub request: HttpReq<'a>,
    stream: net::TcpStream,
    brick: &'a [u8],
    // end of header in brick
    header_end: usize,
}

impl Closet<'_> {
    /// you need to give this function a pre-alloc buffer. and the length to read. even though it should be the same as buf.len(). just give it
    ///
    /// recommend reading buffer length from header_map in closet.HttpReq
    pub fn request_body(&mut self, buf: &mut [u8], len: usize) -> Result<(), io::Error> {
        // some data is usually leaked into brick. thats what header_end is for
        let rl = self.brick.len() - self.header_end;
        if len > rl {
            // give whole birck + read
            buf.copy_from_slice(&self.brick[self.header_end..]);
            self.read(&mut buf[self.brick.len()..])?;
        } else {
            // dont give whole brick
            buf.copy_from_slice(&self.brick[self.header_end..self.header_end + rl])
        }
        Ok(())
    }
}

impl io::Read for Closet<'_> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.stream.read(buf)
    }
}

impl io::Write for Closet<'_> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.stream.write(buf)
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

// faster tha nchecking byte-by-byte
fn recursive_find_u32_delim(brick: &[u8], delim: u32) -> Option<usize> {
    if brick.len() < 4 {
        return None;
    }
    if u32::from_be_bytes(TryInto::<[u8; 4]>::try_into(&brick[brick.len() - 4..]).unwrap()) == delim
    {
        return Some(brick.len());
    }
    recursive_find_u32_delim(&brick[..brick.len() - 1], delim)
}

fn fl(x: &str) -> Option<(&str, &str, &str)> {
    x.split_ascii_whitespace().collect::<Vec<&str>>().pipe(|v| {
        if v.len() < 3 {
            None
        } else {
            Some((v[0], v[1], v[2]))
        }
    })
}

#[inline(always)]
pub fn pack_httpreq<'a>(
    v: ((&'a str, &'a str, &'a str), HashMap<&'a str, &'a str>),
) -> HttpReq<'a> {
    HttpReq {
        method: v.0 .0,
        raw_url: v.0 .1,
        http_ver: v.0 .2,
        header_map: v.1,
    }
}

fn map_header<'a>(brick: &'a [u8]) -> Option<((&str, &str, &str), HashMap<&str, &str>)> {
    // get as str anyways
    let l: Vec<&str> = str::from_utf8(brick).ok()?.split('\n').collect();
    if l.is_empty() {
        return None;
    }
    (
        fl(l[0])?,
        l.into_iter()
            .skip(1)
            .filter_map(|x| x.split_once(':')?.pipe(|(k, v)| Some((k, v))))
            .collect(),
    )
        .pipe(|v| Some(v))
}

// f any err
pub fn hold_conn(
    mut x: net::TcpStream,
    brick: &mut [u8],
    f: &mut impl for<'a> FnMut(Closet<'a>),
) -> Option<()> {
    // initial read, copy body to vec
    x.read(brick).ok()?;
    // ignore if header is larger than 4KB1
    let header_end = recursive_find_u32_delim(&brick, HTTP_HEADER_END_DELIMITER)?;
    f(Closet {
        request: pack_httpreq(map_header(&brick[..header_end])?),
        stream: x,
        brick,
        header_end,
    });
    Some(())
}

// no str format
// replace later
pub fn dumbass_format_n_write_header(
    dst: &mut impl io::Write,
    first_line: &str,
    jkv: &[(&str, &str)],
) -> Result<(), io::Error> {
    write!(dst, "{first_line}\r\n")?;
    // need outer-layer `?`
    for (k, v) in jkv {
        write!(dst, "{k}: {v}\r\n")?;
    }
    // write final
    dst.write(b"\r\n")?;
    Ok(())
}

fn from_uppercase_hex_u8(c: u8) -> u8 {
    if 0x39 < c {
        (c - 7) & 0xf
    } else {
        c & 0xf
    }
}

/// returns Err() if data is malformed.
///
/// you are to provide your own vec. also check the return
pub fn percent_decode_bytes_vec<'a>(r: &[u8], buf: &mut Vec<u8>) -> Result<(), &'static str> {
    // issue: still returns Ok(()) even if r is initally empty.
    if r.is_empty() {
        return Ok(());
    }
    percent_decode_bytes_vec(
        match r[0] {
            b'%' => {
                if 2 < r.len() {
                    buf.push((from_uppercase_hex_u8(r[1]) * 16) + from_uppercase_hex_u8(r[2]));
                    &r[3..]
                } else {
                    return Err("malformed data");
                }
            }
            _ => match r[0] {
                b'+' => buf.push(b' '),
                _ => buf.push(r[0]),
            }
            .pipe(|_| &r[1..]),
        },
        buf,
    )
}
