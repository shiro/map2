use std::thread;
use std::time::Duration;

use crate::*;

io_test!("simple1", "a", "b");
io_test!("simple2", "b", "b");

//

io_test!("cover1", "i{meta down}q{meta up} is working", "i{meta}{t down}{meta}{t up}{meta} is working");
// io_test!("cover1", "i{meta down}q{meta up} is working", "i{meta}{t{meta} is working");

// io_test!("cover1", "i{#q} is working", "i{leftmeta}t{leftmeta} is working");
// io_test!("cover2", "i{#q down}{#q up} is working", "it is working");
