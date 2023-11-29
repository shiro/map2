'''
Creates a virtual output keyboard device and types "Hello world!" on it.
'''
import map2
import time

map2.default(layout = "us")

writer = map2.Writer(capabilities = {"keys": True})

out = map2.VirtualReader()
map2.link([out, writer])
out.send("Hello world!")

time.sleep(1)
