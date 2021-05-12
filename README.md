# Map2
 
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A scripting language that allows complex key remapping on Linux, written in
Rust.

All of the functionality related to interacting with graphical elements such as
getting the active window information is currently only supported on X11.
Wayland support is planned but probably won't be added for some time.

For details check the [documentation](#documentation).

# Examples

Map2 is a very flexible scripting language that can utilize logic control
statements, arithmetic and provides lots of useful built-in functions.

```
// change the 'a' key to 'b'
a::b;

// map the 'c' key to a code block
c::{
  // output text to standard output
  print("hello world");
  
  // type the text using the virtual keyboard
  send("some text{enter}");
};

// define variables and lambda functions
let sum = |argument1, argument2|{
  return argument1 + argument2;
};

let my_sum = sum(1, 2);
print("the sum of 1 and 2 is: " + my_sum);

// do something when the active window changes
on_window_change(||{
  if(active_window_class() == "firefox"){
    print("firefox is now the active window");
    
    // map 'F1' to ctrl+'t' (open new browser tab)
    f1::^t;
  }else{
    print("firefox is not the active window");
    
    // map 'F1' back
    f1::f1;
  }
});
```

For more examples check the [examples directory](examples/README.md).

# Getting started

**tldr:**

- find the input device file descriptors you want to grab
- write them into a file (i.e. `devices.list`)
- write a script file for remapping the keys
- run `$ map2 -d devices.list script_name.m2`


In order to execute a script simply pass it as an argument to the `map2`
command provided by this package.

```
$ map2 script-name.m2
```

By default a script is able to **output** events though a virtual output device,
but in order for a script to **intercept** input events from physical devices it is
necessary to define which devices should be *grabbed* (all events will pass
through the script).

To describe which devices should be grabbed it is necessary to provide a list
of file descriptor paths, regular expressions are also supported.

For example, in order to grab a keyboard and Logitech mouse, one might specify
the device list as such:

*devices.list:*
```
/dev/input/by-id/usb-Logitech_G700s.*-event-kbd
/dev/input/by-path/pci-0000:03:00.0-usb-0:9:1.0-event-kbd
```

In order to find out which file descriptor corresponds to which physical device
one should examine `/dev/input/by-id/` and  `/dev/input/by-path/`.
After defining the device list we can test it using a short script.

*example.m2:*
```
// maps 'a' to 'b'
a::b;
```

And finally execute the script which should successfully remap the keys.

`$ map2 -d devices.list example.m2`

Each device can only be grabbed once. Attempting to run several scripts that
attempt to grab the same device simultaneously will produce warnings and the
device will not be grabbed.

## Install

### Arch Linux

Install stable version:

`$ yay -S map2`

Install latest version:

`$ yay -S map2-git`

Build local package from cloned source:

`$ makepkg -si`

### Other distributions

Install precompiled binary:

- download the latest release from the [releases page](https://github.com/shiro/map2/releases)
- unpack the archive
- copy the files to the respective folders

*if there's an error with system libraries being too old, try building from
source instead.*

Build local package from cloned source:

- `$ cargo build --release`
- copy the compiled binary somewhere in your `PATH` (i.e. `$ cp
  target/release/map2 /usr/bin`)

# Documentation

- [start automatically on startup/login](docs/start-automatically.md)


## Mappings

Basic mappings simply map one key to a different one.

```
a::b; // when 'a' is pressed, 'b' will be pressed instead
```

Under the hood the above expression is a shorthand for the following code:

```
{a down}::{b down};
{a repeat}::{b repeat};
{a up}::{b up};
```

Keys can also be remapped to key sequences, meaning that pressing the key will
result in the key sequence being typed.

```
a::"hello world"; // maps a to "hello world"
```

Also see [Key sequences](#key-sequences).

Complex mappings can be used by putting a code block on the right side of the
mapping expression.

```
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

```
!a::b; // maps 'alt+a' to 'b'
a::+b; // maps 'a' to 'shift+b'
#!^a::+b; // maps 'meta+alt+ctrl+a' to 'shift+b'
```

## Key symbols

To descript keys in key mappings and sequences it is possible to either use
literal key symbols (such as 'a') or any valid input event as defined in
[input-event-codes.h](https://github.com/torvalds/linux/blob/master/include/uapi/linux/input-event-codes.h)
(such as 'KEY_A'). In the case of special keys such as 'left mouse button',
input events need to be used (i.e. `BTN_MOUSE`).

**Non-US keyboard layouts:** Alternative keys that require modifiers (such as
'@', '%', etc.) probably won't work and get mapped to different keys. To fix
this use the modifier flags and the original key instead. (i.e. on the US
layout the '@' symbol requires pressing shift+'2', meaning that `+2` should be
used in simple mappings and `{shift down}2{shift up}` should be used in key
sequences.)

## Variables

Variables can be initialized using the `let` keyword. Assigning a value to a
non-existent variable produces an error.

```
let foo = 33;
```

Variables are dynamically typed, meaning the their type can change at runtime.

```
let foo = 33;
foo = "hello";
```

## Control statements

The flow of execution can be controlled using control statements.

### If statement

If statements can check whether a certain condition is satisfied and execute
code conditionally.

```
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

```
for(let i=0; i<10; i = i+1){
  print(i);
}
```

## Key sequences

Key sequences represent multiple keys with a specific ordering. They can be
used on the right side of a mapping expression in order to type the sequence
when a key is pressed.

```
a::"hello world";
```

Complex keys are also permitted.

```
a::"hello{enter}world{shift down}1{shift up}";
```

## Functions

All functions are either built-in functions provided by the runtime itself or
user defined functions.

### List of built-in functions

#### print(value)

Print the value to the standard output.

```
print(33);

print("hello");

print(||{});
```

#### map_key(trigger, callback)

Maps a key to a callback at runtime, meaning expressions can be used as
parameters. This is useful when the trigger key or callback need to be
dynamically evaluated.

```
map_key("a", ||{
  send("b");
});
```

#### sleep(duration)

Pauses the execution for a certain duration. This does not block other mappings
and tasks.

```
sleep(1000); // sleep for 1 second
```

#### on_window_change(callback)

Registers a callback that is called whenever the active window changes.

```
on_window_change(||{
  print("hello");
});
```

#### active_window_class()

Gets the class name of the currently active window or `Void`.

```
if(active_window_class() == "firefox"){
  print("firefox!");
}
```

#### number_to_char(number: Number)

Converts a number to the corresponding character.

```
let char = num_to_char(97);
print(char); // output: 'a'
```

#### char_to_number(char: String)

Converts a character to the corresponding number.

```
let number = char_to_number("a");
print(number); // output: '97'
```

#### exit(exit_code?: Number)

Terminates the application with the specified exit code. If no exit code is
provided it defaults to '0'.

```
exit(); // exits the application and indicates success
exit(1); // exits the application and indicates an error
```

#### execute(command: String, ...arguments: String[]): String | Void

Executes the given command with the provided arguments.  
The standard output of the command is returned as a string if the command
succeeds. If the command fails, `Void` is returned.

```
let message = execute("echo", "hello", "world");
let now = execute("date");
```

## Comments

Code inside of comments is not evaluated and will be ignored. There exist two
types of comments: line comments and in-line comments.

```
// this is a line comment
print(/* this is an in-line comment */ "hello");
```

# Feature roadmap

- [ ] more built-ins
- [ ] escaped characters in strings and key sequences
- [ ] update documentation and refactor code
- [ ] better tests to avoid regressions
- [ ] pre-packaged binaries for various distros
- [ ] mouse events
- [ ] Wayland support (someday)

# Contributing

If you want to report bugs, add suggestions or help out with development please
check the [issues page](https://github.com/shiro/map2/issues) and open an issue
if it doesn't exist yet.

# License

MIT

# Authors

- shiro <shiro@usagi.io>
