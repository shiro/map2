import map2
import time

map2.default()

writer = map2.Writer(capabilities = {"rel": True, "buttons": True})

out = map2.VirtualReader()
out.link(writer)
out.send("h")

time.sleep(1)