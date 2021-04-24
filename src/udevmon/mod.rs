use std::io::{stdout, Write};
use tokio::io::AsyncReadExt;

#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct input_event {
    pub time: timeval,
    pub type_: u16,
    pub code: u16,
    pub value: i32,
}

#[allow(non_camel_case_types)]
pub type time_t = i64;
#[allow(non_camel_case_types)]
pub type suseconds_t = i64;

#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
#[allow(non_camel_case_types)]
pub struct timeval {
    pub tv_sec: time_t,
    pub tv_usec: suseconds_t,
}

impl PartialEq for input_event {
    fn eq(&self, other: &Self) -> bool {
        self.code == other.code &&
            self.type_ == other.type_ &&
            self.value == other.value
    }
}

// impl PartialEq<Key> for input_event {
//     fn eq(&self, other: &Key) -> bool {
//         self.code as i32 == other.event_code &&
//             self.value == other.key_type
//     }
// }

impl Eq for input_event {}


fn print_event(ev: &input_event) {
    unsafe {
        stdout().write_all(any_as_u8_slice(ev)).unwrap();
        stdout().flush().unwrap();
    }
}

unsafe fn any_as_u8_slice_mut<T: Sized>(p: &mut T) -> &mut [u8] {
    ::std::slice::from_raw_parts_mut(
        (p as *const T) as *mut u8,
        ::std::mem::size_of::<T>(),
    )
}

unsafe fn any_as_u8_slice<T: Sized>(p: &T) -> &[u8] {
    ::std::slice::from_raw_parts(
        (p as *const T) as *const u8,
        ::std::mem::size_of::<T>(),
    )
}

async fn listen_to_key_events(ev: &mut input_event, input: &mut tokio::io::Stdin) {
    unsafe {
        let slice = any_as_u8_slice_mut(ev);
        match input.read_exact(slice).await {
            Ok(_) => (),
            Err(_) => {
                ::std::mem::forget(slice);
                panic!("error reading stdin");
            }
        }
    }
}

