use std::{collections::HashMap, fmt::Formatter};

pub fn schema<T: facet::Facet>() -> Schema {
    let (root_property, definitions, title) = analyze_shape(T::SHAPE);
    Schema {
        schema: "http://json-schema.org/draft-07/schema#",
        title,
        root_property,
        definitions,
    }
}

fn get_type_name(shape: &facet::Shape) -> String {
    struct MyTypeName(facet::TypeNameFn);
    impl std::fmt::Display for MyTypeName {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            self.0(f, facet::TypeNameOpts::none())
        }
    }
    format!("{}", MyTypeName(shape.vtable.type_name))
}

fn analyze_shape(shape: &facet::Shape) -> (Property, Definitions, String) {
    let mut definitions = Definitions::new();
    let property = match shape.def {
        facet::Def::Scalar(scalar_def) => match scalar_def.affinity {
            facet::ScalarAffinity::Number(number_affinity) => match number_affinity.bits {
                facet::NumberBits::Integer { bits, sign } => Property::Type {
                    r#type: TypeOrTypes::Type(Type::Integer),
                    format: Some(match (bits, sign) {
                        (8, facet::Signedness::Signed) => TypeFormat::Int8,
                        (16, facet::Signedness::Signed) => TypeFormat::Int16,
                        (32, facet::Signedness::Signed) => TypeFormat::Int32,
                        (64, facet::Signedness::Signed) => TypeFormat::Int64,
                        (128, facet::Signedness::Signed) => TypeFormat::Int128,
                        (_, facet::Signedness::Signed) => TypeFormat::Int,
                        (8, facet::Signedness::Unsigned) => TypeFormat::Uint8,
                        (16, facet::Signedness::Unsigned) => TypeFormat::Uint16,
                        (32, facet::Signedness::Unsigned) => TypeFormat::Uint32,
                        (64, facet::Signedness::Unsigned) => TypeFormat::Uint64,
                        (128, facet::Signedness::Unsigned) => TypeFormat::Uint128,
                        (_, facet::Signedness::Unsigned) => TypeFormat::Uint,
                        (_, _neither_signed_nor_unsigned_ha) => todo!(),
                    }),
                    required: vec![],
                    properties: None,
                    minimum: None,
                    maximum: None,
                },
                facet::NumberBits::Float {
                    sign_bits,
                    exponent_bits,
                    mantissa_bits,
                } => Property::Type {
                    r#type: TypeOrTypes::Type(Type::Number),
                    format: Some(match (sign_bits, exponent_bits, mantissa_bits) {
                        (1, 8, 23) => TypeFormat::Float,
                        (1, 11, 52) => TypeFormat::Double,
                        _ => panic!("AAAAA"),
                    }),
                    required: vec![],
                    properties: None,
                    minimum: None,
                    maximum: None,
                },
                facet::NumberBits::Fixed {
                    sign_bits,
                    integer_bits,
                    fraction_bits,
                } => todo!(),
                _ => todo!(),
            },
            facet::ScalarAffinity::String(string_affinity) => Property::Type {
                r#type: TypeOrTypes::Type(Type::String),
                format: None,
                required: vec![],
                properties: None,
                minimum: None,
                maximum: None,
            },
            facet::ScalarAffinity::Boolean(bool_affinity) => Property::Type {
                r#type: TypeOrTypes::Type(Type::Boolean),
                format: None,
                required: vec![],
                properties: None,
                minimum: None,
                maximum: None,
            },
            facet::ScalarAffinity::Empty(empty_affinity) => todo!(),
            facet::ScalarAffinity::SocketAddr(socket_addr_affinity) => todo!(),
            facet::ScalarAffinity::IpAddr(ip_addr_affinity) => todo!(),
            facet::ScalarAffinity::Opaque(opaque_affinity) => todo!(),
            facet::ScalarAffinity::Other(other_affinity) => todo!(),
            _ => todo!(),
        },

        facet::Def::Struct(struct_def) => match struct_def.kind {
            facet::StructKind::Struct => Property::Type {
                r#type: TypeOrTypes::Type(Type::Object),
                format: None,
                required: struct_def
                    .fields
                    .iter()
                    .filter(|field| !matches!(field.shape.def, facet::Def::Option(_)))
                    .map(|field| field.name.to_owned())
                    .collect(),

                properties: Some(
                    struct_def
                        .fields
                        .iter()
                        .map(|field| {
                            if is_struct(field.shape) {
                                let (property, new_definitions, name) = analyze_shape(field.shape);
                                let struct_ref = format!("#/definitions/{name}");
                                definitions.insert(name, property);
                                definitions.extend(new_definitions);
                                (field.name.to_owned(), Property::Ref { r#ref: struct_ref })
                            } else {
                                let (property, new_definitions, name) = analyze_shape(field.shape);
                                definitions.extend(new_definitions);
                                (field.name.to_owned(), property)
                            }
                        })
                        .collect(),
                ),
                minimum: None,
                maximum: None,
            },
            facet::StructKind::TupleStruct => todo!(),
            facet::StructKind::Tuple => todo!(),
            _ => todo!(),
        },
        facet::Def::Map(map_def) => Property::Type {
            r#type: TypeOrTypes::Type(Type::Object),
            format: None,
            required: vec![],
            properties: None,
            minimum: None,
            maximum: None,
        },
        facet::Def::List(list_def) => Property::Type {
            r#type: TypeOrTypes::Type(Type::Array),
            format: None,
            required: vec![],
            properties: None,
            minimum: None,
            maximum: None,
        },
        facet::Def::Option(option_def) => {
            let (property_inner, new_definitions, name) = analyze_shape(option_def.t);
            let new_property = match property_inner {
                Property::Type {
                    r#type: TypeOrTypes::Type(Type::Object),
                    format,
                    required,
                    properties,
                    minimum,
                    maximum,
                } => {
                    definitions.insert(
                        name.clone(),
                        Property::Type {
                            r#type: TypeOrTypes::Type(Type::Object),
                            format,
                            required,
                            properties,
                            minimum,
                            maximum,
                        },
                    );
                    Property::AnyOf {
                        any_of: vec![
                            Property::Ref {
                                r#ref: format!("#/definitions/{name}"),
                            },
                            Property::Type {
                                r#type: TypeOrTypes::Type(Type::Null),
                                format: None,
                                required: vec![],
                                properties: None,
                                minimum: None,
                                maximum: None,
                            },
                        ],
                    }
                }
                Property::Type {
                    r#type,
                    format,
                    required,
                    properties,
                    minimum,
                    maximum,
                } => Property::Type {
                    r#type: TypeOrTypes::Types({
                        let mut types: Vec<_> = r#type.into_iter().chain([Type::Null]).collect();
                        types.sort();
                        types.dedup();
                        types
                    }),
                    format,
                    required,
                    properties,
                    minimum,
                    maximum,
                },
                other => other,
            };
            definitions.extend(new_definitions);
            new_property
        }
        facet::Def::Enum(_) => Property::AnyOf { any_of: todo!() },
        _ => panic!("AAAAAA I can't deal with being in the future"),
    };
    (property, definitions, get_type_name(shape))
}

fn is_struct(shape: &facet::Shape) -> bool {
    matches!(shape.def, facet::Def::Struct(_))
}

type Definitions = HashMap<String, Property>;

#[derive(Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Schema {
    #[serde(rename = "$schema")]
    schema: &'static str,
    title: String,
    #[serde(flatten)]
    root_property: Property,
    #[serde(default, skip_serializing_if = "Definitions::is_empty")]
    definitions: Definitions,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Type {
    Object,
    Boolean,
    Integer,
    Number,
    Array,
    String,
    Null,
}

#[derive(Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
pub enum Property {
    Type {
        r#type: TypeOrTypes,
        #[serde(skip_serializing_if = "Option::is_none")]
        format: Option<TypeFormat>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        required: Vec<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        properties: Option<HashMap<String, Property>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        minimum: Option<i128>,
        #[serde(skip_serializing_if = "Option::is_none")]
        maximum: Option<i128>,
    },
    Ref {
        #[serde(rename = "$ref")]
        r#ref: String,
    },
    AnyOf {
        #[serde(rename = "anyOf")]
        any_of: Vec<Property>,
    },
}

#[derive(Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
pub enum TypeOrTypes {
    Type(Type),
    Types(Vec<Type>),
}

impl IntoIterator for TypeOrTypes {
    type Item = Type;
    type IntoIter = <Vec<Type> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            TypeOrTypes::Type(t) => vec![t].into_iter(),
            TypeOrTypes::Types(items) => items.into_iter(),
        }
    }
}

#[derive(Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TypeFormat {
    Int,
    Int8,
    Int16,
    Int32,
    Int64,
    Int128,
    Uint,
    Uint8,
    Uint16,
    Uint32,
    Uint64,
    Uint128,
    Float,
    Double,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_struct() {
        #[derive(facet_derive::Facet)]
        struct WowSoTest {
            much: String,
            tdd: bool,
            is: i8,
            happening: usize,
        }

        assert_eq!(
            schema::<WowSoTest>(),
            serde_json::from_str(include_str!("../test_data/simple_struct.json")).unwrap(),
        )
    }

    #[test]
    fn test_nested_struct() {
        #[derive(facet_derive::Facet)]
        struct WowSoTest {
            outer: i32,
            inner: InnerWowSoTest,
        }
        #[derive(facet_derive::Facet)]
        struct InnerWowSoTest {
            payload: i32,
        }

        assert_eq!(
            schema::<WowSoTest>(),
            serde_json::from_str(include_str!("../test_data/nested_struct.json")).unwrap(),
        )
    }

    #[test]
    fn test_double_nested_struct() {
        #[derive(facet_derive::Facet)]
        struct WowSoTest {
            outer: i32,
            inner: InnerWowSoTest,
        }
        #[derive(facet_derive::Facet)]
        struct InnerWowSoTest {
            payload: i32,
            subinner: InnerInnerWowSoTest,
        }
        #[derive(facet_derive::Facet)]
        struct InnerInnerWowSoTest {
            payload2: i32,
        }

        assert_eq!(
            schema::<WowSoTest>(),
            serde_json::from_str(include_str!("../test_data/double_nested_struct.json")).unwrap(),
        )
    }

    #[test]
    fn test_option_fields() {
        #[derive(facet_derive::Facet)]
        struct WowSoTest {
            needed: i32,
            nice_to_have: Option<i32>,
            nested: InnerWowSoTest,
        }
        #[derive(facet_derive::Facet)]
        struct InnerWowSoTest {
            subinner: Option<InnerInnerWowSoTest>,
        }
        #[derive(facet_derive::Facet)]
        struct InnerInnerWowSoTest {
            payload: i32,
        }

        assert_eq!(
            schema::<WowSoTest>(),
            serde_json::from_str(include_str!("../test_data/option_fields.json")).unwrap(),
        )
    }
}
