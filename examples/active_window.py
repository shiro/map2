import map2

def on_focus(window_info):
  """Called when a window gets focus"""
  print(f"Focused window: {window_info['class']} - {window_info['name']}")

def on_blur(window_info):
  """Called when a window loses focus"""
  print(f"Blurred window: {window_info['class']} - {window_info['name']}")

window = map2.Window()
focus_sub = window.on("focus", on_focus)
blur_sub = window.on("blur", on_blur)

# Keep the listener running
map2.wait()

# Optional: Unsubscribe when done
# window.off(focus_sub)
# window.off(blur_sub)
