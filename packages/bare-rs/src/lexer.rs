use std::iter::Peekable;
use std::str::Chars;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Type,
    Enum,
    Optional,
    List,
    Map,
    Union,
    Struct,
    Primitive(String),
    UserType(String),
    Identifier(String),
    Number(u64),
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    LAngle,
    RAngle,
    Colon,
    Equal,
    Pipe,
}

// === Lexer ===
pub struct Lexer<'a> {
    input: Peekable<Chars<'a>>,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Lexer {
            input: input.chars().peekable(),
        }
    }

    pub fn lex(&mut self) -> Result<Vec<Token>, String> {
        let mut tokens = Vec::new();
        while let Some(token) = self.next_token()? {
            tokens.push(token);
        }
        Ok(tokens)
    }

    fn next_token(&mut self) -> Result<Option<Token>, String> {
        while let Some(&ch) = self.input.peek() {
            match ch {
                ' ' | '\t' | '\n' => {
                    self.input.next();
                    continue;
                }
                '#' => {
                    self.skip_comment();
                    continue;
                }
                '{' => {
                    self.input.next();
                    return Ok(Some(Token::LBrace));
                }
                '}' => {
                    self.input.next();
                    return Ok(Some(Token::RBrace));
                }
                '[' => {
                    self.input.next();
                    return Ok(Some(Token::LBracket));
                }
                ']' => {
                    self.input.next();
                    return Ok(Some(Token::RBracket));
                }
                '<' => {
                    self.input.next();
                    return Ok(Some(Token::LAngle));
                }
                '>' => {
                    self.input.next();
                    return Ok(Some(Token::RAngle));
                }
                ':' => {
                    self.input.next();
                    return Ok(Some(Token::Colon));
                }
                '=' => {
                    self.input.next();
                    return Ok(Some(Token::Equal));
                }
                '|' => {
                    self.input.next();
                    return Ok(Some(Token::Pipe));
                }
                '0'..='9' => return Ok(Some(self.lex_number()?)),
                'A'..='Z' => return Ok(Some(self.lex_usertype()?)),
                'a'..='z' => return Ok(Some(self.lex_keyword_or_ident()?)),
                _ => panic!("Unexpected character: {}", ch),
            }
        }

        Ok(None)
    }

    fn lex_number(&mut self) -> Result<Token, String> {
        let mut num = String::new();
        while let Some(&ch) = self.input.peek() {
            if ch.is_ascii_digit() {
                num.push(ch);
                self.input.next();
            } else {
                break;
            }
        }

        match num.parse() {
            Ok(num) => Ok(Token::Number(num)),
            Err(e) => Err(e.to_string())
        }
    }

    fn lex_usertype(&mut self) -> Result<Token, String> {
        let mut name = String::new();
        while let Some(&ch) = self.input.peek() {
            if ch.is_ascii_alphanumeric() || ch == '_' {
                name.push(ch);
                self.input.next();
            } else {
                break;
            }
        }

        Ok(Token::UserType(name))
    }

    fn lex_keyword_or_ident(&mut self) -> Result<Token, String> {
        let mut ident = String::new();
        while let Some(&ch) = self.input.peek() {
            if ch.is_ascii_alphanumeric() || ch == '_' {
                ident.push(ch);
                self.input.next();
            } else {
                break;
            }
        }
        match ident.as_str() {
            "type" => Ok(Token::Type),
            "enum" => Ok(Token::Enum),
            "optional" => Ok(Token::Optional),
            "list" => Ok(Token::List),
            "map" => Ok(Token::Map),
            "union" => Ok(Token::Union),
            "struct" => Ok(Token::Struct),
            "uint" | "u8" | "u16" | "u32" | "u64" |
            "int" | "i8" | "i16" | "i32" | "i64" |
            "f32" | "f64" | "bool" | "str" | "void" | "data" => Ok(Token::Primitive(ident)),
            _ => Ok(Token::Identifier(ident)),
        }
    }

    fn skip_comment(&mut self) {
        while let Some(&c) = self.input.peek() {
            self.input.next();
            if c == '\n' { break; }
        }
    }
}



#[cfg(test)]
mod tests {
    use super::*;

    fn tokenize(input: &str) -> Vec<Token> {
        Lexer::new(input).lex().unwrap()
    }

    #[test]
    fn test_primitive_tokens() {
        let input = "u8 i32 f64 bool str void";
        let tokens = tokenize(input);
        assert_eq!(
            tokens,
            vec![
                Token::Primitive("u8".into()),
                Token::Primitive("i32".into()),
                Token::Primitive("f64".into()),
                Token::Primitive("bool".into()),
                Token::Primitive("str".into()),
                Token::Primitive("void".into()),
            ]
        );
    }

    #[test]
    fn test_struct_syntax() {
        let input = "type Person <struct { name: str age: u8 }>";
        let tokens = tokenize(input);
        assert_eq!(
            tokens,
            vec![
                Token::Type,
                Token::UserType("Person".into()),
                Token::LAngle,
                Token::Struct,
                Token::LBrace,
                Token::Identifier("name".into()),
                Token::Colon,
                Token::Primitive("str".into()),
                Token::Identifier("age".into()),
                Token::Colon,
                Token::Primitive("u8".into()),
                Token::RBrace,
                Token::RAngle
            ]
        );
    }

    #[test]
    fn test_enum_with_values() {
        let input = "type Role <enum { Admin = 0 User = 1 }>";
        let tokens = tokenize(input);
        assert_eq!(
            tokens,
            vec![
                Token::Type,
                Token::UserType("Role".into()),
                Token::LAngle,
                Token::Enum,
                Token::LBrace,
                Token::UserType("Admin".into()),
                Token::Equal,
                Token::Number(0),
                Token::UserType("User".into()),
                Token::Equal,
                Token::Number(1),
                Token::RBrace,
                Token::RAngle
            ]
        );
    }

    #[test]
    fn test_comments_are_skipped() {
        let input = "# Comment 1\ntype A <u8> # inline comment\ntype B <bool>";
        let tokens = tokenize(input);
        assert_eq!(
            tokens,
            vec![
                Token::Type,
                Token::UserType("A".into()),
                Token::LAngle,
                Token::Primitive("u8".into()),
                Token::RAngle,
                Token::Type,
                Token::UserType("B".into()),
                Token::LAngle,
                Token::Primitive("bool".into()),
                Token::RAngle
            ]
        );
    }

    #[test]
    fn test_list_with_length() {
        let input = "type Data <list<u8>[4]>";
        let tokens = tokenize(input);
        assert_eq!(
            tokens,
            vec![
                Token::Type,
                Token::UserType("Data".into()),
                Token::LAngle,
                Token::List,
                Token::LAngle,
                Token::Primitive("u8".into()),
                Token::RAngle,
                Token::LBracket,
                Token::Number(4),
                Token::RBracket,
                Token::RAngle
            ]
        );
    }

    #[test]
    fn test_union_type() {
        let input = "type U <union { | u8 = 1 | i32 = 2 }>";
        let tokens = tokenize(input);
        assert_eq!(
            tokens,
            vec![
                Token::Type,
                Token::UserType("U".into()),
                Token::LAngle,
                Token::Union,
                Token::LBrace,
                Token::Pipe,
                Token::Primitive("u8".into()),
                Token::Equal,
                Token::Number(1),
                Token::Pipe,
                Token::Primitive("i32".into()),
                Token::Equal,
                Token::Number(2),
                Token::RBrace,
                Token::RAngle
            ]
        );
    }
}
