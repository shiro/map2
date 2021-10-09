import map2

reader = map2.Reader(patterns=["/dev/input/by-path/pci-0000:03:00.0-usb-0:9:1.0-event-kbd"])
writer = map2.Writer(reader)

print("start")

writer.map("pagedown", lambda: map2.exit())

tab_pressed = False
key_pressed = False


def tab_down(): global tab_pressed, key_pressed; tab_pressed = True; key_pressed = False
writer.map("{tab down}", tab_down)


def tab_up():
    global tab_pressed, key_pressed
    tab_pressed = False
    if not key_pressed: writer.send("{tab}")
writer.map("{tab up}", tab_up)


def key_tab_mod(key):
    global tab_pressed
    if tab_pressed: writer.send("{alt down}{meta down}{shift down}" + key + "{alt up}{meta up}{shift up}");


caps_down = False
writer.map("^capslock", "capslock")

def capslock_down(): global caps_down, key_pressed; caps_down = True; key_pressed = False; writer.send("{ctrl down}")
writer.map("{capslock down}", capslock_down)


def capslock_up():
    global caps_down, key_pressed
    caps_down = False
    key_pressed = False
    writer.send("{ctrl up}")
    if not key_pressed: writer.send("{esc}")
writer.map("{capslock up}", capslock_up)

lalt = False

def leftalt_down(): global lalt; lalt = True; writer.send_modifier("{leftalt down}")
writer.map("{leftalt down}", leftalt_down)
def leftalt_up(): global lalt; lalt = False; writer.send_modifier("{leftalt up}")
writer.map("!{leftalt up}", leftalt_up)

ralt = False

def rightalt_down(): global ralt; ralt = True; writer.send_modifier("{rightalt down}")
writer.map("{rightalt down}", rightalt_down)
def rightalt_up(): global ralt; ralt = False; writer.send_modifier("{rightalt up}")
writer.map("!{rightalt up}", rightalt_up)

def directional_mod(key, direction):
    def map(key_down, key_down_str, key_up, key_up_str):
        global key_pressed
        def key_down_fn():
            global lalt, ralt, caps_down, key_pressed
            key_pressed = True

            if lalt:
                writer.send("{" + direction + " down}")
                return 0
            if caps_down:
                writer.send("{alt down}{meta down}{shift down}{ctrl down}" + key + "{alt up}{meta up}{shift up}{ctrl up}")
                return 0

            if ralt:
                writer.send("{rightalt up}{alt down}{meta down}" + key + "{alt up}{meta up}{rightalt down}")
                return 0

            if key_tab_mod(key_down): return 0
            writer.send(key_down_str)
        writer.map(key_down, key_down_fn)

        def key_up_fn():
            global lalt, ralt
            if lalt: writer.send("{" + direction + " up}"); return 0
            if ralt: return 0
            writer.send(key_up_str)
        writer.map(key_up, key_up_fn)

    key_down = "{" + key + " down}"
    key_up = "{" + key + " up}"

    map(key_down, key_down, key_up, key_up)
    map("!" + key_down, "{alt down}" + key_down + "{alt up}", "!" + key_up, "{alt down}" + key_up + "{alt up}")

def handle_key(key):
    key_down = "{" + key + " down}"

    if key == "h": directional_mod(key, "left"); return 0
    if key == "j": directional_mod(key, "down"); return 0
    if key == "k": directional_mod(key, "up"); return 0
    if key == "l": directional_mod(key, "right"); return 0

    def key_down_fn():
        global key_pressed
        key_pressed = True
        if key_tab_mod(key_down): return 0
        writer.send(key_down)

    writer.map(key_down, key_down_fn)

for i in range(97, 97 + 26): handle_key(chr(i))
for i in range(ord("0"), ord("9")): handle_key(chr(i))
handle_key("space")
handle_key("/")
handle_key(";")
handle_key("]")
handle_key("[")

map2.wait()
