<div align="center">
  <h1>map2</h1>
  <h3>Linux input remapping<br />Remap your keyboard, mouse, controller and more!</h3>

  [![GitHub](https://img.shields.io/badge/GitHub-code-blue?logo=github)](https://github.com/shiro/map2)
  [![MIT License](https://img.shields.io/github/license/shiro/map2?color=43A047&logo=linux&logoColor=white)](https://github.com/shiro/map2/blob/master/LICENSE)
  [![Discord](https://img.shields.io/discord/1178929723208896543?label=Discord&color=5E35B1&logo=discord&logoColor=ffffff)](https://discord.gg/brKgH43XQN)
  [![Build](https://img.shields.io/github/actions/workflow/status/shiro/map2/CI.yml?color=00897B&logo=github-actions&logoColor=white)](https://github.com/shiro/map2/actions/workflows/CI.yml)
  [![Donate](https://img.shields.io/badge/Ko--Fi-donate-orange?logo=ko-fi&color=E53935)](https://ko-fi.com/C0C3RTCCI)
</div>

Want to remap your input devices like keyboards, mice, controllers and more?  
There's nothing you can't remap with **map2**!

- 🖱️ **Remap keys, mouse events, controllers, pedals, and more!**
- 🔧  **Highly configurable**, using Python
- 🚀 **Blazingly fast**, written in Rust
- 📦 **Tiny install size** (around 5Mb), almost no dependencies
- ❤️ **Open source**, made with love

Visit our [official documentation](https://shiro.github.io/map2/en/basics/introduction)
for the full feature list and API.

---

<div align="center">
    <b>If you like open source, consider supporting</b>
    <br/>
    <br/>
    <a href='https://ko-fi.com/C0C3RTCCI' target='_blank'><img height='36' style='border:0px;height:36px;' src='https://storage.ko-fi.com/cdn/kofi3.png?v=3' border='0' alt='Buy Me a Coffee at ko-fi.com' /></a>
</div>

## Install

The easiest way is to use `pip`:

```bash
pip install map2
```

For more, check out the [Install documentation](https://shiro.github.io/map2/en/basics/install/).

After installing, please read the
[Getting started documentation](https://shiro.github.io/map2/en/basics/getting-started).

## Example

```python
import map2

# readers intercept all keyboard inputs and forward them
reader = map2.Reader(patterns=["/dev/input/by-id/my-keyboard"])
# mappers change inputs, you can also chain multiple mappers!
mapper = map2.Mapper()
# writers create new virtual devices we can write into
writer = map2.Writer(clone_from = "/dev/input/by-id/my-keyboard")
# finally, link nodes to control the event flow
map2.link([reader, mapper, writer])

# map the "a" key to "B"
mapper.map("a", "B")

# map "CTRL + ALT + u" to "META + SHIFT + w"
mapper.map("^!u", "#+w")

# key sequences are also supported
mapper.map("s", "hello world!")

# use the full power of Python using functions
def custom_function(key, state):
  print("called custom function")

  # custom conditions and complex sequences
  if key == "d":
    return "{ctrl down}a{ctrl up}"
  return True

mapper.map("d", custom_function)
```

## Build from source

To build from source, make sure python and rust are installed.

```bash
# create a python virtual environment
python -m venv .env
source .env/bin/activate

# build the library
maturin develop
```

While the virtual environment is activated, all scripts ran from this terminal
will use the newly built version of map2.


## Contributing

If you want to report bugs, add suggestions or help out with development please
check the [Discord channel](https://discord.gg/brKgH43XQN) and the [issues page](https://github.com/shiro/map2/issues) and open an issue
if it doesn't exist yet.

## License

MIT

## Authors

- shiro <shiro@usagi.io>
