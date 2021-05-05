# Key mods RS

A scripting language that allows complex key remapping on Linux, written in
Rust.

For details check the [documentation](#documentation).

# Install

## Arch Linux

Just run:

`makepkg -si`

# Documentation

## Mappings

Basic mappings simply map one key to a different one.

```.rust
a::b; // when 'a' is pressed, 'b' will be pressed instead
```

Under the hood the above expression is a shorthand for the following code:

```.rust
{a down}::{b down};
{a repeat}::{b repeat};
{a up}::{b up};
```

Keys can also be remapped to key sequences, meaning that pressing the key will
result in the key sequence being typed.

```.rust
a::"hello world"; // maps a to "hello world"
```

Also see [Key sequences](#key-sequences).

Complex mappings can be used by putting a code block on the right side of the
mapping expression.

```.rust
a::{
  sleep(200);
  print("hi");
  send("b");
};
```

### Modifier flags

Modifier flags can be added to key mappings in order to press down the key with
the given modifier. Multiple flags can be used at once.  
The following modifier flags can be used:

- `^` - control
- `+` - shift
- `!` - alt
- `#` - meta

Flags can be used on both sides of a mapping expression:

```.rust
!a::b; // maps 'alt+a' to 'b'
a::+b; // maps 'a' to 'shift+b'
#!^a::+b; // maps 'meta+alt+ctrl+a' to 'shift+b'
```

## Variables

Variables can be initialized using the `let` keyword. Assigning a value to a
non-existent variable produces an error.

```.rust
let foo = 33;
```

Variables are dynamically typed, meaning the their type can change at runtime.

```.rust
let foo = 33;
foo = "hello";
```

## Control statements

The flow of execution can be controlled using control statements.

### If statement

If statements can check whether a certain condition is satisfied and execute
code conditionally.

```.rust
let a = 3;

if (a == 1){
  print("a is 1");
}else if (a == 3){
  print("a is 3");
}else{
  print("a is not 1 or 3");
}
```

### For loop

For loops are useful when a code block should be run several times.

```.rust
for(let i=0; i<10; i = i+1){
  print(i);
}
```

## Key sequences

Key sequences represent multiple keys with a specific ordering. They can be
used on the right side of a mapping expression in order to type the sequence
when a key is pressed.

```.rust
a::"hello world";
```

Complex keys are also permitted.

```.rust
a::"hello{enter}world{shift down}1{shift up}";
```

## Functions

All functions are either built-in functions provided by the runtime itself or
user defined functions.

### List of built-in functions

#### print(value)

Print the value to the standard output.

```.rust
print(33);

print("hello");

print(||{});
```

#### map_key(trigger, callback)

Maps a key to a callback at runtime, meaning expressions can be used as
parameters. This is useful when the trigger key or callback need to be
dynamically evaluated.

```.rust
map_key("a", ||{
  send("b");
});
```

#### sleep(duration)

Pauses the execution for a certain duration. This does not block other mappings
and tasks.

```.rust
sleep(1000); // sleep for 1 second
```

#### on_window_change(callback)

Registers a callback that is called whenever the active window changes.

```.rust
on_window_change(||{
  print("hello");
});
```

#### active_window_class()

Gets the class name of the currently active window or `Void`.

```.rust
if(active_window_class() == "firefox"){
  print("firefox!");
}
```

#### number_to_char(number: Number)

Converts a number to the corresponding character.

```.rust
let char = num_to_char(97);
print(char); // output: 'a'
```

#### char_to_number(char: String)

Converts a character to the corresponding number.

```.rust
let number = char_to_number("a");
print(number); // output: '97'
```

# Feature roadmap

- [ ] all basic expressions (i.e. multiplication, division, string and number
      concatenation, etc.)
- [ ] proper error reporting
- [ ] more command line options

# Authors

- shiro <shiro@usagi.io>

# License

MIT
