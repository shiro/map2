import map2
import time

reader = map2.Reader(patterns=[
    "/dev/input/by-id/usb-Logitech_USB_Receiver-if01-event-.*",
    "/dev/input/by-id/usb-Logitech_G700s_Rechargeable_Gaming_Mouse_017DF9570007-.*-event-.*",
    "/dev/input/by-path/pci-0000:03:00.0-usb-0:9:1.0-event-kbd",
])
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
    writer.send("{ctrl up}")
    if not key_pressed: writer.send("{esc}")
    key_pressed = False
    caps_down = False
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

for i in range(ord("a"), ord("z")): handle_key(chr(i))
for i in range(ord("0"), ord("9")): handle_key(chr(i))
for char in ["space", "/", ":", ".", ",", ";", "[", "]"]: handle_key(char)


def setup_mouse():
    writer.map_key("f13", "kp1")
    writer.map_key("f14", "kp2")
    writer.map_key("f15", "kp3")
    writer.map_key("f16", "kp4")
    writer.map_key("f17", "kp5")
    writer.map_key("f18", "kp6")
    writer.map_key("f19", "kp7")
    writer.map_key("f20", "kp8")
    writer.map_key("f21", "kp9")

setup_mouse()


def map_figma_shortcut(key, command):
    def key_fn():
        writer.send("{ctrl down}/{ctrl up}")
        time.sleep(0.2)
        writer.send(command + "{enter}")
    writer.map(key, key_fn)


def on_window_change(active_window_class):
    setup_mouse()

    if active_window_class == "firefox":
        writer.map_key("f13", "^tab")
        writer.map_key("+f13", "+^tab")
        writer.map_key("f14", "^t")
        writer.map_key("f16", "f5")
        writer.map_key("f21", "^w")
    elif active_window_class == "figma-linux":
        map_figma_shortcut("f13", "palette-pick")
        map_figma_shortcut("f14", "atom-sync")
        map_figma_shortcut("f15", "batch styler")
        map_figma_shortcut("f16", "chroma colors")
        map_figma_shortcut("f17", "scripter")
        map_figma_shortcut("f20", "theme-flip")


window = map2.Window()
window.on_window_change(on_window_change)

map2.wait()