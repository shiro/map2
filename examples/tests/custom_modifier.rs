use crate::*;
use std::thread;
use std::time::Duration;

io_test2!("basic1", "input {capslock down}{a down}{capslock up}", "output {b down}", "input {a up}a", "output {b up}a");
io_test!("basic2", "a{capslock down}a{capslock up}c", "abc");
io_test!("basic3", "{capslock down}a{capslock up}{capslock down}a{capslock up}{capslock down}a{capslock up}", "bbb");

io_test!("out_of_order_pre1", "{a down}{capslock down}{a up}{capslock up}", "a{capslock}");
io_test!("out_of_order_pre2", "{b down}{capslock down}{b up}{capslock up}", "b{capslock}");
io_test!(
    "out_of_order_pre3",
    "{b down}{capslock down}{b repeat}{b up}{capslock up}",
    "{b down}{b repeat}{b up}{capslock}"
);

io_test!("out_of_order_post1", "{capslock down}{a down}{capslock up}{a up}", "b");
io_test!(
    "out_of_order_post2",
    "{capslock down}{a down}{capslock up}{a up}{capslock down}{a down}{capslock up}{a up}",
    "bb"
);

io_test!("with_modifier1", "{capslock down}{meta down}q{meta up}{capslock up}", "{meta}w{meta}");
