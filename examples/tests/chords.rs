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

// chained
io_test2!(
    "multi_chord3",
    "input {a down}{b down}",
    "sleep 55",
    "input {b up}{d down}",
    "sleep 55",
    "input {d up}{a up}",
    "output ce"
);

// chained + overlapping
io_test2!(
    "multi_chord4",
    "input {a down}{b down}",
    "sleep 55",
    "input {d down}{b up}",
    "sleep 55",
    "input {d up}{a up}",
    "output ce"
);

io_test2!(
    "chord_to_function",
    "global counter 0",
    "input {c down}{d down}{c up}{d up}",
    "sleep 55",
    "global counter 1",
    "output "
);

// TODO do we want to hold c here?
io_test2!("foo1", "input {a down}{b down}", "output c");
// io_test2!("foo2", "input {a down}{d down}", "output {a down}{d down}");

// tests if unrelated keys still work normally
io_test2!("unrelated_1", "input {z down}", "output {z down}", "input {z up}", "output {z up}");
io_test2!(
    "unrelated_2",
    "input {z down}{x down}",
    "output {z down}{x down}",
    "input {z up}{x up}",
    "output {z up}{x up}"
);
// press 2 keys together that are in different chords
io_test2!("unrelated_3", "input {a down}{i down}", "output ai", "input {a up}{i up}", "output ");
