import map2

def on_window_change(active_window_class):
  print("active window class: {}".format(active_window_class))


window = map2.Window()
window.on_window_change(on_window_change)