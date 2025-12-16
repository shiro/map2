use crate::*;

io_test2!("break1", "input {meta down}{q down}{meta up}", "output {meta}t");
io_test2!("break2", "input {meta down}{q down}{meta up}{q up}", "output {meta}t");
io_test2!("break3", "input {meta down}{q down}{w down}", "output {meta}t{meta down}{w down}");
io_test2!("break4", "input {meta down}{q down}{shift}{q up}", "output {meta}t{shift down}{meta down}{shift up}");

// io_test2!(
//     "break_set2_1",
//     "input {meta down}{ctrl down}{1 down}{meta up}",
//     "output {meta down}{ctrl down}{ctrl up}{meta up}{shift down}{alt down}2{shift up}{alt up}{ctrl down}",
// );

// io_test2!("break_fn1", "input {meta down}{2 down}", "output {meta}a");
