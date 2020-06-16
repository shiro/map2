extern crate nom;

use input_linux_sys::input_event;

use nom::{
    IResult,
    bytes::complete::{tag, take_while_m_n},
    combinator::map_res,
    sequence::tuple,
};
use std::process::exit;

// struct timeval{
//     time_t         tv_sec      seconds
// suseconds_t    tv_usec     microseconds
// }
//
// struct input_event {
//     // struct timeval time;
//     _type: u16,
//     code: u16,
//     value: i32,
// }


struct Packet<'a> {
    header: u32,
    data: &'a [u8],
}

fn main() {
    // println!("Hello, world!");
    exit(1);

    loop {}
}

// fn packet_parser<'a>(input: &[u8]) -> IResult<&[u8], input_event<'a>> {
//     do_parse!(input,
//         // assuming you want to parse those as big endian numbers
//         header: be_u32 >>
//         length:  be_u32 >>
//         data:     take!(length) >>
//         (Packet {
//             header: header,
//             data: data
//         })
// }
