#!/usr/bin/env python3
# Parses linux/input.h scanning for #define KEY_FOO 134
# Prints Rust source header files that can be used for
# mapping and lookup tables.
#
# The original version of this file is in libevdev
#

import re
import sys


class Bits(object):
    pass


prefixes = [
    "EV_",
    "REL_",
    "ABS_",
    "KEY_",
    "BTN_",
    "LED_",
    "SND_",
    "MSC_",
    "SW_",
    "FF_",
    "SYN_",
    "REP_",
    "INPUT_PROP_",
    "BUS_"
]

blacklist = [
    "EV_VERSION",
    "BTN_MISC",
    "BTN_MOUSE",
    "BTN_JOYSTICK",
    "BTN_GAMEPAD",
    "BTN_DIGI",
    "BTN_WHEEL",
    "BTN_TRIGGER_HAPPY",
]

btn_additional = [
    [0, "BTN_A"],
    [0, "BTN_B"],
    [0, "BTN_X"],
    [0, "BTN_Y"],
]

event_names = [
    "REL_",
    "ABS_",
    "KEY_",
    "BTN_",
    "LED_",
    "SND_",
    "MSC_",
    "SW_",
    "FF_",
    "SYN_",
    "REP_",
]


def convert(name):
    s1 = re.sub('(.)([A-Z][a-z]+)', r'\1_\2', name)
    return re.sub('([a-z0-9])([A-Z])', r'\1_\2', s1).lower()


def get_enum_name(prefix):
    if prefix == "ev":
        return "EventType"
    elif prefix == "input_prop":
        return "InputProp"
    elif prefix == "bus":
        return "BusType"
    else:
        return "EV_" + prefix.upper()


def print_enums(bits, prefix):

    if not hasattr(bits, prefix):
        return

    enum_name = get_enum_name(prefix)
    associated_names = []

    print("#[allow(non_camel_case_types)]")
    print('#[cfg_attr(feature = "serde", derive(Serialize), derive(Deserialize))]')
    print("#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]")
    print("pub enum %s {" % enum_name)
    for val, names in list(getattr(bits, prefix).items()):
        # Note(ndesh): We use EV_MAX as proxy to write the UNKnown event
        if names[0] == "EV_MAX":
            print("    EV_UNK,")
        print("    %s = %s," % (names[0], val))
        if len(names) > 1:
            associated_names.extend([(names[0], names[1:])])
    if prefix == "key":
        for val, names in list(getattr(bits, "btn").items()):
            print("    %s = %s," % (names[0], val))
            if len(names) > 1:
                associated_names.extend([(names[0], names[1:])])
    print("}")
    print("")

    if len(associated_names) > 0:
        print("impl %s {" % enum_name)
        for orig, names in associated_names:
            for name in names:
                print("    pub const %s: %s = %s::%s;" %
                      (name, enum_name, enum_name, orig))
        print("}")
        print("")


def print_enums_convert_fn(bits, prefix):
    if prefix == "ev":
        fn_name = "EventType"
    elif prefix == "input_prop":
        fn_name = "InputProp"
    elif prefix == "bus":
        fn_name = "BusType"
    else:
        fn_name = "EV_" + prefix.upper()

    if not hasattr(bits, prefix):
        return

    print("pub fn %s(code: u32) -> Option<%s> {" %
          ("int_to_" + convert(fn_name), fn_name))
    print("    match code {")
    for val, names in list(getattr(bits, prefix).items()):
        # Note(ndesh): We use EV_MAX as proxy to write the UNKnown event
        if names[0] == "EV_MAX":
            print("        c if c < 31 => Some(EventType::EV_UNK),")
        print("        %s => Some(%s::%s)," % (val, fn_name, names[0]))
    if prefix == "key":
        for val, names in list(getattr(bits, "btn").items()):
            print("        %s => Some(%s::%s)," % (val, fn_name, names[0]))
    print("        _ => None,")
    print("    }")
    print("}")
    print("")


def print_enums_fromstr(bits, prefix):

    if not hasattr(bits, prefix):
        return

    enum_name = get_enum_name(prefix)

    print('impl std::str::FromStr for %s {' % enum_name)
    print('    type Err = ();')
    print('    fn from_str(s: &str) -> Result<Self, Self::Err> {')
    print('        match s {')
    for _val, names in list(getattr(bits, prefix).items()):
        name = names[0]
        print('            "%s" => Ok(%s::%s),' % (name, enum_name, name))
    print('            _ => Err(()),')
    print('        }')
    print('    }')
    print('}')
    print('')


def print_event_code(bits, prefix):
    if not hasattr(bits, prefix):
        return

    print("#[allow(non_camel_case_types)]")
    print('#[cfg_attr(feature = "serde", derive(Serialize), derive(Deserialize))]')
    print("#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]")
    print("pub enum EventCode {")
    for val, [name] in list(getattr(bits, prefix).items()):
        if name[3:]+"_" in event_names:
            print("    %s(%s)," % (name, name))
        elif name == "EV_FF_STATUS":
            print("    EV_FF_STATUS(EV_FF),")
        else:
            # Note(ndesh): We use EV_MAX as proxy to write the UNKnown event
            if name == "EV_MAX":
                print("    EV_UNK { event_type: u32, event_code: u32 },")
            print("    %s," % (name))
    if prefix == "key":
        for val, names in list(getattr(bits, "btn").items()):
            for name in names:
                print("    %s = %s," % (name, val))
    print("}")
    print("")


def print_mapping_table(bits):
    for prefix in prefixes:
        if prefix == "BTN_":
            continue
        print_enums(bits, prefix[:-1].lower())
        print_enums_convert_fn(bits, prefix[:-1].lower())
        print_enums_fromstr(bits, prefix[:-1].lower())
        if prefix == "EV_":
            print_event_code(bits, prefix[:-1].lower())


def parse_define(bits, line):
    m = re.match(r"^#define\s+(\w+)\s+(\w+)", line)
    if m is None:
        return

    name = m.group(1)

    if name in blacklist:
        return

    try:
        value = int(m.group(2), 0)
    except ValueError:
        return

    for prefix in prefixes:
        if not name.startswith(prefix):
            continue

        attrname = prefix[:-1].lower()

        if not hasattr(bits, attrname):
            setattr(bits, attrname, {})
        b = getattr(bits, attrname)
        if value in b:
            b[value].append(name)
        else:
            b[value] = [name]


def parse(fp):
    bits = Bits()

    lines = fp.readlines()
    for line in lines:
        if not line.startswith("#define"):
            continue
        parse_define(bits, line)

    return bits


def usage(prog):
    print("Usage: {} <files>".format(prog))


if __name__ == "__main__":
    if len(sys.argv) <= 1:
        usage(sys.argv[0])
        sys.exit(2)

    print("/* THIS FILE IS GENERATED, DO NOT EDIT */")
    print("")
    print('#[cfg(feature = "serde")]')
    print("use serde::{Deserialize, Serialize};")
    print("")

    for arg in sys.argv[1:]:
        with open(arg) as f:
            bits = parse(f)
            print_mapping_table(bits)
