'''
Creates a virtual output keyboard device and types "Hello world!" on it.
'''
import map2
import time

map2.default(layout = "us")

reader = map2.Reader()
writer = map2.Writer(capabilities = {"keys": True})

map2.link([reader, writer])

reader.send("Hello world!")

# keep running for 1sec so the event can be processed
time.sleep(1)