use std::{collections::HashMap, fmt::Formatter};

pub fn schema<T: facet::Facet>() -> Schema {
    let (root_property, definitions) = analyze_shape(T::SHAPE);
    Schema {
        schema: "http://json-schema.org/draft-07/schema#",
        title: get_type_name(T::SHAPE),
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

fn analyze_shape(shape: &facet::Shape) -> (Property, Definitions) {
    let mut definitions = Definitions::new();
    let property = match shape.def {
        facet::Def::Scalar(scalar_def) => match scalar_def.affinity {
            facet::ScalarAffinity::Number(number_affinity) => match number_affinity.bits {
                facet::NumberBits::Integer { bits, sign } => Property::Type {
                    r#type: Type::Integer,
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
                    required: None,
                    properties: None,
                    minimum: None,
                    maximum: None,
                },
                facet::NumberBits::Float {
                    sign_bits,
                    exponent_bits,
                    mantissa_bits,
                } => Property::Type {
                    r#type: Type::Number,
                    format: Some(match (sign_bits, exponent_bits, mantissa_bits) {
                        (1, 8, 23) => TypeFormat::Float,
                        (1, 11, 52) => TypeFormat::Double,
                        _ => panic!("AAAAA"),
                    }),
                    required: None,
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
                r#type: Type::String,
                format: None,
                required: None,
                properties: None,
                minimum: None,
                maximum: None,
            },
            facet::ScalarAffinity::Boolean(bool_affinity) => Property::Type {
                r#type: Type::Boolean,
                format: None,
                required: None,
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
            facet::StructKind::Struct => {
                Property::Type {
                    r#type: Type::Object,
                    format: None,
                    required: Some(
                        struct_def
                            .fields
                            .iter()
                            // lmao
                            .filter(|field| get_type_name(field.shape) != "Option")
                            .map(|field| field.name.to_owned())
                            .collect(),
                    ),
                    properties: Some(
                        struct_def
                            .fields
                            .iter()
                            .map(|field| {
                                if is_struct(field.shape) {
                                    let struct_name = get_type_name(field.shape);
                                    let struct_ref = format!("#/definitions/{struct_name}");
                                    let (property, new_definitions) = analyze_shape(field.shape);
                                    definitions.insert(struct_name, property);
                                    definitions.extend(new_definitions);
                                    (field.name.to_owned(), Property::Ref { r#ref: struct_ref })
                                } else {
                                    let (property, new_definitions) = analyze_shape(field.shape);
                                    definitions.extend(new_definitions);
                                    (field.name.to_owned(), property)
                                }
                            })
                            .collect(),
                    ),
                    minimum: None,
                    maximum: None,
                }
            }
            facet::StructKind::TupleStruct => todo!(),
            facet::StructKind::Tuple => todo!(),
            _ => todo!(),
        },
        facet::Def::Map(_) => Property::Type {
            r#type: Type::Object,
            format: None,
            required: None,
            properties: None,
            minimum: None,
            maximum: None,
        },
        facet::Def::List(_) => Property::Type {
            r#type: Type::Array,
            format: None,
            required: None,
            properties: None,
            minimum: None,
            maximum: None,
        },
        facet::Def::Enum(_) => Property::AnyOf(todo!()),
        _ => panic!("AAAAAA I can't deal with being in the future"),
    };
    (property, definitions)
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

#[derive(Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Type {
    Object,
    Boolean,
    Integer,
    Number,
    Array,
    String,
}

#[derive(Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
pub enum Property {
    Type {
        r#type: Type,
        #[serde(skip_serializing_if = "Option::is_none")]
        format: Option<TypeFormat>,
        #[serde(skip_serializing_if = "Option::is_none")]
        required: Option<Vec<String>>,
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
    AnyOf(Vec<Type>),
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
}
