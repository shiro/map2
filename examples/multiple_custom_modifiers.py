import map2

reader = map2.Reader(patterns=["/dev/input/by-id/example"])
writer = map2.Writer(clone_from = "/dev/input/by-id/example")

mod1 = map2.ModifierMapper("a")
mod2 = map2.ModifierMapper("b")

map2.link([reader, mod1, mod2, writer])


mod1.map("z", "1")
mod2.map("z", "2")


mod2.map("a", "y")

# 1: exclusive - only 1 active
# 2: double mod combos
