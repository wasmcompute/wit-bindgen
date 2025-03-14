from typing import Callable, List, Tuple
import wasmtime
from invalid import Invalid, InvalidImports
from invalid.imports import Imports, imports as i
from helpers import TestWasi

class MyImports(Imports):
    def roundtrip_u8(self, x: int) -> int:
        raise Exception('unreachable')

    def roundtrip_s8(self, x: int) -> int:
        raise Exception('unreachable')

    def roundtrip_u16(self, x: int) -> int:
        raise Exception('unreachable')

    def roundtrip_s16(self, x: int) -> int:
        raise Exception('unreachable')

    def roundtrip_bool(self, x: bool) -> bool:
        raise Exception('unreachable')

    def roundtrip_char(self, x: str) -> str:
        raise Exception('unreachable')

    def roundtrip_enum(self, x: i.E) -> i.E:
        raise Exception('unreachable')

    def unaligned1(self, x: List[int]) -> None:
        raise Exception('unreachable')

    def unaligned2(self, x: List[int]) -> None:
        raise Exception('unreachable')

    def unaligned3(self, x: List[int]) -> None:
        raise Exception('unreachable')

    def unaligned4(self, x: List[i.Flag32]) -> None:
        raise Exception('unreachable')

    def unaligned5(self, x: List[i.Flag64]) -> None:
        raise Exception('unreachable')

    def unaligned6(self, x: List[i.UnalignedRecord]) -> None:
        raise Exception('unreachable')

    def unaligned7(self, x: List[float]) -> None:
        raise Exception('unreachable')

    def unaligned8(self, x: List[float]) -> None:
        raise Exception('unreachable')

    def unaligned9(self, x: List[str]) -> None:
        raise Exception('unreachable')

    def unaligned10(self, x: List[bytes]) -> None:
        raise Exception('unreachable')


def new_wasm() -> Tuple[wasmtime.Store, Invalid]:
    store = wasmtime.Store()
    wasm = Invalid(store, InvalidImports(MyImports(), TestWasi()))
    return (store, wasm)

def run() -> None:
    (store, wasm) = new_wasm()

    def assert_throws(f: Callable, msg: str) -> None:
        try:
            f()
            raise RuntimeError('expected exception')
        except TypeError as e:
            actual = str(e)
        except OverflowError as e:
            actual = str(e)
        except ValueError as e:
            actual = str(e)
        except IndexError as e:
            actual = str(e)
        if not msg in actual:
            print(actual)
            assert(msg in actual)

    # FIXME(#376) these should succeed
    assert_throws(lambda: wasm.invalid_bool(store), 'discriminant for bool')
    (store, wasm) = new_wasm()
    assert_throws(lambda: wasm.invalid_u8(store), 'must be between')
    (store, wasm) = new_wasm()
    assert_throws(lambda: wasm.invalid_s8(store), 'must be between')
    (store, wasm) = new_wasm()
    assert_throws(lambda: wasm.invalid_u16(store), 'must be between')
    (store, wasm) = new_wasm()
    assert_throws(lambda: wasm.invalid_s16(store), 'must be between')

    (store, wasm) = new_wasm()
    assert_throws(lambda: wasm.invalid_char(store), 'not a valid char')
    (store, wasm) = new_wasm()
    assert_throws(lambda: wasm.invalid_enum(store), 'not a valid E')

    # FIXME(#370) should call `unalignedN` and expect an error

if __name__ == '__main__':
    run()
