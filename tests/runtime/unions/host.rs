use anyhow::Result;

wit_bindgen_host_wasmtime_rust::generate!({
    import: "../../tests/runtime/unions/imports.wit",
    default: "../../tests/runtime/unions/exports.wit",
    name: "exports",
});

#[derive(Default)]
pub struct MyImports;

impl imports::Imports for MyImports {
    fn add_one_integer(&mut self, num: imports::AllIntegers) -> Result<imports::AllIntegers> {
        use imports::AllIntegers;
        Ok(match num {
            AllIntegers::Bool(false) => AllIntegers::Bool(true),
            AllIntegers::Bool(true) => AllIntegers::Bool(false),
            AllIntegers::U8(n) => AllIntegers::U8(n.wrapping_add(1)),
            AllIntegers::U16(n) => AllIntegers::U16(n.wrapping_add(1)),
            AllIntegers::U32(n) => AllIntegers::U32(n.wrapping_add(1)),
            AllIntegers::U64(n) => AllIntegers::U64(n.wrapping_add(1)),
            AllIntegers::I8(n) => AllIntegers::I8(n.wrapping_add(1)),
            AllIntegers::I16(n) => AllIntegers::I16(n.wrapping_add(1)),
            AllIntegers::I32(n) => AllIntegers::I32(n.wrapping_add(1)),
            AllIntegers::I64(n) => AllIntegers::I64(n.wrapping_add(1)),
        })
    }
    fn add_one_float(&mut self, num: imports::AllFloats) -> Result<imports::AllFloats> {
        use imports::AllFloats;
        Ok(match num {
            AllFloats::F32(n) => AllFloats::F32(n + 1.0),
            AllFloats::F64(n) => AllFloats::F64(n + 1.0),
        })
    }
    fn replace_first_char(&mut self, text: imports::AllText, c: char) -> Result<imports::AllText> {
        use imports::AllText;
        Ok(match text {
            AllText::Char(_) => AllText::Char(c),
            AllText::String(t) => AllText::String(format!("{}{}", c, &t[1..])),
        })
    }
    fn identify_integer(&mut self, num: imports::AllIntegers) -> Result<u8> {
        use imports::AllIntegers;
        Ok(match num {
            AllIntegers::Bool { .. } => 0,
            AllIntegers::U8 { .. } => 1,
            AllIntegers::U16 { .. } => 2,
            AllIntegers::U32 { .. } => 3,
            AllIntegers::U64 { .. } => 4,
            AllIntegers::I8 { .. } => 5,
            AllIntegers::I16 { .. } => 6,
            AllIntegers::I32 { .. } => 7,
            AllIntegers::I64 { .. } => 8,
        })
    }
    fn identify_float(&mut self, num: imports::AllFloats) -> Result<u8> {
        use imports::AllFloats;
        Ok(match num {
            AllFloats::F32 { .. } => 0,
            AllFloats::F64 { .. } => 1,
        })
    }
    fn identify_text(&mut self, text: imports::AllText) -> Result<u8> {
        use imports::AllText;
        Ok(match text {
            AllText::Char { .. } => 0,
            AllText::String { .. } => 1,
        })
    }
    fn identify_duplicated(&mut self, dup: imports::DuplicatedS32) -> Result<u8> {
        use imports::DuplicatedS32;
        Ok(match dup {
            DuplicatedS32::I320 { .. } => 0,
            DuplicatedS32::I321 { .. } => 1,
            DuplicatedS32::I322 { .. } => 2,
        })
    }
    fn add_one_duplicated(
        &mut self,
        dup: imports::DuplicatedS32,
    ) -> Result<imports::DuplicatedS32> {
        use imports::DuplicatedS32;
        Ok(match dup {
            DuplicatedS32::I320(n) => DuplicatedS32::I320(n.wrapping_add(1)),
            DuplicatedS32::I321(n) => DuplicatedS32::I321(n.wrapping_add(1)),
            DuplicatedS32::I322(n) => DuplicatedS32::I322(n.wrapping_add(1)),
        })
    }
    fn identify_distinguishable_num(&mut self, num: imports::DistinguishableNum) -> Result<u8> {
        use imports::DistinguishableNum;
        Ok(match num {
            DistinguishableNum::F64 { .. } => 0,
            DistinguishableNum::I64 { .. } => 1,
        })
    }
    fn add_one_distinguishable_num(
        &mut self,
        num: imports::DistinguishableNum,
    ) -> Result<imports::DistinguishableNum> {
        use imports::DistinguishableNum;
        Ok(match num {
            DistinguishableNum::F64(n) => DistinguishableNum::F64(n + 1.0),
            DistinguishableNum::I64(n) => DistinguishableNum::I64(n.wrapping_add(1)),
        })
    }
}

fn run(wasm: &str) -> Result<()> {
    let (exports, mut store) = crate::instantiate(
        wasm,
        |linker| imports::add_to_linker(linker, |cx| -> &mut MyImports { &mut cx.imports }),
        |store, module, linker| Exports::instantiate(store, module, linker),
    )?;

    exports.test_imports(&mut store)?;

    // Booleans
    assert!(matches!(
        exports.add_one_integer(&mut store, AllIntegers::Bool(false))?,
        AllIntegers::Bool(true)
    ));
    assert!(matches!(
        exports.add_one_integer(&mut store, AllIntegers::Bool(true))?,
        AllIntegers::Bool(false)
    ));
    // Unsigned integers
    assert!(matches!(
        exports.add_one_integer(&mut store, AllIntegers::U8(0))?,
        AllIntegers::U8(1)
    ));
    assert!(matches!(
        exports.add_one_integer(&mut store, AllIntegers::U8(u8::MAX))?,
        AllIntegers::U8(0)
    ));
    assert!(matches!(
        exports.add_one_integer(&mut store, AllIntegers::U16(0))?,
        AllIntegers::U16(1)
    ));
    assert!(matches!(
        exports.add_one_integer(&mut store, AllIntegers::U16(u16::MAX))?,
        AllIntegers::U16(0)
    ));
    assert!(matches!(
        exports.add_one_integer(&mut store, AllIntegers::U32(0))?,
        AllIntegers::U32(1)
    ));
    assert!(matches!(
        exports.add_one_integer(&mut store, AllIntegers::U32(u32::MAX))?,
        AllIntegers::U32(0)
    ));
    assert!(matches!(
        exports.add_one_integer(&mut store, AllIntegers::U64(0))?,
        AllIntegers::U64(1)
    ));
    assert!(matches!(
        exports.add_one_integer(&mut store, AllIntegers::U64(u64::MAX))?,
        AllIntegers::U64(0)
    ));
    // Signed integers
    assert!(matches!(
        exports.add_one_integer(&mut store, AllIntegers::I8(0))?,
        AllIntegers::I8(1)
    ));
    assert!(matches!(
        exports.add_one_integer(&mut store, AllIntegers::I8(i8::MAX))?,
        AllIntegers::I8(i8::MIN)
    ));
    assert!(matches!(
        exports.add_one_integer(&mut store, AllIntegers::I16(0))?,
        AllIntegers::I16(1)
    ));
    assert!(matches!(
        exports.add_one_integer(&mut store, AllIntegers::I16(i16::MAX))?,
        AllIntegers::I16(i16::MIN)
    ));
    assert!(matches!(
        exports.add_one_integer(&mut store, AllIntegers::I32(0))?,
        AllIntegers::I32(1)
    ));
    assert!(matches!(
        exports.add_one_integer(&mut store, AllIntegers::I32(i32::MAX))?,
        AllIntegers::I32(i32::MIN)
    ));
    assert!(matches!(
        exports.add_one_integer(&mut store, AllIntegers::I64(0))?,
        AllIntegers::I64(1)
    ));
    assert!(matches!(
        exports.add_one_integer(&mut store, AllIntegers::I64(i64::MAX))?,
        AllIntegers::I64(i64::MIN)
    ));

    // Floats
    match exports.add_one_float(&mut store, AllFloats::F32(0.0))? {
        AllFloats::F32(r) => assert_eq!(r, 1.0),
        _ => panic!(),
    }
    match exports.add_one_float(&mut store, AllFloats::F32(420.0))? {
        AllFloats::F32(r) => assert_eq!(r, 421.0),
        _ => panic!(),
    }
    match exports.add_one_float(&mut store, AllFloats::F64(0.0))? {
        AllFloats::F64(r) => assert_eq!(r, 1.0),
        _ => panic!(),
    }
    match exports.add_one_float(&mut store, AllFloats::F64(420.0))? {
        AllFloats::F64(r) => assert_eq!(r, 421.0),
        _ => panic!(),
    }

    // Text
    assert!(matches!(
        exports.replace_first_char(&mut store, AllTextParam::Char('a'), 'z')?,
        AllTextResult::Char('z')
    ));
    match exports.replace_first_char(&mut store, AllTextParam::String("abc"), 'z')? {
        AllTextResult::String(s) => assert_eq!(s, "zbc"),
        _ => panic!(),
    }

    // Identify Integers
    assert_eq!(
        exports.identify_integer(&mut store, AllIntegers::Bool(false))?,
        0
    );
    assert_eq!(exports.identify_integer(&mut store, AllIntegers::U8(0))?, 1);
    assert_eq!(
        exports.identify_integer(&mut store, AllIntegers::U16(0))?,
        2
    );
    assert_eq!(
        exports.identify_integer(&mut store, AllIntegers::U32(0))?,
        3
    );
    assert_eq!(
        exports.identify_integer(&mut store, AllIntegers::U64(0))?,
        4
    );
    assert_eq!(exports.identify_integer(&mut store, AllIntegers::I8(0))?, 5);
    assert_eq!(
        exports.identify_integer(&mut store, AllIntegers::I16(0))?,
        6
    );
    assert_eq!(
        exports.identify_integer(&mut store, AllIntegers::I32(0))?,
        7
    );
    assert_eq!(
        exports.identify_integer(&mut store, AllIntegers::I64(0))?,
        8
    );

    // Identify floats
    assert_eq!(exports.identify_float(&mut store, AllFloats::F32(0.0))?, 0);
    assert_eq!(exports.identify_float(&mut store, AllFloats::F64(0.0))?, 1);

    // Identify text
    assert_eq!(
        exports.identify_text(&mut store, AllTextParam::Char('\0'))?,
        0
    );
    assert_eq!(
        exports.identify_text(&mut store, AllTextParam::String(""))?,
        1
    );

    // Identify Duplicated
    assert_eq!(
        exports.identify_duplicated(&mut store, DuplicatedS32::I320(0))?,
        0
    );
    assert_eq!(
        exports.identify_duplicated(&mut store, DuplicatedS32::I321(0))?,
        1
    );
    assert_eq!(
        exports.identify_duplicated(&mut store, DuplicatedS32::I322(0))?,
        2
    );

    assert!(matches!(
        exports.add_one_duplicated(&mut store, DuplicatedS32::I320(0))?,
        DuplicatedS32::I320(1)
    ));
    assert!(matches!(
        exports.add_one_duplicated(&mut store, DuplicatedS32::I321(0))?,
        DuplicatedS32::I321(1)
    ));
    assert!(matches!(
        exports.add_one_duplicated(&mut store, DuplicatedS32::I322(0))?,
        DuplicatedS32::I322(1)
    ));

    // Identify Distinguishable Num
    assert_eq!(
        exports.identify_distinguishable_num(&mut store, DistinguishableNum::F64(0.0))?,
        0
    );
    assert_eq!(
        exports.identify_distinguishable_num(&mut store, DistinguishableNum::I64(0))?,
        1
    );

    match exports.add_one_distinguishable_num(&mut store, DistinguishableNum::F64(0.0))? {
        DistinguishableNum::F64(f) => assert_eq!(f, 1.0),
        _ => panic!(),
    };
    assert!(matches!(
        exports.add_one_distinguishable_num(&mut store, DistinguishableNum::I64(0))?,
        DistinguishableNum::I64(1),
    ));
    Ok(())
}
