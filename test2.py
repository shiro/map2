import map2
import time

reader = map2.EventReader()
writer = map2.EventWriter(reader)


def hi():
    print("hi")


writer.map("a", "c")
writer.map("s", hi)

time.sleep(5)
