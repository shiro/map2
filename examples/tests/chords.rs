use crate::*;

io_test2!("basic1", "input a", "sleep 55", "input b", "sleep 55", "output ab");
io_test2!("basic2", "input a", "input b", "sleep 55", "output ab");

io_test2!(
    "hold1",
    "input {a down}",
    "sleep 55",
    "output {a down}",
    "input {a repeat}{a up}",
    "output {a repeat}{a up}"
);

io_test2!(
    "break_chord1",
    "input {a down}",
    "sleep 10",
    "input {z down}",
    "sleep 10",
    "output {a down}{z down}",
    "input {a up}{z up}",
    "sleep 10",
    "output {a up}{z up}"
);

io_test2!("simple_chord1", "input {a down}{b down}{a up}{b up}", "sleep 55", "output c");

io_test2!("multi_chord1", "input {a down}{b down}{b up}{b down}{a up}{b up}", "sleep 55", "output cc");
io_test2!(
    "multi_chord2",
    "input {a down}",
    "sleep 55",
    "output {a down}",
    "input {b down}{a up}a",
    "sleep 10",
    "output {b down}{a up}a"
);

io_test2!(
    "chord_to_function",
    "global counter 0",
    "input {c down}{d down}{c up}{d up}",
    "sleep 55",
    "global counter 1",
    // TODO assert empty
);

// TODO do we want to hold c here?
io_test2!("foo1", "input {a down}{b down}", "output c");
// io_test2!("foo2", "input {a down}{d down}", "output {a down}{d down}");
