import wasmtime
from many_arguments import ManyArguments, ManyArgumentsImports
from helpers import TestWasi

class MyImports:
    def many_arguments(self,
            a1: int,
            a2: int,
            a3: int,
            a4: int,
            a5: int,
            a6: int,
            a7: int,
            a8: int,
            a9: int,
            a10: int,
            a11: int,
            a12: int,
            a13: int,
            a14: int,
            a15: int,
            a16: int) -> None:
        assert(a1 == 1)
        assert(a2 == 2)
        assert(a3 == 3)
        assert(a4 == 4)
        assert(a5 == 5)
        assert(a6 == 6)
        assert(a7 == 7)
        assert(a8 == 8)
        assert(a9 == 9)
        assert(a10 == 10)
        assert(a11 == 11)
        assert(a12 == 12)
        assert(a13 == 13)
        assert(a14 == 14)
        assert(a15 == 15)
        assert(a16 == 16)


def run() -> None:
    store = wasmtime.Store()
    wasm = ManyArguments(store, ManyArgumentsImports(MyImports(), TestWasi()))

    wasm.many_arguments(store, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13,14, 15, 16)

if __name__ == '__main__':
    run()
