use anyhow::{anyhow, bail, Context, Result};
use id_arena::{Arena, Id};
use pulldown_cmark::{CodeBlockKind, CowStr, Event, Options, Parser, Tag};
use std::borrow::Cow;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

pub mod abi;
mod ast;
mod sizealign;
pub use sizealign::*;

/// Checks if the given string is a legal identifier in wit.
pub fn validate_id(s: &str) -> Result<()> {
    ast::validate_id(0, s)?;
    Ok(())
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Interface {
    pub name: String,
    pub types: Arena<TypeDef>,
    pub type_lookup: HashMap<String, TypeId>,
    pub interfaces: Arena<Interface>,
    pub interface_lookup: HashMap<String, InterfaceId>,
    pub functions: Vec<Function>,
    pub globals: Vec<Global>,
}

pub type TypeId = Id<TypeDef>;
pub type InterfaceId = Id<Interface>;

#[derive(Debug, Clone, PartialEq)]
pub struct TypeDef {
    pub docs: Docs,
    pub kind: TypeDefKind,
    pub name: Option<String>,
    /// `None` if this type is originally declared in this instance or
    /// otherwise `Some` if it was originally defined in a different module.
    pub foreign_module: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TypeDefKind {
    Record(Record),
    Flags(Flags),
    Tuple(Tuple),
    Variant(Variant),
    Enum(Enum),
    Option(Type),
    Result(Result_),
    Union(Union),
    List(Type),
    Future(Option<Type>),
    Stream(Stream),
    Type(Type),
}

#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone)]
pub enum Type {
    Bool,
    U8,
    U16,
    U32,
    U64,
    S8,
    S16,
    S32,
    S64,
    Float32,
    Float64,
    Char,
    String,
    Id(TypeId),
}

#[derive(PartialEq, Debug, Copy, Clone)]
pub enum Int {
    U8,
    U16,
    U32,
    U64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Record {
    pub fields: Vec<Field>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Field {
    pub docs: Docs,
    pub name: String,
    pub ty: Type,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Flags {
    pub flags: Vec<Flag>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Flag {
    pub docs: Docs,
    pub name: String,
}

#[derive(Debug)]
pub enum FlagsRepr {
    U8,
    U16,
    U32(usize),
}

impl Flags {
    pub fn repr(&self) -> FlagsRepr {
        match self.flags.len() {
            n if n <= 8 => FlagsRepr::U8,
            n if n <= 16 => FlagsRepr::U16,
            n => FlagsRepr::U32(sizealign::align_to(n, 32) / 32),
        }
    }
}

impl FlagsRepr {
    pub fn count(&self) -> usize {
        match self {
            FlagsRepr::U8 => 1,
            FlagsRepr::U16 => 1,
            FlagsRepr::U32(n) => *n,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Tuple {
    pub types: Vec<Type>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Variant {
    pub cases: Vec<Case>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Case {
    pub docs: Docs,
    pub name: String,
    pub ty: Option<Type>,
}

impl Variant {
    pub fn tag(&self) -> Int {
        match self.cases.len() {
            n if n <= u8::max_value() as usize => Int::U8,
            n if n <= u16::max_value() as usize => Int::U16,
            n if n <= u32::max_value() as usize => Int::U32,
            _ => panic!("too many cases to fit in a repr"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Enum {
    pub cases: Vec<EnumCase>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EnumCase {
    pub docs: Docs,
    pub name: String,
}

impl Enum {
    pub fn tag(&self) -> Int {
        match self.cases.len() {
            n if n <= u8::max_value() as usize => Int::U8,
            n if n <= u16::max_value() as usize => Int::U16,
            n if n <= u32::max_value() as usize => Int::U32,
            _ => panic!("too many cases to fit in a repr"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Result_ {
    pub ok: Option<Type>,
    pub err: Option<Type>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Union {
    pub cases: Vec<UnionCase>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnionCase {
    pub docs: Docs,
    pub ty: Type,
}

impl Union {
    pub fn tag(&self) -> Int {
        match self.cases.len() {
            n if n <= u8::max_value() as usize => Int::U8,
            n if n <= u16::max_value() as usize => Int::U16,
            n if n <= u32::max_value() as usize => Int::U32,
            _ => panic!("too many cases to fit in a repr"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Stream {
    pub element: Option<Type>,
    pub end: Option<Type>,
}

#[derive(Clone, Default, Debug, PartialEq)]
pub struct Docs {
    pub contents: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Global {
    pub docs: Docs,
    pub name: String,
    pub ty: Type,
}

pub type Params = Vec<(String, Type)>;

#[derive(Debug, Clone, PartialEq)]
pub enum Results {
    Named(Params),
    Anon(Type),
}

pub enum ResultsTypeIter<'a> {
    Named(std::slice::Iter<'a, (String, Type)>),
    Anon(std::iter::Once<&'a Type>),
}

impl<'a> Iterator for ResultsTypeIter<'a> {
    type Item = &'a Type;

    fn next(&mut self) -> Option<&'a Type> {
        match self {
            ResultsTypeIter::Named(ps) => ps.next().map(|p| &p.1),
            ResultsTypeIter::Anon(ty) => ty.next(),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            ResultsTypeIter::Named(ps) => ps.size_hint(),
            ResultsTypeIter::Anon(ty) => ty.size_hint(),
        }
    }
}

impl<'a> ExactSizeIterator for ResultsTypeIter<'a> {}

impl Results {
    // For the common case of an empty results list.
    pub fn empty() -> Results {
        Results::Named(Vec::new())
    }

    pub fn len(&self) -> usize {
        match self {
            Results::Named(params) => params.len(),
            Results::Anon(_) => 1,
        }
    }

    pub fn iter_types(&self) -> ResultsTypeIter {
        match self {
            Results::Named(ps) => ResultsTypeIter::Named(ps.iter()),
            Results::Anon(ty) => ResultsTypeIter::Anon(std::iter::once(ty)),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Function {
    pub docs: Docs,
    pub name: String,
    pub kind: FunctionKind,
    pub params: Params,
    pub results: Results,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FunctionKind {
    Freestanding,
}

impl Function {
    pub fn item_name(&self) -> &str {
        match &self.kind {
            FunctionKind::Freestanding => &self.name,
        }
    }
}

fn unwrap_md(contents: &str) -> String {
    let mut wit = String::new();
    let mut last_pos = 0;
    let mut in_wit_code_block = false;
    Parser::new_ext(contents, Options::empty())
        .into_offset_iter()
        .for_each(|(event, range)| match (event, range) {
            (Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(CowStr::Borrowed("wit")))), _) => {
                in_wit_code_block = true;
            }
            (Event::Text(text), range) if in_wit_code_block => {
                // Ensure that offsets are correct by inserting newlines to
                // cover the Markdown content outside of wit code blocks.
                for _ in contents[last_pos..range.start].lines() {
                    wit.push_str("\n");
                }
                wit.push_str(&text);
                last_pos = range.end;
            }
            (Event::End(Tag::CodeBlock(CodeBlockKind::Fenced(CowStr::Borrowed("wit")))), _) => {
                in_wit_code_block = false;
            }
            _ => {}
        });
    wit
}

impl Interface {
    pub fn parse(name: &str, input: &str) -> Result<Interface> {
        Interface::parse_with(name, input, |f| {
            Err(anyhow!("cannot load submodule `{}`", f))
        })
    }

    pub fn parse_file(path: impl AsRef<Path>) -> Result<Interface> {
        let path = path.as_ref();
        let parent = path.parent().unwrap();
        let contents = std::fs::read_to_string(&path)
            .with_context(|| format!("failed to read: {}", path.display()))?;
        Interface::parse_with(path, &contents, |path| load_fs(parent, path))
    }

    pub fn parse_with(
        filename: impl AsRef<Path>,
        contents: &str,
        mut load: impl FnMut(&str) -> Result<(PathBuf, String)>,
    ) -> Result<Interface> {
        Interface::_parse_with(
            filename.as_ref(),
            contents,
            &mut load,
            &mut HashSet::new(),
            &mut HashMap::new(),
        )
    }

    fn _parse_with(
        filename: &Path,
        contents: &str,
        load: &mut dyn FnMut(&str) -> Result<(PathBuf, String)>,
        visiting: &mut HashSet<PathBuf>,
        map: &mut HashMap<String, Interface>,
    ) -> Result<Interface> {
        let name = filename
            .file_name()
            .context("wit path must end in a file name")?
            .to_str()
            .context("wit filename must be valid unicode")?
            // TODO: replace with `file_prefix` if/when that gets stabilized.
            .split(".")
            .next()
            .unwrap();
        let mut contents = contents;

        // If we have a ".md" file, it's a wit file wrapped in a markdown file;
        // parse the markdown to extract the `wit` code blocks.
        let md_contents;
        if filename.extension().and_then(|s| s.to_str()) == Some("md") {
            md_contents = unwrap_md(contents);
            contents = &md_contents[..];
        }

        // Parse the `contents `into an AST
        let ast = match ast::Ast::parse(contents) {
            Ok(ast) => ast,
            Err(mut e) => {
                let file = filename.display().to_string();
                ast::rewrite_error(&mut e, &file, contents);
                return Err(e);
            }
        };

        // Load up any modules into our `map` that have not yet been parsed.
        if !visiting.insert(filename.to_path_buf()) {
            bail!("file `{}` recursively imports itself", filename.display())
        }
        for item in ast.items.iter() {
            let u = match item {
                ast::Item::Use(u) => u,
                _ => continue,
            };
            if map.contains_key(&*u.from[0].name) {
                continue;
            }
            let (filename, contents) = load(&u.from[0].name)
                // TODO: insert context here about `u.name.span` and `filename`
                ?;
            let instance = Interface::_parse_with(&filename, &contents, load, visiting, map)?;
            map.insert(u.from[0].name.to_string(), instance);
        }
        visiting.remove(filename);

        // and finally resolve everything into our final instance
        match ast.resolve(name, map) {
            Ok(i) => Ok(i),
            Err(mut e) => {
                let file = filename.display().to_string();
                ast::rewrite_error(&mut e, &file, contents);
                Err(e)
            }
        }
    }

    /// Gets the core export name for the given function.
    pub fn core_export_name<'a>(&self, default_export: bool, func: &'a Function) -> Cow<'a, str> {
        if default_export || self.name.is_empty() {
            Cow::Borrowed(&func.name)
        } else {
            Cow::Owned(format!("{}#{}", self.name, func.name))
        }
    }

    pub fn topological_types(&self) -> Vec<TypeId> {
        let mut ret = Vec::new();
        let mut visited = HashSet::new();
        for (id, _) in self.types.iter() {
            self.topo_visit(id, &mut ret, &mut visited);
        }
        ret
    }

    fn topo_visit(&self, id: TypeId, list: &mut Vec<TypeId>, visited: &mut HashSet<TypeId>) {
        if !visited.insert(id) {
            return;
        }
        match &self.types[id].kind {
            TypeDefKind::Flags(_) | TypeDefKind::Enum(_) => {}
            TypeDefKind::Type(t) | TypeDefKind::List(t) => self.topo_visit_ty(t, list, visited),
            TypeDefKind::Record(r) => {
                for f in r.fields.iter() {
                    self.topo_visit_ty(&f.ty, list, visited);
                }
            }
            TypeDefKind::Tuple(t) => {
                for t in t.types.iter() {
                    self.topo_visit_ty(t, list, visited);
                }
            }
            TypeDefKind::Variant(v) => {
                for v in v.cases.iter() {
                    if let Some(ty) = v.ty {
                        self.topo_visit_ty(&ty, list, visited);
                    }
                }
            }
            TypeDefKind::Option(ty) => self.topo_visit_ty(ty, list, visited),
            TypeDefKind::Result(r) => {
                if let Some(ok) = r.ok {
                    self.topo_visit_ty(&ok, list, visited);
                }
                if let Some(err) = r.err {
                    self.topo_visit_ty(&err, list, visited);
                }
            }
            TypeDefKind::Union(u) => {
                for t in u.cases.iter() {
                    self.topo_visit_ty(&t.ty, list, visited);
                }
            }
            TypeDefKind::Future(ty) => {
                if let Some(ty) = ty {
                    self.topo_visit_ty(ty, list, visited);
                }
            }
            TypeDefKind::Stream(s) => {
                if let Some(element) = s.element {
                    self.topo_visit_ty(&element, list, visited);
                }
                if let Some(end) = s.end {
                    self.topo_visit_ty(&end, list, visited);
                }
            }
        }
        list.push(id);
    }

    fn topo_visit_ty(&self, ty: &Type, list: &mut Vec<TypeId>, visited: &mut HashSet<TypeId>) {
        if let Type::Id(id) = ty {
            self.topo_visit(*id, list, visited);
        }
    }

    pub fn all_bits_valid(&self, ty: &Type) -> bool {
        match ty {
            Type::U8
            | Type::S8
            | Type::U16
            | Type::S16
            | Type::U32
            | Type::S32
            | Type::U64
            | Type::S64
            | Type::Float32
            | Type::Float64 => true,

            Type::Bool | Type::Char | Type::String => false,

            Type::Id(id) => match &self.types[*id].kind {
                TypeDefKind::List(_)
                | TypeDefKind::Variant(_)
                | TypeDefKind::Enum(_)
                | TypeDefKind::Option(_)
                | TypeDefKind::Result(_)
                | TypeDefKind::Future(_)
                | TypeDefKind::Stream(_)
                | TypeDefKind::Union(_) => false,
                TypeDefKind::Type(t) => self.all_bits_valid(t),
                TypeDefKind::Record(r) => r.fields.iter().all(|f| self.all_bits_valid(&f.ty)),
                TypeDefKind::Tuple(t) => t.types.iter().all(|t| self.all_bits_valid(t)),

                // FIXME: this could perhaps be `true` for multiples-of-32 but
                // seems better to probably leave this as unconditionally
                // `false` for now, may want to reconsider later?
                TypeDefKind::Flags(_) => false,
            },
        }
    }

    pub fn get_variant(&self, ty: &Type) -> Option<&Variant> {
        if let Type::Id(id) = ty {
            match &self.types[*id].kind {
                TypeDefKind::Variant(v) => Some(v),
                _ => None,
            }
        } else {
            None
        }
    }
}

fn load_fs(root: &Path, name: &str) -> Result<(PathBuf, String)> {
    let wit = root.join(name).with_extension("wit");

    // Attempt to read a ".wit" file.
    match fs::read_to_string(&wit) {
        Ok(contents) => Ok((wit, contents)),

        // If no such file was found, attempt to read a ".wit.md" file.
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
            let wit_md = wit.with_extension("wit.md");
            match fs::read_to_string(&wit_md) {
                Ok(contents) => Ok((wit_md, contents)),
                Err(_err) => Err(err.into()),
            }
        }

        Err(err) => return Err(err.into()),
    }
}
