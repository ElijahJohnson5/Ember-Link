use convert_case::{Case, Casing};

use std::collections::HashSet;

#[derive(Debug)]
pub enum AnyType {
    Primitive(String),
    Optional(Box<AnyType>),
    List(Box<AnyType>, Option<u64>),
    Map(Box<AnyType>, Box<AnyType>),
    Data(Option<u64>),
    Struct(Vec<StructField>),
    Enum(Vec<EnumVariant>),
    Union(Vec<UnionMember>),
    UserTypeRef(String),
}

#[derive(Debug)]
pub struct Ast {
    pub user_types: Vec<UserType>,
}

#[derive(Debug)]
pub struct UserType {
    pub name: String,
    pub definition: AnyType,
}

#[derive(Debug)]
pub struct StructField {
    pub name: String,
    pub typ: AnyType,
}

#[derive(Debug)]
pub struct EnumVariant {
    pub name: String,
    pub value: Option<u64>,
}

#[derive(Debug)]
pub struct UnionMember {
    pub typ: AnyType,
    pub tag: Option<u64>,
}

#[derive(Debug)]
pub struct Options<'a> {
    pub serde_string: &'a Option<String>,
    pub serde_string_union: &'a Option<String>,
    pub derives: &'a Option<Vec<String>>,
    pub uses: &'a Option<Vec<String>>,
}

impl Ast {
    pub fn to_rust(&self, options: &Options) -> String {
        let mut all_types = vec![];
        let mut hoisted_uses = vec![];

        let mut derives_string: String = "Debug".into();

        if let Some(derives) = options.derives {

            for derive in derives {
                derives_string += &format!(", {}", derive);
            }
        }

        for user_type in self.user_types.iter() {

            let (rust, mut hoisted, mut inner_hoisted_uses) = user_type.to_rust(&derives_string, options);

            all_types.push(rust);
            all_types.append(&mut hoisted);

            hoisted_uses.append(&mut inner_hoisted_uses);
        }

        let mut seen = HashSet::new();

        if let Some(uses) = options.uses {
            for use_string in uses {
                hoisted_uses.push(use_string.clone());
            }
        }

        hoisted_uses.retain(|item| seen.insert(item.clone()));

        let uses = hoisted_uses.join("\n");
        let all_types = all_types.join("\n");

        if uses.is_empty() {
            all_types
        } else {
            uses + "\n" + &all_types
        }
    }
}

impl UserType {
    pub fn to_rust(&self, derives: &str, options: &Options) -> (String, Vec<String>, Vec<String>) {
        let mut hoisted_defs = Vec::new();
        let mut hoisted_uses = Vec::new();

        let type_string = match &self.definition {
            AnyType::Struct(fields) => {
                let mut field_strings = Vec::new();
                for field in fields {
                    let (rust_type, mut hoisted, mut inner_hoisted_uses) = field.to_rust(derives, options, &self.name);
                    field_strings.push(rust_type);
                    hoisted_defs.append(&mut hoisted);
                    hoisted_uses.append(&mut inner_hoisted_uses);
                }

                if let Some(serde_string) = options.serde_string {
                    format!("\n#[derive({})]\n{}\npub struct {} {{\n{}\n}}", derives, serde_string, self.name.to_case(Case::UpperCamel), field_strings.join(",\n"))
                } else {
                    format!("\n#[derive({})]\npub struct {} {{\n{}\n}}", derives, self.name.to_case(Case::UpperCamel), field_strings.join(",\n"))
                }
            }
            AnyType::Enum(variants) => {
                let body = variants.iter().map(|variant| variant.to_rust()).collect::<Vec<_>>().join(",\n");

                format!("\n#[derive({})]\npub enum {} {{\n{}\n}}", derives, self.name.to_case(Case::UpperCamel), body)
            }
            AnyType::Union(members) => {
                let mut member_strings = Vec::new();
                for member in members {
                    let (rust_type, mut hoisted, mut inner_hoisted_uses) = member.to_rust(derives, options, &self.name);
                    member_strings.push(rust_type);
                    hoisted_defs.append(&mut hoisted);
                    hoisted_uses.append(&mut inner_hoisted_uses);
                }

                if let Some(serde_string) = options.serde_string_union {
                    format!("\n#[derive({})]\n{}\npub enum {} {{\n{}\n}}", derives, serde_string, self.name.to_case(Case::UpperCamel), member_strings.join(",\n"))
                } else {
                    format!("\n#[derive({})]\npub enum {} {{\n{}\n}}", derives, self.name.to_case(Case::UpperCamel), member_strings.join(",\n"))
                }
            }
            any => {
                let (rust_type, mut hoisted, mut inner_hoisted_uses) = any.to_rust(derives, options, &self.name, None);

                hoisted_defs.append(&mut hoisted);
                hoisted_uses.append(&mut inner_hoisted_uses);



                format!("\npub type {} = {};", self.name.to_case(Case::UpperCamel), rust_type)
            }
        };

        (type_string, hoisted_defs, hoisted_uses)
    }
}


impl AnyType {
    pub fn to_rust(&self, derives: &str, options: &Options, parent_name: &str, field_name: Option<&str>) -> (String, Vec<String>, Vec<String>) {
        match self {
            AnyType::Primitive(name) => { 
                let rust_type = match name.as_str() {
                    "u8" => "u8".into(),
                    "u16" => "u16".into(),
                    "u32" => "u32".into(),
                    "u64" => "u64".into(),
                    "i8" => "i8".into(),
                    "i16" => "i16".into(),
                    "i32" => "i32".into(),
                    "i64" => "i64".into(),
                    "f32" => "f32".into(),
                    "f64" => "f64".into(),
                    "bool" => "bool".into(),
                    "str" => "String".into(),
                    "void" => "()".into(),
                    other => other.into(),
                };
        
                (rust_type, vec![], vec![])
            }
            AnyType::Optional(typ) => {
                let (inner_str, hoisted_defs, hoisted_use) = typ.to_rust(derives, options, parent_name, field_name);

                (format!("Option<{}>", inner_str), hoisted_defs, hoisted_use)
            },
            AnyType::List(typ, length) => {
                let (inner_str, hoisted_defs, hoisted_use) = typ.to_rust(derives, options, parent_name, field_name);

                let rust_type = if let Some(length) = length {
                    format!("[{}; {}]", inner_str, length)
                } else {
                    format!("Vec<{}>",inner_str)
                };

                (rust_type, hoisted_defs, hoisted_use)
            }
            AnyType::Map(key_typ, value_typ) => {
                let (key_typ, mut key_hoisted, mut key_hoisted_use) = key_typ.to_rust(derives, options,  parent_name, field_name);
                let (value_typ, mut value_hoisted, mut value_hoisted_use) = value_typ.to_rust(derives, options,  parent_name, field_name);

                key_hoisted.append(&mut value_hoisted);
                key_hoisted_use.append(&mut value_hoisted_use);

                let hash_map_use = "use std::collections::HashMap;".into();

                if !(key_hoisted_use.contains(&hash_map_use)) {
                    key_hoisted_use.push(hash_map_use);
                }

                (
                    format!("HashMap<{}, {}>", key_typ, value_typ),
                    key_hoisted,
                    key_hoisted_use
                )
            }
            AnyType::Data(length) => {
                if let Some(length) = length {
                    (format!("[u8; {}]", length), vec![], vec![])
                } else {
                    ("Vec<u8>".into(), vec![], vec![])
                }
            }
            AnyType::Struct(fields) => {
                let struct_name = if let Some(field_name) = field_name {
                    format!("{}{}", parent_name, field_name).to_case(Case::UpperCamel)
                } else {
                    format!("{}Nested", parent_name).to_case(Case::UpperCamel)
                };

                let mut hoisted_defs = Vec::new();
                let mut hoisted_uses = Vec::new();

                let mut field_strings = Vec::new();
                for field in fields {
                    let (rust_type, mut hoisted, mut inner_hoisted_uses) = field.to_rust(derives, options, &struct_name);
                    field_strings.push(rust_type);
                    hoisted_defs.append(&mut hoisted);
                    hoisted_uses.append(&mut inner_hoisted_uses);
                }

                let struct_def = if let Some(serde_string) = options.serde_string {
                    format!(
                        "\n#[derive({})]\n{}\npub struct {} {{\n{}\n}}",
                        derives,
                        serde_string,
                        struct_name,
                        field_strings.join(",\n")
                    )
                } else {
                    format!(
                        "\n#[derive({})]\npub struct {} {{\n{}\n}}",
                        derives,
                        struct_name,
                        field_strings.join(",\n")
                    )
                };
                

                hoisted_defs.insert(0, struct_def);
                (struct_name, hoisted_defs, hoisted_uses)
            }
            AnyType::Enum(variants) => {
                let enum_name = if let Some(field_name) = field_name {
                    format!("{}{}", parent_name, field_name).to_case(Case::UpperCamel)
                } else {
                    format!("{}Enum", parent_name).to_case(Case::UpperCamel)
                };


                let mut variant_strings = Vec::new();
                for variant in variants {
                    let rust_type = variant.to_rust();
                    variant_strings.push(rust_type);
                }

                let enum_def = 
                    format!(
                        "\n#[derive({})]\npub enum {} {{\n{}\n}}",
                        derives,
                        enum_name,
                        variant_strings.join(",\n")
                    );

                (enum_name, vec![enum_def], vec![])
            }
            AnyType::Union(members) => {
                let union_name = if let Some(field_name) = field_name {
                    format!("{}{}", parent_name, field_name).to_case(Case::UpperCamel)
                } else {
                    format!("{}Nested", parent_name).to_case(Case::UpperCamel)
                };

                let mut hoisted_defs = Vec::new();
                let mut hoisted_uses = Vec::new();

                let mut member_strings = Vec::new();
                for member in members {
                    let (rust_type, mut hoisted, mut inner_hoisted_uses) = member.to_rust(derives, options, &union_name);
                    member_strings.push(rust_type);
                    hoisted_defs.append(&mut hoisted);
                    hoisted_uses.append(&mut inner_hoisted_uses);
                }

                let union_def = if let Some(serde_string) = options.serde_string {
                    format!(
                        "\n#[derive({})]\n{}\npub enum {} {{\n{}\n}}",
                        derives,
                        serde_string,
                        union_name,
                        member_strings.join(",\n")
                    )
                } else {
                    format!(
                        "\n#[derive({})]\npub enum {} {{\n{}\n}}",
                        derives,
                        union_name,
                        member_strings.join(",\n")
                    )
                }; 



                hoisted_defs.insert(0, union_def);
                (union_name, hoisted_defs, hoisted_uses)
            }
            AnyType::UserTypeRef(name) => (name.to_case(Case::UpperCamel), vec![], vec![]),
        }
    }
}

impl StructField {
    fn to_rust(&self, derives: &str, options: &Options, parent_name: &str) -> (String, Vec<String>, Vec<String>) {
        let (inner_str, hoisted_defs, hoisted_uses) = self.typ.to_rust(derives, options, parent_name, Some(&self.name.to_case(Case::UpperCamel)));
        (format!("    pub {}: {}", self.name.to_case(Case::Snake), inner_str), hoisted_defs, hoisted_uses)
    }
}

impl EnumVariant {
    fn to_rust(&self) -> String {
        match &self.value {
            Some(value) => format!("    {} = {}", self.name.to_case(Case::UpperCamel), value),
            None => format!("    {}", self.name.to_case(Case::UpperCamel)),
        }
    }
}

impl UnionMember {
    fn to_rust(&self, derives: &str, options: &Options, parent_name: &str) -> (String, Vec<String>, Vec<String>) {
        let (inner_str, hoisted_defs, hoisted_uses) = self.typ.to_rust(derives, options, parent_name, None);

        (format!("    {}({})", inner_str, inner_str), hoisted_defs, hoisted_uses)
    }
}

#[cfg(test)]
mod tests {
    use crate::{lexer, parser};

    use super::*;

    fn parse_schema(schema: &str) -> Ast {
        let tokens = lexer::Lexer::new(schema).lex().unwrap();
        parser::Parser::new(tokens).parse_user_types().unwrap()
    }

    fn assert_string_lines_eq_without_whitespace(a: &str, b: &str) {
        let a_trimmed = a.lines()
            .filter(|line| !line.trim().is_empty()).map(|line| line.trim())
            .collect::<Vec<&str>>()
            .join("\n");
        let b_trimmed = b.lines()
            .filter(|line| !line.trim().is_empty()).map(|line| line.trim())
            .collect::<Vec<&str>>()
            .join("\n");
        assert_eq!(a_trimmed, b_trimmed);
    }

    // Write tests for the to_rust functions for all the types
    #[test]
    fn test_to_rust_enum() {
        let schema = r#"
            type Department enum {
                ACCOUNTING
                ADMINISTRATION
                CUSTOMER_SERVICE
                DEVELOPMENT
                JSMITH = 99
            }
        "#;
        let ast = parse_schema(schema);
        
        assert_string_lines_eq_without_whitespace(
            &ast.to_rust(&Options { serde_string: &None, serde_string_union: &None, derives: &None, uses: &None }), 
            r#"
                #[derive(Debug)]
                pub enum Department {
                    Accounting,
                    Administration,
                    CustomerService,
                    Development,
                    Jsmith = 99
                }
            "#.trim(),
        );
    }

    #[test]
    fn test_to_rust_struct() {
        let schema = r#"
            type Customer struct {
                name: str
                email: str
                address: Address
            }
        "#;
        let ast = parse_schema(schema);

        assert_string_lines_eq_without_whitespace(
            &ast.to_rust(&Options { serde_string: &None, serde_string_union: &None, derives: &None, uses: &None }), 
            r#"
                #[derive(Debug)]
                pub struct Customer {
                    pub name: String,
                    pub email: String,
                    pub address: Address
                }
            "#.trim(),
        );
    }

    #[test]
    fn test_to_rust_list() {
        let schema = r#"
            type Address list<str>[4] # street, city, state, country
        "#;
        let ast = parse_schema(schema);
        assert_string_lines_eq_without_whitespace(
            &ast.to_rust(&Options { serde_string: &None, serde_string_union: &None, derives: &None, uses: &None }), 
            r#"
                pub type Address = [String; 4];
            "#.trim(),
        );
    }

    #[test]
    fn test_with_derives_array() {
        let schema = r#"
            type Customer struct {
                name: str
                email: str
                address: Address
                phone: list<str>[4]
            }
        "#;
        let ast = parse_schema(schema);
        assert_string_lines_eq_without_whitespace(
            &ast.to_rust(&Options { serde_string: &None, serde_string_union: &None, derives: &Some(vec!["Clone".into(), "Serialize".into()]), uses: &None }), 
            r#"
                #[derive(Debug, Clone, Serialize)]
                pub struct Customer {
                    pub name: String,
                    pub email: String,
                    pub address: Address,
                    pub phone: [String; 4]
                }
            "#.trim(),
        );
    }

    #[test]
    fn test_with_uses_array() {
        let schema = r#"
            type Customer struct {
                name: str
                email: str
                address: Address
                phone: list<str>[4]
            }
        "#;
        let ast = parse_schema(schema);
        assert_string_lines_eq_without_whitespace(
            &ast.to_rust(&Options { serde_string: &None, serde_string_union: &None, derives: &None, uses: &Some(vec!["use std::collections::HashMap".into()]) }),
            r#"
                use serde::{Deserialize, Serialize}
                #[derive(Debug)]
                pub struct Customer {
                    pub name: String,
                    pub email: String,
                    pub address: Address,
                    pub phone: [String; 4]
                }
            "#.trim(),
        );
    }

    #[test]
    fn test_to_rust_list_no_length() {
        let schema = r#"
            type Address list<str>
        "#;
        let ast = parse_schema(schema);
        assert_string_lines_eq_without_whitespace(
            &ast.to_rust(&Options { serde_string: &None, serde_string_union: &None, derives: &None, uses: &None }), 
            r#"
                pub type Address = Vec<String>;
            "#.trim(),
        );
    }

    #[test]
    fn test_complicated_example() {
        let schema = r#"
            type PublicKey data[128]
            type Time str # ISO 8601

            type Department enum {
                ACCOUNTING
                ADMINISTRATION
                CUSTOMER_SERVICE
                DEVELOPMENT

                # Reserved for the CEO
                JSMITH = 99
            }

            type Address list<str>[4] # street, city, state, country

            type Customer struct {
                name: str
                email: str
                address: Address
                orders: list<struct {
                    orderId: i64
                    quantity: i32
                    price: struct {
                        amount: f64
                        currency: enum {
                            USD
                            EUR = 99
                        }
                    }
                }>
                metadata: map<str><data>
            }

            type Employee struct {
                name: str
                email: str
                address: Address
                department: Department
                hireDate: Time
                publicKey: optional<PublicKey>
                metadata: map<str><data>
            }

            type TerminatedEmployee void
        "#;

        let ast = parse_schema(schema);
        assert_string_lines_eq_without_whitespace(
            &ast.to_rust(&Options { serde_string: &None, serde_string_union: &None, derives: &None, uses: &None }), 
            r#"
                use std::collections::HashMap;
                pub type PublicKey = [u8; 128];
                pub type Time = String;
                #[derive(Debug)]
                pub enum Department {
                    Accounting,
                    Administration,
                    CustomerService,
                    Development,
                    Jsmith = 99
                }
                pub type Address = [String; 4];
                #[derive(Debug)]
                pub struct Customer {
                    pub name: String,
                    pub email: String,
                    pub address: Address,
                    pub orders: Vec<CustomerOrders>,
                    pub metadata: HashMap<String, Vec<u8>>
                }
                #[derive(Debug)]
                pub struct CustomerOrders {
                    pub order_id: i64,
                    pub quantity: i32,
                    pub price: CustomerOrdersPrice
                }
                #[derive(Debug)]
                pub struct CustomerOrdersPrice {
                    pub amount: f64,
                    pub currency: CustomerOrdersPriceCurrency
                }
                #[derive(Debug)]
                pub enum CustomerOrdersPriceCurrency {
                    Usd,
                    Eur = 99
                }
                #[derive(Debug)]
                pub struct Employee {
                    pub name: String,
                    pub email: String,
                    pub address: Address,
                    pub department: Department,
                    pub hire_date: Time,
                    pub public_key: Option<PublicKey>,
                    pub metadata: HashMap<String, Vec<u8>>
                }
                pub type TerminatedEmployee = ();
            "#.trim()
        );
    }
}