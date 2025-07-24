import builtins
import typing
from typing_extensions import Unpack
import re

type KeyState = typing.Literal['down'] | typing.Literal['up'] |  typing.Literal['repeat']

type AnyMapper = Mapper | TextMapper | ModifierMapper | ChordMapper
type SrcNode = Reader | AnyMapper
type DstNode = Writer| AnyMapper
type AnyNode = SrcNode | DstNode

class DeviceMatcher(typing.TypedDict):
    path: typing.NotRequired[str | re.Pattern]
    properties: typing.NotRequired[dict[str, str | re.Pattern]]

class DeviceInfo(typing.TypedDict):
    path: typing.NotRequired[str]
    sys_path: typing.NotRequired[str]
    properties: typing.NotRequired[dict[str, str]]

class MapAxisCallback(typing.Protocol):
    def __call__(self, axis: builtins.str, value: builtins.int, /) -> builtins.int | builtins.bool | None: ...

class MapKeyCallback(typing.Protocol):
    def __call__(self, key: builtins.str, value: KeyState, /) -> builtins.str | builtins.bool | None: ...

class MapChordCallback(typing.Protocol):
    def __call__(self, /) -> builtins.str | builtins.bool | None: ...

class TransformerSpecs(typing.TypedDict):
    model: typing.NotRequired[str]
    layout: typing.NotRequired[str]
    variant: typing.NotRequired[str]
    options: typing.NotRequired[str]

class NodeBase:
    def name(self) -> builtins.str: ...
    def unlink_all(self) -> None: ...

class SrcNodeBase:
    def insert_after(self, target: DstNode) -> None: ...
    def link_to(self, target: DstNode) -> None: ...
    def next(self) -> builtins.list[DstNode]: ...
    def unlink_to(self, target: DstNode) -> builtins.bool: ...
    def unlink_to_all(self) -> None: ...

class DstNodeBase:
    def prev(self) -> builtins.list[SrcNode]: ...
    def link_from(self, target: SrcNode) -> None: ...
    def insert_before(self, target: SrcNode) -> None: ...
    def unlink_from(self, target: SrcNode) -> builtins.bool: ...
    def unlink_from_all(self) -> None: ...

class MapperBase:
    def send_after(self, value: builtins.str) -> None: ...

class MapperNewKwargsBase(typing.TypedDict):
    name: typing.NotRequired[builtins.str]

class MapperSnapshot: ...

class MapperNewKwargs(MapperNewKwargsBase, TransformerSpecs): ...

class Mapper(NodeBase, SrcNodeBase, DstNodeBase, MapperBase):
    r"""
    Maps input events to other input events
    """
    def __new__(cls, **kwargs: Unpack[MapperNewKwargs]) -> Mapper: ...
    def map(self, src: builtins.str, dst: builtins.str | MapKeyCallback) -> None:
        r"""
        Maps a key to an action, sequence or function
        """
    def map_key(self, src:builtins.str, dst:builtins.str) -> None:
        r"""
        Maps a key to an action or sequence
        """
    def map_fallback(self, handler: MapKeyCallback) -> None: ...
    def map_relative(self, handler: MapAxisCallback) -> None: ...
    def map_absolute(self, handler: MapAxisCallback) -> None: ...
    def nop(self, src: builtins.str) -> None: ...
    @typing.overload
    def snapshot(self, existing: None = None) -> MapperSnapshot: ...
    @typing.overload
    def snapshot(self, existing: MapperSnapshot) -> None: ...
    def snapshot(self, existing: typing.Optional[MapperSnapshot]=None) -> typing.Optional[MapperSnapshot]: ...
    def send(self, val: builtins.str) -> None: ...

class TextMapperSnapshot: ...

class TextMapperNewKwargs(MapperNewKwargsBase, TransformerSpecs): ...

class TextMapper(NodeBase, SrcNodeBase, DstNodeBase, MapperBase):
    r"""
    Maps sequences of input events to other sequences of input events
    """
    def __new__(cls, **kwargs: Unpack[TextMapperNewKwargs]) -> TextMapper: ...
    def map(self, src: builtins.str, dst: builtins.str | MapKeyCallback) -> None:
        r"""
        Maps a key sequnce to another key sequnce
        """
    @typing.overload
    def snapshot(self, existing: None = None) -> TextMapperSnapshot: ...
    @typing.overload
    def snapshot(self, existing: TextMapperSnapshot) -> None: ...

class ModifierMapperSnapshot: ...

class ModifierMapper(NodeBase, SrcNodeBase, DstNodeBase, MapperBase):
    r"""
    Designates a key as a modifier and maps other keys when held togetger with it
    """
    def __new__(cls, key: builtins.str, **kwargs) -> ModifierMapper: ...
    def map(self, src: builtins.str, dst: builtins.str) -> None:
        r"""
        Maps the modifier and a key to an action, sequence or function
        """
    @typing.overload
    def snapshot(self, existing: None = None) -> ModifierMapperSnapshot: ...
    @typing.overload
    def snapshot(self, existing: ModifierMapperSnapshot) -> None: ...


class ChordMapperSnapshot: ...

class ChordMapperNewKwargs(MapperNewKwargsBase, TransformerSpecs): ...

class ChordMapper(NodeBase, SrcNodeBase, DstNodeBase, MapperBase):
    def __new__(cls, **kwargs: Unpack[ChordMapperNewKwargs]) -> ChordMapper: ...
    def map(self, src: list[builtins.str], dst: builtins.str | MapChordCallback) -> None:
        r"""
        Maps several keys, pressed at once, to a sequence, action or function
        """
    def send(self, val: builtins.str) -> None: ...
    @typing.overload
    def snapshot(self, existing: None = None) -> ChordMapperSnapshot: ...
    @typing.overload
    def snapshot(self, existing: ChordMapperSnapshot) -> None: ...

class ReaderNewKwargs(typing.TypedDict):
    name: typing.NotRequired[str]
    filters: typing.NotRequired[DeviceMatcher | list[str | DeviceMatcher]]

class Reader:
    r"""
    Reads input events from sources such as device nodes
    """
    def __new__(cls, **kwargs: Unpack[ReaderNewKwargs]) -> Reader: ...
    devices: list[DeviceInfo]
    def on_connect(self, handler: typing.Callable) -> None: ...
    def on_disconnect(self, handler: typing.Callable) -> None: ...
    def link_to(self, target:DstNode) -> None: ...
    def unlink_to(self, target:DstNode) -> builtins.bool: ...
    def unlink_to_all(self) -> None: ...
    def unlink_all(self) -> None: ...
    def name(self) -> builtins.str: ...
    def next(self) -> builtins.list[DstNode]: ...
    def send(self, val: builtins.str) -> None: ...
    def __test__write_ev(self, ev: builtins.str) -> None: ...

class AbsInfo(typing.TypedDict):
    value: typing.NotRequired[builtins.int]
    min: typing.NotRequired[builtins.int]
    max: typing.NotRequired[builtins.int]
    fuzz: typing.NotRequired[builtins.int]
    resolution: typing.NotRequired[builtins.int]

class WriterCapabilities(typing.TypedDict):
    keys: typing.NotRequired[bool]
    rel: typing.NotRequired[bool]
    buttons: typing.NotRequired[bool]
    abs: typing.NotRequired[bool | dict[builtins.str, builtins.bool | AbsInfo]]

class WriterNewKwargs(typing.TypedDict):
    name: typing.NotRequired[str]
    clone_from: typing.NotRequired[builtins.str]
    capabilities: typing.NotRequired[WriterCapabilities]

class Writer:
    r"""
    Writes input events to a virtual output device
    """
    def __new__(cls, **kwargs: Unpack[WriterNewKwargs]) -> Writer: ...
    def link_from(self, target:SrcNode) -> None: ...
    def unlink_from(self, target:SrcNode) -> builtins.bool: ...
    def unlink_from_all(self) -> None: ...
    def unlink_all(self) -> None: ...
    def name(self) -> builtins.str: ...
    def prev(self) -> builtins.list[SrcNode]: ...
    def send(self, val: builtins.str) -> None: ...
    def __test__read_ev(self) -> typing.Optional[builtins.str]: ...

class WatcherNewKwargs(typing.TypedDict):
    name: typing.NotRequired[str]
    filters: typing.NotRequired[DeviceMatcher | list[str | DeviceMatcher]]

class Watcher:
    r"""
    Watches input devices and notifies on connect/disconnect
    """
    def __new__(cls, **kwargs: Unpack[WatcherNewKwargs]) -> Watcher: ...
    devices: list[DeviceInfo]
    def on_connect(self, handler: typing.Callable | None) -> None: ...
    def on_disconnect(self, handler: typing.Callable | None) -> None: ...


def default(**kwargs: Unpack[TransformerSpecs]) -> None:
    r"""
    Sets global defaults for various nodes
    """

def link(chain: builtins.list[AnyNode]) -> None:
    r"""
    Links nodes together sequentially, in the provided order
    """

def wait() -> None:
    r"""
    Keeps the program running until explicitly stopped
    """

def exit() -> None:
    r"""
    Stops execution of the program
    """
