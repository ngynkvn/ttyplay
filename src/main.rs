extern crate nom;

use nom::IResult;
use nom::number::complete::le_u32;
use nom::bytes::complete::take;
use nom::sequence::tuple;
use nom::multi::many1;
use std::fs;
use std::env;
use std::io::Write;
use std::{thread, time};
use std::time::Duration;

#[derive(Debug)]
struct TTYHeader {
    pub tv_sec: u32,
    pub tv_usec: u32,
    pub len: u32
}

struct TTYChunk {
    pub header: TTYHeader,
    pub codes: Vec<u8>
}

struct TTYRecord {
    pub chunks: Vec<TTYChunk>
}

fn header(input: &[u8]) -> IResult<&[u8], TTYHeader> {
    let (input, (tv_sec, tv_usec, len)) = tuple((le_u32, le_u32, le_u32))(input)?;
    Ok((input, TTYHeader {tv_sec, tv_usec, len}))
}

fn chunk(input: &[u8]) -> IResult<&[u8], TTYChunk> {
    let (input, head) = header(input)?;
    let (input, codes) = take(head.len)(input)?;
    Ok((input, TTYChunk {header: head, codes: codes.to_vec()}))
}

fn record(input: &[u8]) -> IResult<&[u8], TTYRecord> {
    let (input, chunks) = many1(chunk)(input)?;
    Ok((input, TTYRecord {chunks}))
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let arg_file = &args[1];
    let input = fs::read(arg_file).unwrap();
    let (i, record) = record(&input).unwrap();

    let mut prev = (0, 0);
    for c in record.chunks {
        if prev == (0, 0) {
            std::io::stdout().write_all(&c.codes);
            prev = (c.header.tv_sec, c.header.tv_usec);
            continue;
        }

        let (sec, usec) = (c.header.tv_sec, c.header.tv_usec);
        let curr = Duration::from_secs(sec.into()) + Duration::from_nanos(usec.into());
        let p = Duration::from_secs(prev.0.into()) + Duration::from_nanos(prev.1.into());
        thread::sleep(curr - p);
        prev = (sec, usec);
        std::io::stdout().write_all(&c.codes);
        
    }
}
