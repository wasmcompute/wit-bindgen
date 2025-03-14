(component
  (type (;0;) (func (param "x" u8)))
  (type (;1;) (func (param "x" s8)))
  (type (;2;) (func (param "x" u16)))
  (type (;3;) (func (param "x" s16)))
  (type (;4;) (func (param "x" u32)))
  (type (;5;) (func (param "x" s32)))
  (type (;6;) (func (param "x" u64)))
  (type (;7;) (func (param "x" s64)))
  (type (;8;) (func (param "p1" u8) (param "p2" s8) (param "p3" u16) (param "p4" s16) (param "p5" u32) (param "p6" s32) (param "p7" u64) (param "p8" s64)))
  (type (;9;) (func (result u8)))
  (type (;10;) (func (result s8)))
  (type (;11;) (func (result u16)))
  (type (;12;) (func (result s16)))
  (type (;13;) (func (result u32)))
  (type (;14;) (func (result s32)))
  (type (;15;) (func (result u64)))
  (type (;16;) (func (result s64)))
  (type (;17;) (tuple s64 u8))
  (type (;18;) (func (result 17)))
  (type (;19;) (func (result "a" s64) (result "b" u8)))
  (type (;20;) 
    (instance
      (alias outer 1 0 (type (;0;)))
      (export "a1" (func (type 0)))
      (alias outer 1 1 (type (;1;)))
      (export "a2" (func (type 1)))
      (alias outer 1 2 (type (;2;)))
      (export "a3" (func (type 2)))
      (alias outer 1 3 (type (;3;)))
      (export "a4" (func (type 3)))
      (alias outer 1 4 (type (;4;)))
      (export "a5" (func (type 4)))
      (alias outer 1 5 (type (;5;)))
      (export "a6" (func (type 5)))
      (alias outer 1 6 (type (;6;)))
      (export "a7" (func (type 6)))
      (alias outer 1 7 (type (;7;)))
      (export "a8" (func (type 7)))
      (alias outer 1 8 (type (;8;)))
      (export "a9" (func (type 8)))
      (alias outer 1 9 (type (;9;)))
      (export "r1" (func (type 9)))
      (alias outer 1 10 (type (;10;)))
      (export "r2" (func (type 10)))
      (alias outer 1 11 (type (;11;)))
      (export "r3" (func (type 11)))
      (alias outer 1 12 (type (;12;)))
      (export "r4" (func (type 12)))
      (alias outer 1 13 (type (;13;)))
      (export "r5" (func (type 13)))
      (alias outer 1 14 (type (;14;)))
      (export "r6" (func (type 14)))
      (alias outer 1 15 (type (;15;)))
      (export "r7" (func (type 15)))
      (alias outer 1 16 (type (;16;)))
      (export "r8" (func (type 16)))
      (alias outer 1 18 (type (;17;)))
      (export "pair-ret" (func (type 17)))
      (alias outer 1 19 (type (;18;)))
      (export "multi-ret" (func (type 18)))
    )
  )
  (import "integers" (instance (;0;) (type 20)))
)