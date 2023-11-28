import map2
import time

map2.default()

writer = map2.Writer(capabilities = {"rel": True, "buttons": True})

out = map2.VirtualReader()
map2.link([out, writer])
out.send("h")

time.sleep(1)