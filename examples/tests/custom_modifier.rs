use crate::*;
use std::thread;
use std::time::Duration;

io_test!("cover1", "{capslock down}{meta down}q{meta up}{capslock up}", "{meta down}{meta up}w{meta down}{meta up}");

io_test!("cover2", "{capslock down}{a down}{capslock up}", "b");
io_test!("cover3", "{capslock down}{a down}{capslock up}{a up}", "b{a up}");

io_test!("cover4", "{a down}{capslock down}{a up}{capslock up}", "a{capslock}");

io_test!("cover5", "{b down}{capslock down}{b up}{capslock up}", "b{capslock}");
io_test!("cover6", "{b down}{capslock down}{b repeat}{b up}{capslock up}", "{b down}{b repeat}{b up}{capslock}");
