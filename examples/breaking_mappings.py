import map2

reader = map2.Reader(filters=["/dev/input/by-id/example"])
mapper = map2.Mapper()
writer = map2.Writer()

map2.link([reader, mapper, writer])

mapper.map("#q", "t")

mapper.map_key("#^1", "!+2")


fn_called = False

def fn(*args):
  global fn_called
  fn_called = True

mapper.map("#2 down", "a")
mapper.nop("#2 repeat")
mapper.map("#2 up", fn)
