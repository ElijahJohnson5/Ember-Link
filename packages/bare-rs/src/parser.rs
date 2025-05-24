use crate::lexer::Token;
use crate::ast::*;
use std::iter::Peekable;
use std::vec::IntoIter;

pub struct Parser {
    tokens: Peekable<IntoIter<Token>>,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens: tokens.into_iter().peekable(),
        }
    }

    pub fn parse_user_types(&mut self) -> Result<Ast, String> {
        let mut types = Vec::new();
        while self.tokens.peek().is_some() {
            types.push(self.parse_user_type()?);
        }

        Ok(Ast { user_types: types })
    }

    fn parse_user_type(&mut self) -> Result<UserType, String> {
        self.expect(&Token::Type)?;
        let name = match self.tokens.next() {
            Some(Token::UserType(n)) => {
                if n.contains("_") {
                    return Err(format!("User type name cannot contain underscores: {}", n))
                } else {
                    n
                }
            },
            other => return Err(format!("Expected user-type-name, got {:?}", other)),
        };
        let typ = self.parse_any_type()?;
        Ok(UserType { name, definition: typ })
    }

    fn parse_any_type(&mut self) -> Result<AnyType, String> {
        match self.tokens.next() {
            Some(Token::Primitive(name)) if name == "data" => {
                let length = if let Some(Token::LBracket) = self.tokens.peek() {
                    self.tokens.next();
                    let len = match self.tokens.next() {
                        Some(Token::Number(n)) => n,
                        other => return Err(format!("Expected number in data[], got {:?}", other)),
                    };
                    self.expect(&Token::RBracket)?;
                    Some(len)
                } else {
                    None
                };
                Ok(AnyType::Data(length))
            }
            Some(Token::Primitive(name)) => Ok(AnyType::Primitive(name)),
            Some(Token::UserType(name)) => Ok(AnyType::UserTypeRef(name)),
            Some(Token::Optional) => {
                let inner = self.parse_wrapped_type()?;
                Ok(AnyType::Optional(Box::new(inner)))
            }
            Some(Token::List) => {
                let inner = self.parse_wrapped_type()?;
                let length = if let Some(Token::LBracket) = self.tokens.peek() {
                    self.tokens.next();
                    let len = match self.tokens.next() {
                        Some(Token::Number(n)) => n,
                        other => return Err(format!("Expected number in list[], got {:?}", other)),
                    };
                    self.expect(&Token::RBracket)?;
                    Some(len)
                } else {
                    None
                };
                Ok(AnyType::List(Box::new(inner), length))
            }
            Some(Token::Map) => {
                let key = self.parse_wrapped_type()?;
                let value = self.parse_wrapped_type()?;
                Ok(AnyType::Map(Box::new(key), Box::new(value)))
            }
            Some(Token::Enum) => {
                self.expect(&Token::LBrace)?;
                let mut vals = Vec::new();
                while let Some(Token::UserType(name)) = self.tokens.peek().cloned() {
                    self.tokens.next();
                    let value = if let Some(Token::Equal) = self.tokens.peek() {
                        self.tokens.next();
                        if let Some(Token::Number(x)) = self.tokens.next() { Some(x) } else { return Err(format!("Expected integer after = in enum")) }
                    } else { None };
                    vals.push(EnumVariant { name, value });
                }
                self.expect(&Token::RBrace)?;
                Ok(AnyType::Enum(vals))
            }
            Some(Token::Union) => {
                self.expect(&Token::LBrace)?;
                let mut members = Vec::new();
                while let Some(tok) = self.tokens.peek() {
                    if tok == &Token::RBrace { break; }
                    if tok == &Token::Pipe { self.tokens.next(); continue; }
                    let typ = self.parse_any_type()?;
                    let tag = if let Some(Token::Equal) = self.tokens.peek() {
                        self.tokens.next();
                        match self.tokens.next() {
                            Some(Token::Number(n)) => Some(n),
                            _ => return Err(format!("Expected number after = in union")),
                        }
                    } else {
                        None
                    };
                    members.push(UnionMember { typ, tag });
                }
                self.expect(&Token::RBrace)?;
                Ok(AnyType::Union(members))
            }
            Some(Token::Struct) => {
                self.expect(&Token::LBrace)?;
                let mut fields = Vec::new();
                while let Some(Token::Identifier(name)) = self.tokens.peek().cloned() {
                    self.tokens.next();
                    self.expect(&Token::Colon)?;
                    let typ = self.parse_any_type()?;
                    fields.push(StructField { name, typ });
                }
                self.expect(&Token::RBrace)?;
                Ok(AnyType::Struct(fields))
            }
            other => return Err(format!("Unexpected token in type: {:?}", other)),
        }
    }

    fn parse_wrapped_type(&mut self) -> Result<AnyType, String> {
        self.expect(&Token::LAngle)?;
        let ty = self.parse_any_type();
        self.expect(&Token::RAngle)?;
        ty
    }

    fn expect(&mut self, token: &Token) -> Result<(), String> {
        let actual = self.tokens.next().unwrap();
        if actual != *token {
            Err(format!("Expected {:?}, found {:?}", token, actual))
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::lexer::Lexer;

    use super::*;

    fn parse_schema(src: &str) -> Ast {
        let toks = Lexer::new(src).lex().unwrap();
        Parser::new(toks).parse_user_types().unwrap()
    }

    #[test]
    fn test_parse_simple_type_ref() {
        let ast = parse_schema("type A u8");
        assert_eq!(ast.user_types.len(), 1);
        let ut = &ast.user_types[0];
        assert_eq!(ut.name, "A");
        match &ut.definition {
            AnyType::Primitive(p) => assert_eq!(p, "u8"),
            _ => panic!("Expected primitive type"),
        }
    }

    #[test]
    fn test_parse_data_with_length() {
        let ast = parse_schema("type B data[10]");
        match &ast.user_types[0].definition {
            AnyType::Data(Some(10)) => {},
            _ => panic!("Expected data[10]"),
        }
    }

    #[test]
    fn test_parse_optional_and_list() {
        let ast = parse_schema("type C optional<u16> type D list<str>[5]");
        assert_eq!(ast.user_types.len(), 2);
        match &ast.user_types[0].definition {
            AnyType::Optional(inner) => match &**inner {
                AnyType::Primitive(p) => assert_eq!(p, "u16"),
                _ => panic!(),
            },
            _ => panic!(),
        }
        match &ast.user_types[1].definition {
            AnyType::List(inner, Some(5)) => match &**inner {
                AnyType::Primitive(p) => assert_eq!(p, "str"),
                _ => panic!(),
            },
            _ => panic!(),
        }
    }

    #[test]
    fn test_parse_map() {
        let ast = parse_schema("type M map<u8> <i32>");
        match &ast.user_types[0].definition {
            AnyType::Map(key, val) => {
                match &**key { AnyType::Primitive(k) => assert_eq!(k, "u8"), _ => panic!() }
                match &**val { AnyType::Primitive(v) => assert_eq!(v, "i32"), _ => panic!() }
            }
            _ => panic!(),
        }
    }

    #[test]
    fn test_parse_enum() {
        let ast = parse_schema("type E enum { Red = 1 Green Blue = 3 }");
        match &ast.user_types[0].definition {
            AnyType::Enum(vals) => {
                assert_eq!(vals.len(), 3);
                assert_eq!(&vals[0].name, "Red"); assert_eq!(vals[0].value, Some(1));
                assert_eq!(&vals[1].name, "Green"); assert_eq!(vals[1].value, None);
                assert_eq!(&vals[2].name, "Blue"); assert_eq!(vals[2].value, Some(3));
            }
            _ => panic!(),
        }
    }

    #[test]
    fn test_parse_struct() {
        let ast = parse_schema("type S struct { x: u8 y: bool }");
        match &ast.user_types[0].definition {
            AnyType::Struct(fields) => {
                assert_eq!(fields.len(), 2);
                assert_eq!(fields[0].name, "x"); match &fields[0].typ { AnyType::Primitive(p) => assert_eq!(p, "u8"), _ => panic!() }
                assert_eq!(fields[1].name, "y"); match &fields[1].typ { AnyType::Primitive(p) => assert_eq!(p, "bool"), _ => panic!() }
            }
            _ => panic!(),
        }
    }

    #[test]
    #[should_panic(expected = "User type name cannot contain underscores:")]
    fn test_invalid_user_type_name() {
        parse_schema("type Bad_Name <u8>");
    }
}
