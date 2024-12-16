use crate::*;

io_test2!("basic1", "input a", "sleep 55", "input b", "sleep 55", "output ab");

io_test2!("hold1", "input {a down}", "sleep 55", "input {a repeat}{a up}", "output {a down}{a repeat}{a up}");

io_test2!(
    "break_chord1",
    "input {a down}",
    "sleep 10",
    "input {z down}",
    "sleep 10",
    "output a{z down}",
    "input {a up}{z up}",
    "sleep 10",
    "output {z up}"
);

io_test2!("simple_chord1", "input {a down}{b down}{a up}{b up}", "sleep 55", "output c");

io_test2!("multi_chord1", "input {a down}{b down}{b up}{b down}{a up}{b up}", "sleep 55", "output cc");

io_test2!(
    "chord_to_function",
    "global counter 0",
    "input {c down}{d down}{c up}{d up}",
    "sleep 55",
    "global counter 1"
);
