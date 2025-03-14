use anyhow::{anyhow, Context, Result};
use indexmap::IndexMap;
use std::hash::{Hash, Hasher};
use wasmparser::{
    types, ComponentExport, ComponentTypeRef, Parser, Payload, PrimitiveValType, ValidPayload,
    Validator, WasmFeatures,
};
use wit_parser::*;

/// Represents information about a decoded WebAssembly component.
struct ComponentInfo<'a> {
    /// Wasmparser-defined type information learned after a component is fully
    /// validated.
    types: types::Types,
    /// Map of imports and what type they're importing.
    imports: IndexMap<&'a str, ComponentTypeRef>,
    /// Map of exports and what they're exporting.
    exports: IndexMap<&'a str, ComponentExport<'a>>,
}

impl<'a> ComponentInfo<'a> {
    /// Creates a new component info by parsing the given WebAssembly component bytes.
    fn new(bytes: &'a [u8]) -> Result<Self> {
        let mut validator = Validator::new_with_features(WasmFeatures {
            component_model: true,
            ..Default::default()
        });
        let mut exports = IndexMap::new();
        let mut imports = IndexMap::new();
        let mut depth = 1;
        let mut types = None;

        for payload in Parser::new(0).parse_all(bytes) {
            let payload = payload?;
            match validator.payload(&payload)? {
                ValidPayload::Ok => {}
                ValidPayload::Parser(_) => depth += 1,
                ValidPayload::End(t) => {
                    depth -= 1;
                    if depth == 0 {
                        types = Some(t);
                    }
                }
                ValidPayload::Func(..) => {}
            }

            match payload {
                Payload::ComponentImportSection(s) if depth == 1 => {
                    for import in s {
                        let import = import?;
                        let prev = imports.insert(import.name, import.ty);
                        assert!(prev.is_none());
                    }
                }
                Payload::ComponentExportSection(s) if depth == 1 => {
                    for export in s {
                        let export = export?;
                        let prev = exports.insert(export.name, export);
                        assert!(prev.is_none());
                    }
                }
                _ => {}
            }
        }
        Ok(Self {
            types: types.unwrap(),
            imports,
            exports,
        })
    }
}

/// Represents an interface decoder for WebAssembly components.
struct InterfaceDecoder<'a> {
    info: &'a ComponentInfo<'a>,
    interface: Interface,

    // Note that the hash keys in these maps are `&types::Type` where we're
    // hashing the memory address of the pointer itself. The purpose here is to
    // ensure that two `TypeId` entries which come from two different index
    // spaces which point to the same type can name the same type, so the hash
    // key is the result after `TypeId` lookup.
    type_map: IndexMap<PtrHash<'a, types::Type>, Type>,
    name_map: IndexMap<PtrHash<'a, types::Type>, &'a str>,
}

/// Parsed representation of interfaces found within a component.
///
/// This is more-or-less a "world" and will likely be replaced one day with a
/// `wit-parser` representation of a world.
#[derive(Default)]
pub struct ComponentInterfaces {
    /// The "default export" which is the interface directly exported from the
    /// component at the top level.
    pub default: Option<Interface>,
    /// Imported interfaces, keyed by name, of the component.
    pub imports: IndexMap<String, Interface>,
    /// Exported interfaces, keyed by name, of the component.
    pub exports: IndexMap<String, Interface>,
}

/// Decode the interfaces imported and exported by a component.
///
/// This function takes a binary component as input and will infer the
/// `Interface` representation of its imports and exports. More-or-less this
/// will infer the "world" from a binary component. The binary component at this
/// time is either a "types only" component produced by `wit-component` or an
/// actual output of `wit-component`.
///
/// The returned interfaces represent the description of imports and exports
/// from the component.
///
/// This can fail if the input component is invalid or otherwise isn't of the
/// expected shape. At this time not all component shapes are supported here.
pub fn decode_component_interfaces(bytes: &[u8]) -> Result<ComponentInterfaces> {
    let info = ComponentInfo::new(bytes)?;
    let mut imports = IndexMap::new();
    let mut exports = IndexMap::new();

    for (name, ty) in info.imports.iter() {
        // Imports right now are only supported if they're an import of an
        // instance. The instance is expected to export only functions and types
        // where types are named types used in functions.
        let ty = match *ty {
            ComponentTypeRef::Instance(i) => match info.types.type_at(i, false).unwrap() {
                types::Type::ComponentInstance(i) => i,
                _ => unreachable!(),
            },
            _ => unimplemented!(),
        };
        let mut iface = InterfaceDecoder::new(&info).decode(ty.exports(info.types.as_ref()))?;
        iface.name = name.to_string();
        imports.insert(iface.name.clone(), iface);
    }

    let mut default = IndexMap::new();
    for (name, export) in info.exports.iter() {
        // Get a `ComponentEntityType` which describes the type of the item
        // being exported here. If a type itself is being exported then "peel"
        // it to feign an actual entity being exported here to handle both
        // type-only and normal components produced by `wit-component`.
        let mut ty = info
            .types
            .component_entity_type_from_export(export)
            .unwrap();
        if let types::ComponentEntityType::Type(id) = ty {
            match info.types.type_from_id(id).unwrap() {
                types::Type::ComponentInstance(_) => ty = types::ComponentEntityType::Instance(id),
                types::Type::ComponentFunc(_) => ty = types::ComponentEntityType::Func(id),
                _ => {}
            }
        }

        match ty {
            // If an instance is being exported then that means this is an
            // interface being exported, so decode the interface here and
            // register an export.
            types::ComponentEntityType::Instance(ty) => {
                let ty = info
                    .types
                    .type_from_id(ty)
                    .unwrap()
                    .as_component_instance_type()
                    .unwrap();
                let mut iface =
                    InterfaceDecoder::new(&info).decode(ty.exports(info.types.as_ref()))?;
                iface.name = name.to_string();
                exports.insert(iface.name.clone(), iface);
            }

            // Otherwise assume everything else is part of the "default" export.
            ty => {
                default.insert(name.to_string(), ty);
            }
        }
    }

    let default = if default.is_empty() {
        None
    } else {
        Some(InterfaceDecoder::new(&info).decode(&default)?)
    };

    Ok(ComponentInterfaces {
        imports,
        exports,
        default,
    })
}

impl<'a> InterfaceDecoder<'a> {
    /// Creates a new interface decoder for the given component information.
    fn new(info: &'a ComponentInfo<'a>) -> InterfaceDecoder<'a> {
        Self {
            info,
            interface: Interface::default(),
            name_map: IndexMap::new(),
            type_map: IndexMap::new(),
        }
    }

    /// Consumes the decoder and returns the interface representation assuming
    /// that the interface is made of the specified exports.
    pub fn decode(
        mut self,
        map: &'a IndexMap<String, types::ComponentEntityType>,
    ) -> Result<Interface> {
        let mut aliases = Vec::new();
        // Populate names in the name map first
        for (name, ty) in map {
            let id = match ty {
                types::ComponentEntityType::Type(id) => *id,
                _ => continue,
            };

            let ty = self.info.types.type_from_id(id).unwrap();
            let key = PtrHash(ty);
            if self.name_map.contains_key(&key) {
                aliases.push((name, key));
            } else {
                let prev = self.name_map.insert(PtrHash(ty), name);
                assert!(prev.is_none());
            }
        }

        // Iterate over all exports an interpret them as defined items within
        // the interface, either functions or types at this time.
        for (name, ty) in map {
            match ty {
                types::ComponentEntityType::Func(ty) => {
                    match self.info.types.type_from_id(*ty).unwrap() {
                        types::Type::ComponentFunc(ty) => {
                            self.add_function(name, ty)?;
                        }
                        _ => unimplemented!(),
                    }
                }
                types::ComponentEntityType::Type(id) => {
                    assert!(matches!(
                        self.info.types.type_from_id(*id).unwrap(),
                        types::Type::Defined(_)
                    ));
                    self.decode_type(&types::ComponentValType::Type(*id))?;
                }
                _ => unimplemented!(),
            }
        }

        for (name, key) in aliases {
            let ty = self.type_map[&key];
            self.interface.types.alloc(TypeDef {
                docs: Default::default(),
                kind: TypeDefKind::Type(ty),
                name: Some(name.to_string()),
                foreign_module: None,
            });
        }

        Ok(self.interface)
    }

    fn decode_params(
        &mut self,
        func_name: &str,
        ps: &[(String, types::ComponentValType)],
    ) -> Result<Params> {
        let mut params = Vec::new();
        for (name, ty) in ps.iter() {
            validate_id(name).with_context(|| {
                format!(
                    "function `{}` has a parameter `{}` that is not a valid identifier",
                    func_name, name
                )
            })?;

            params.push((name.clone(), self.decode_type(ty)?));
        }
        Ok(params)
    }

    fn decode_results(
        &mut self,
        func_name: &str,
        ps: &[(Option<String>, types::ComponentValType)],
    ) -> Result<Results> {
        let mut results = Vec::new();
        for (name, ty) in ps.iter() {
            let name = match name {
                Some(name) => {
                    let name = name.to_string();
                    validate_id(&name).with_context(|| {
                        format!(
                            "function `{}` has a result type `{}` that is not a valid identifier",
                            func_name, name
                        )
                    })?;
                    Some(name)
                }
                None => None,
            };

            results.push((name, self.decode_type(ty)?));
        }

        // Results must be either
        // - A single anonymous type
        // - Any number of named types
        match results.len() {
            1 => {
                // We either have a single anonymous type or a single
                // named type. Either is valid.
                let (name, ty) = results.into_iter().next().unwrap();
                match name {
                    Some(name) => Ok(Results::Named(vec![(name, ty)])),
                    None => Ok(Results::Anon(ty)),
                }
            }
            _ => {
                // Otherwise, all types must be named.
                let mut rs = Vec::new();
                for (name, ty) in results.into_iter() {
                    match name {
                        Some(name) => rs.push((name, ty)),
                        None => {
                            return Err(anyhow!(
                                "function `{}` is missing a result type name",
                                func_name
                            ))
                        }
                    }
                }
                Ok(Results::Named(rs))
            }
        }
    }

    fn add_function(&mut self, func_name: &str, ty: &types::ComponentFuncType) -> Result<()> {
        validate_id(func_name)
            .with_context(|| format!("function name `{}` is not a valid identifier", func_name))?;

        let params = self.decode_params(func_name, &ty.params)?;
        let results = self.decode_results(func_name, &ty.results)?;

        self.interface.functions.push(Function {
            docs: Docs::default(),
            name: func_name.to_string(),
            kind: FunctionKind::Freestanding,
            params,
            results,
        });

        Ok(())
    }

    fn decode_type(&mut self, ty: &types::ComponentValType) -> Result<Type> {
        Ok(match ty {
            types::ComponentValType::Primitive(ty) => self.decode_primitive(*ty)?,
            types::ComponentValType::Type(id) => {
                let ty = self.info.types.type_from_id(*id).unwrap();
                let key = PtrHash(ty);
                if let Some(ty) = self.type_map.get(&key) {
                    return Ok(*ty);
                }

                let name = self.name_map.get(&key).map(ToString::to_string);

                if let Some(name) = name.as_deref() {
                    validate_id(name).with_context(|| {
                        format!("type name `{}` is not a valid identifier", name)
                    })?;
                }

                let ty = match ty {
                    types::Type::Defined(ty) => match ty {
                        types::ComponentDefinedType::Primitive(ty) => {
                            self.decode_named_primitive(name, ty)?
                        }
                        types::ComponentDefinedType::Record(r) => {
                            self.decode_record(name, r.fields.iter())?
                        }
                        types::ComponentDefinedType::Variant(v) => {
                            self.decode_variant(name, v.cases.iter())?
                        }
                        types::ComponentDefinedType::List(ty) => {
                            let inner = self.decode_type(ty)?;
                            Type::Id(self.alloc_type(name, TypeDefKind::List(inner)))
                        }
                        types::ComponentDefinedType::Tuple(t) => {
                            self.decode_tuple(name, &t.types)?
                        }
                        types::ComponentDefinedType::Flags(names) => {
                            self.decode_flags(name, names.iter())?
                        }
                        types::ComponentDefinedType::Enum(names) => {
                            self.decode_enum(name, names.iter())?
                        }
                        types::ComponentDefinedType::Union(u) => {
                            self.decode_union(name, &u.types)?
                        }
                        types::ComponentDefinedType::Option(ty) => self.decode_option(name, ty)?,
                        types::ComponentDefinedType::Result { ok, err } => {
                            self.decode_result(name, ok.as_ref(), err.as_ref())?
                        }
                    },
                    _ => unreachable!(),
                };

                let prev = self.type_map.insert(key, ty);
                assert!(prev.is_none());
                ty
            }
        })
    }

    fn decode_optional_type(
        &mut self,
        ty: Option<&types::ComponentValType>,
    ) -> Result<Option<Type>> {
        match ty {
            Some(ty) => self.decode_type(ty).map(Some),
            None => Ok(None),
        }
    }

    fn decode_named_primitive(
        &mut self,
        name: Option<String>,
        ty: &PrimitiveValType,
    ) -> Result<Type> {
        let mut ty = self.decode_primitive(*ty)?;
        if let Some(name) = name {
            validate_id(&name)
                .with_context(|| format!("type name `{}` is not a valid identifier", name))?;

            ty = Type::Id(self.alloc_type(Some(name), TypeDefKind::Type(ty)));
        }

        Ok(ty)
    }

    fn decode_primitive(&mut self, ty: PrimitiveValType) -> Result<Type> {
        Ok(match ty {
            PrimitiveValType::Bool => Type::Bool,
            PrimitiveValType::S8 => Type::S8,
            PrimitiveValType::U8 => Type::U8,
            PrimitiveValType::S16 => Type::S16,
            PrimitiveValType::U16 => Type::U16,
            PrimitiveValType::S32 => Type::S32,
            PrimitiveValType::U32 => Type::U32,
            PrimitiveValType::S64 => Type::S64,
            PrimitiveValType::U64 => Type::U64,
            PrimitiveValType::Float32 => Type::Float32,
            PrimitiveValType::Float64 => Type::Float64,
            PrimitiveValType::Char => Type::Char,
            PrimitiveValType::String => Type::String,
        })
    }

    fn decode_record(
        &mut self,
        record_name: Option<String>,
        fields: impl ExactSizeIterator<Item = (&'a String, &'a types::ComponentValType)>,
    ) -> Result<Type> {
        let record_name =
            record_name.ok_or_else(|| anyhow!("interface has an unnamed record type"))?;

        let record = Record {
            fields: fields
                .map(|(name, ty)| {
                    validate_id(name).with_context(|| {
                        format!(
                            "record `{}` has a field `{}` that is not a valid identifier",
                            record_name, name
                        )
                    })?;

                    Ok(Field {
                        docs: Docs::default(),
                        name: name.to_string(),
                        ty: self.decode_type(ty)?,
                    })
                })
                .collect::<Result<_>>()?,
        };

        Ok(Type::Id(self.alloc_type(
            Some(record_name),
            TypeDefKind::Record(record),
        )))
    }

    fn decode_variant(
        &mut self,
        variant_name: Option<String>,
        cases: impl ExactSizeIterator<Item = (&'a String, &'a types::VariantCase)>,
    ) -> Result<Type> {
        let variant_name =
            variant_name.ok_or_else(|| anyhow!("interface has an unnamed variant type"))?;

        let variant = Variant {
            cases: cases
                .map(|(name, case)| {
                    validate_id(name).with_context(|| {
                        format!(
                            "variant `{}` has a case `{}` that is not a valid identifier",
                            variant_name, name
                        )
                    })?;

                    Ok(Case {
                        docs: Docs::default(),
                        name: name.to_string(),
                        ty: self.decode_optional_type(case.ty.as_ref())?,
                    })
                })
                .collect::<Result<_>>()?,
        };

        Ok(Type::Id(self.alloc_type(
            Some(variant_name),
            TypeDefKind::Variant(variant),
        )))
    }

    fn decode_tuple(
        &mut self,
        name: Option<String>,
        tys: &[types::ComponentValType],
    ) -> Result<Type> {
        let tuple = Tuple {
            types: tys
                .iter()
                .map(|ty| self.decode_type(ty))
                .collect::<Result<_>>()?,
        };

        Ok(Type::Id(self.alloc_type(name, TypeDefKind::Tuple(tuple))))
    }

    fn decode_flags(
        &mut self,
        flags_name: Option<String>,
        names: impl ExactSizeIterator<Item = &'a String>,
    ) -> Result<Type> {
        let flags_name =
            flags_name.ok_or_else(|| anyhow!("interface has an unnamed flags type"))?;

        let flags = Flags {
            flags: names
                .map(|name| {
                    validate_id(name).with_context(|| {
                        format!(
                            "flags `{}` has a flag named `{}` that is not a valid identifier",
                            flags_name, name
                        )
                    })?;

                    Ok(Flag {
                        docs: Docs::default(),
                        name: name.clone(),
                    })
                })
                .collect::<Result<_>>()?,
        };

        Ok(Type::Id(
            self.alloc_type(Some(flags_name), TypeDefKind::Flags(flags)),
        ))
    }

    fn decode_enum(
        &mut self,
        enum_name: Option<String>,
        names: impl ExactSizeIterator<Item = &'a String>,
    ) -> Result<Type> {
        let enum_name = enum_name.ok_or_else(|| anyhow!("interface has an unnamed enum type"))?;
        let enum_ = Enum {
            cases: names
                .map(|name| {
                    validate_id(name).with_context(|| {
                        format!(
                            "enum `{}` has a value `{}` that is not a valid identifier",
                            enum_name, name
                        )
                    })?;

                    Ok(EnumCase {
                        docs: Docs::default(),
                        name: name.to_string(),
                    })
                })
                .collect::<Result<_>>()?,
        };

        Ok(Type::Id(
            self.alloc_type(Some(enum_name), TypeDefKind::Enum(enum_)),
        ))
    }

    fn decode_union(
        &mut self,
        name: Option<String>,
        tys: &[types::ComponentValType],
    ) -> Result<Type> {
        let union = Union {
            cases: tys
                .iter()
                .map(|ty| {
                    Ok(UnionCase {
                        docs: Docs::default(),
                        ty: self.decode_type(ty)?,
                    })
                })
                .collect::<Result<_>>()?,
        };

        Ok(Type::Id(self.alloc_type(name, TypeDefKind::Union(union))))
    }

    fn decode_option(
        &mut self,
        name: Option<String>,
        payload: &types::ComponentValType,
    ) -> Result<Type> {
        let payload = self.decode_type(payload)?;
        Ok(Type::Id(
            self.alloc_type(name, TypeDefKind::Option(payload)),
        ))
    }

    fn decode_result(
        &mut self,
        name: Option<String>,
        ok: Option<&types::ComponentValType>,
        err: Option<&types::ComponentValType>,
    ) -> Result<Type> {
        let ok = self.decode_optional_type(ok)?;
        let err = self.decode_optional_type(err)?;
        Ok(Type::Id(self.alloc_type(
            name,
            TypeDefKind::Result(Result_ { ok, err }),
        )))
    }

    fn alloc_type(&mut self, name: Option<String>, kind: TypeDefKind) -> TypeId {
        self.interface.types.alloc(TypeDef {
            docs: Docs::default(),
            kind,
            name,
            foreign_module: None,
        })
    }
}

struct PtrHash<'a, T>(&'a T);

impl<T> PartialEq for PtrHash<'_, T> {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self.0, other.0)
    }
}

impl<T> Eq for PtrHash<'_, T> {}

impl<T> Hash for PtrHash<'_, T> {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        std::ptr::hash(self.0, hasher)
    }
}
