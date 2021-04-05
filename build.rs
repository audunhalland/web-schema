// build.rs

use std::env;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

#[path = "src/static_unicase.rs"]
mod static_unicase;

#[path = "src/attr/attr_type.rs"]
mod attr_type;

#[path = "src/defs/html5_defs.rs"]
mod html5_defs;

use static_unicase::StaticUniCase;

fn main() {
    codegen().unwrap();
}

struct NamespaceDesc {
    name: &'static str,
    path: &'static str,
}

struct TagDef {
    web_ns: &'static str,
    static_id: usize,
    pub_const_ident: String,
    tag: &'static str,
    is_void: bool,
}

struct AttributeDef {
    web_ns: &'static str,
    static_id: usize,
    pub_const_ident: String,
    attr: &'static str,
    prop: &'static str,
    flags: u32,
}

fn codegen() -> std::io::Result<()> {
    let out_dir = env::var("OUT_DIR").unwrap();

    let _tag_defs = tag_defs();
    let attribute_defs = attribute_defs();

    codegen_static_attribute_symbols(
        &attribute_defs,
        Path::new(&out_dir).join("codegen_attr_symbols.rs"),
    )?;

    codegen_static_web_attr_ns_lookup_tables(
        &attribute_defs,
        NamespaceDesc {
            name: "HTML5",
            path: "crate::html5",
        },
        Path::new(&out_dir).join("codegen_static_html_attrs.rs"),
    )?;

    Ok(())
}

fn tag_defs() -> Vec<TagDef> {
    let mut defs = vec![];

    for (tag, is_void) in html5_defs::tags::DEFS {
        defs.push(TagDef {
            web_ns: "HTML5",
            static_id: defs.len(),
            pub_const_ident: format!("{}", tag.replace('-', "_").to_uppercase()),
            tag,
            is_void: is_void.0,
        });
    }

    defs
}

fn attribute_defs() -> Vec<AttributeDef> {
    let mut defs = vec![];

    for (attr, prop, flags) in html5_defs::attrs::DEFS {
        defs.push(AttributeDef {
            web_ns: "HTML5",
            static_id: defs.len(),
            pub_const_ident: format!("{}", attr.replace('-', "_").to_uppercase()),
            attr,
            prop,
            flags: *flags,
        });
    }

    defs
}

fn codegen_static_attribute_symbols(
    defs: &[AttributeDef],
    out_path: std::path::PathBuf,
) -> std::io::Result<()> {
    let mut f = BufWriter::new(File::create(&out_path)?);

    writeln!(&mut f, "use crate::WebNS;")?;
    writeln!(
        &mut f,
        "use crate::static_web_attr::{{StaticWebAttr, StaticWebAttrSymbolNamespace}};"
    )?;
    writeln!(&mut f, "use crate::attr::attr_type::*;")?;
    writeln!(&mut f)?;

    // Symbol definition array:
    {
        writeln!(
            &mut f,
            "pub(crate) const __WEB_ATTRS: [StaticWebAttr; {len}] = [",
            len = defs.len()
        )?;

        for def in defs.iter() {
            writeln!(
                &mut f,
                r#"    StaticWebAttr {{ web_ns: WebNS::{web_ns}, name: "{attr}", property: "{prop}", attr_type: AttrType({flags}) }},"#,
                web_ns = def.web_ns,
                attr = def.attr,
                prop = def.prop,
                flags = def.flags
            )?;
        }

        writeln!(&mut f, "];\n",)?;
    }

    // Symbol namespace for all known attributes:
    {
        writeln!(
            &mut f,
            r#"
pub(crate) const __ATTR_SYMBOL_NS: StaticWebAttrSymbolNamespace = StaticWebAttrSymbolNamespace {{
    web_attrs: &__WEB_ATTRS,
}};"#,
        )?;
    }

    Ok(())
}

fn codegen_static_web_attr_ns_lookup_tables(
    defs: &[AttributeDef],
    ns_desc: NamespaceDesc,
    out_path: std::path::PathBuf,
) -> std::io::Result<()> {
    let mut f = BufWriter::new(File::create(&out_path)?);

    let defs: Vec<_> = defs
        .iter()
        .filter(|def| def.web_ns == ns_desc.name)
        .collect();

    writeln!(&mut f, "use dyn_symbol::Symbol;")?;
    writeln!(
        &mut f,
        "use crate::static_web_attr::{{StaticWebAttrLookupTables}};"
    )?;
    writeln!(&mut f, "use crate::static_unicase::*;")?;
    writeln!(&mut f, "use crate::symbols::*;")?;
    writeln!(&mut f)?;

    // Attribute class:
    {
        writeln!(
            &mut f,
            r#"
pub(crate) const __ATTR_LOOKUP_TABLES: StaticWebAttrLookupTables = StaticWebAttrLookupTables {{
    static_symbol_ns: &__ATTR_SYMBOL_NS,"#,
        )?;

        // Attribute name map:
        {
            let def_keys: Vec<_> = defs
                .iter()
                .map(|def| {
                    (
                        def,
                        PhfKeyRef {
                            key: StaticUniCase::new(def.attr),
                            ref_expr: format!(
                                "StaticUniCase::new(__WEB_ATTRS[{}].name)",
                                def.static_id
                            ),
                        },
                    )
                })
                .collect();

            let mut map_codegen: phf_codegen::Map<PhfKeyRef<StaticUniCase>> =
                phf_codegen::Map::new();
            for (def, key) in def_keys {
                map_codegen.entry(key, &format!("{}", def.static_id));
            }

            writeln!(
                &mut f,
                "    attribute_unicase_map: {},",
                map_codegen.build()
            )?;
        }

        // Prop name map:
        {
            let def_keys: Vec<_> = defs
                .iter()
                .map(|def| {
                    (
                        def,
                        PhfKeyRef {
                            key: def.prop,
                            ref_expr: format!("__WEB_ATTRS[{}].property", def.static_id),
                        },
                    )
                })
                .collect();

            let mut map_codegen: phf_codegen::Map<PhfKeyRef<&'static str>> =
                phf_codegen::Map::new();
            for (def, key) in def_keys {
                map_codegen.entry(key, &format!("{}", def.static_id));
            }

            writeln!(&mut f, "    property_map: {},\n", map_codegen.build())?;
        }

        writeln!(&mut f, "}};\n",)?;
    }

    // Public interface:
    {
        for def in defs.iter() {
            writeln!(
                &mut f,
                r#"
/// The {ns_name} `{attr}` attribute
pub const {pub_const_ident}: Symbol = Symbol::Static(&__ATTR_SYMBOL_NS, {static_id});"#,
                ns_name = ns_desc.name,
                attr = def.attr,
                pub_const_ident = def.pub_const_ident,
                static_id = def.static_id,
            )?;
        }

        writeln!(&mut f, "",)?;
    }

    Ok(())
}

struct PhfKeyRef<T> {
    key: T,
    ref_expr: String,
}

impl<T: PartialEq<T>> PartialEq<PhfKeyRef<T>> for PhfKeyRef<T> {
    fn eq(&self, rhs: &PhfKeyRef<T>) -> bool {
        self.key.eq(&rhs.key)
    }
}
impl<T: Eq> Eq for PhfKeyRef<T> {}

impl<T: std::hash::Hash> std::hash::Hash for PhfKeyRef<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.key.hash(state);
    }
}

impl<T: phf_shared::PhfHash> phf_shared::PhfHash for PhfKeyRef<T> {
    fn phf_hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.key.phf_hash(state);
    }
}

impl<T> phf_shared::FmtConst for PhfKeyRef<T> {
    fn fmt_const(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.ref_expr)
    }
}
