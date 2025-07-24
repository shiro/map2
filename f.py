from time import sleep
import map2
import re


# r = map2.Reader(filters=[
#     { "path": ".*/event.*", "name": "Wireless Controller" },
#     { "path": ".*/event.*", "name": "Sony Interactive Entertainment Wireless Controller" },
# ])
#
#
# r = None
#
# map2.wait()

r = re.compile(r"hell.*d")


mapper = map2.Mapper()
mapper.map("a", "b")

mapper.send("{a down}")
sleep(1)
mapper.reset()

# reader = map2.Reader(name="", filters=99)
# watcher = map2.Watcher(filters=[{"properties": {"NAME": re.compile(r"Apple.*Touch")}}])
# for device in watcher.devices:
#     print(device["sys_path"])
# watcher.on_connect(lambda info: print(info.properties["ID_SERIAL"]))



map2.wait()
