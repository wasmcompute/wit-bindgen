use anyhow::{Context, Result};
use pretty_assertions::assert_eq;
use std::fs;
use std::path::{Path, PathBuf};
use wit_component::ComponentEncoder;
use wit_parser::Interface;

/// Tests the encoding of the "types only" mode of `wit-component`.
///
/// This test looks in the `interfaces/` directory for test cases in a similar
/// format to those in the `components/` where there's a bunch of:
///
/// * `import-*.wit`
/// * `export-*.wit`
/// * `default.wit`
///
/// Where these represent the "world" of a component. Eventually this should
/// probably become just one file. For now though this suffices and plumbs
/// through the respective arguments of `wit-component`. The `*.wit` files are
/// encoded in "types only" mode and verified against `types_only.wat` and then
/// additionally encoded in normal mode with a dummy module to verify that works
/// as well.
///
/// Run the test with the environment variable `BLESS` set to update
/// the wat baseline file.
#[test]
fn interface_encoding() -> Result<()> {
    for entry in fs::read_dir("tests/interfaces")? {
        let path = entry?.path();
        if !path.is_dir() {
            continue;
        }
        run_test(&path).context(format!("failed test `{}`", path.display()))?;
    }

    Ok(())
}

fn run_test(path: &Path) -> Result<()> {
    let test_case = path.file_stem().unwrap().to_str().unwrap();
    println!("test {test_case}");
    let interface_path = path.join("default.wit");

    let mut default = None;
    if interface_path.is_file() {
        default = Some(read_interface(&interface_path)?);
    }

    let mut import_interfaces = Vec::new();
    let mut import_wits = Vec::new();
    let mut export_interfaces = Vec::new();
    let mut export_wits = Vec::new();
    for (path, import) in read_interfaces(&path, "import-*.wit")? {
        import_wits.push((import.name.clone(), path));
        import_interfaces.push(import);
    }
    for (path, export) in read_interfaces(&path, "export-*.wit")? {
        export_wits.push((export.name.clone(), path));
        export_interfaces.push(export);
    }

    let assert_output = |wasm: &[u8], wat: &Path| -> Result<()> {
        let output = wasmprinter::print_bytes(wasm)?;

        if std::env::var_os("BLESS").is_some() {
            fs::write(wat, output)?;
        } else {
            assert_eq!(
                fs::read_to_string(wat)?.replace("\r\n", "\n"),
                output,
                "encoding of `{test_case}` did not match the expected wat file `{}`",
                wat.display(),
            );
        }

        let interfaces = wit_component::decode_component_interfaces(wasm)
            .context(format!("failed to decode bytes for test `{test_case}`"))?;

        if test_case == "empty" {
            return Ok(());
        }

        assert_eq!(interfaces.imports.len(), import_wits.len());
        assert_eq!(interfaces.exports.len(), export_wits.len());
        assert_eq!(interfaces.default.is_some(), default.is_some());

        for (name, path) in import_wits.iter() {
            assert_wit(path, &interfaces.imports[name.as_str()])
                .context(format!("failed to assert wit import `{name}`"))?;
        }
        Ok(())
    };

    // Test a types-only component. This ensures that in "types only" mode we
    // can recover all the original `*.wit` interfaces from the generated
    // artifact.

    println!("testing types only");
    let mut encoder = ComponentEncoder::default()
        .types_only(true)
        .validate(true)
        .imports(import_interfaces.clone())?
        .exports(export_interfaces.clone())?;
    if let Some(default) = &default {
        encoder = encoder.interface(default.clone())?;
    }
    let bytes = encoder.encode().with_context(|| {
        format!("failed to encode a types-only component for test case `{test_case}`")
    })?;
    assert_output(&bytes, &path.join("types_only.wat"))?;

    // Test a full component with a dummy module as the implementation. This
    // tests a different path through `wit-component` to ensure that we can
    // recover the original `*.wit` interfaces from the component output.

    println!("test dummy module");
    let module =
        test_helpers::dummy_module(&import_interfaces, &export_interfaces, default.as_ref());
    let mut encoder = ComponentEncoder::default()
        .module(&module)?
        .validate(true)
        .imports(import_interfaces.clone())?
        .exports(export_interfaces.clone())?;
    if let Some(default) = &default {
        encoder = encoder.interface(default.clone())?;
    }
    let bytes = encoder
        .encode()
        .with_context(|| format!("failed to encode a component for test case `{test_case}`"))?;
    assert_output(&bytes, &path.join("component.wat"))?;

    Ok(())
}

fn assert_wit(wit_path: &Path, interface: &Interface) -> Result<()> {
    let mut printer = wit_component::InterfacePrinter::default();
    let output = printer
        .print(&interface)
        .context("failed to print interface")?;

    if std::env::var_os("BLESS").is_some() {
        fs::write(&wit_path, output)?;
    } else {
        assert_eq!(
            fs::read_to_string(&wit_path)?.replace("\r\n", "\n"),
            output,
            "encoding of wit file `{}` did not match the the decoded interface",
            wit_path.display(),
        );
    }
    Ok(())
}

fn read_interface(path: &Path) -> Result<Interface> {
    Interface::parse_file(&path)
        .with_context(|| format!("failed to parse interface file `{}`", path.display()))
}

fn read_interfaces(dir: &Path, pattern: &str) -> Result<Vec<(PathBuf, Interface)>> {
    glob::glob(dir.join(pattern).to_str().unwrap())?
        .map(|p| {
            let p = p?;
            let mut i = read_interface(&p)?;
            i.name = p
                .file_stem()
                .unwrap()
                .to_str()
                .unwrap()
                .trim_start_matches("import-")
                .trim_start_matches("export-")
                .to_string();
            Ok((p, i))
        })
        .collect::<Result<_>>()
}
