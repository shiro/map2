import map2


# r = map2.Reader(filters=[
#     { "path": ".*/event.*", "name": "Wireless Controller" },
#     { "path": ".*/event.*", "name": "Sony Interactive Entertainment Wireless Controller" },
# ])
#
#
# r = None
#
# map2.wait()

# reader = map2.Reader(name=99)
# watcher = map2.Watcher()
# print(watcher.devices)
# watcher.on_connect(lambda info: print(info.properties["ID_SERIAL"]))



map2.wait()
