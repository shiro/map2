mod script_testing;
use script_testing::*;

#[allow(unused)]
#[path = "../event_handlers.rs"]
mod event_handlers;

#[path = "../../examples/tests/math.test.rs"]
mod math_test;
#[path = "../../examples/tests/hjkl_arrow_keys.test.rs"]
mod hjkl_arrow_keys_test;