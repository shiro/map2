use std::thread;
use std::time::Duration;

use crate::*;

// io_test!("simple1", "a", "b");
// io_test!("simple2", "b", "b");

//

io_test!("cover1", "i{meta down}q{meta up} is working", "i{meta}t{meta} is working");
// io_test!("cover1", "{meta down}q{meta up}", "{meta}t{meta}");
// io_test!("cover1", "i{meta down}q{meta up} is working", "i{meta}{t{meta} is working");

// io_test!("cover1", "i{#q} is working", "i{leftmeta}t{leftmeta} is working");
// io_test!("cover2", "i{#q down}{#q up} is working", "it is working");

io_test!("partial1", "{meta down}q", "{meta}t{meta down}");

io_test!("click1", "{meta down}1{meta up}", "{meta}{ctrl down}a{ctrl up}{meta}");
io_test!("click2", "{meta down}2{meta up}", "{meta}{ctrl down}b{ctrl up}{meta}");
io_test!("click3", "{meta down}3{meta up}", "{meta}{ctrl down}{c down}{ctrl up}{meta}{ctrl down}{c up}{ctrl up}{meta}");
io_test!("click4", "{meta down}4{meta up}", "{meta}{ctrl down}{d down}{ctrl up}{meta}{alt down}{d up}{alt up}{meta}");
