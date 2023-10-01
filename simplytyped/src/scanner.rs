use anyhow::anyhow;

#[derive(Debug, PartialEq)]
pub enum Token {
    Identifier(String),
    Lambda,
    Equal,
    Dot,
    LeftParen,
    RightParen,
    Colon,
    True,
    False,
    Bool,
    If,
    Then,
    Else,
    Arrow,
}

pub struct Scanner {
    stream: String,
    pos: usize,
}

impl Scanner {
    pub fn scan(stream: String) -> anyhow::Result<Vec<Token>> {
        let mut scanner = Self { stream, pos: 0 };
        let mut tokens = Vec::new();
        while let Some(token) = scanner.scan_token() {
            tokens.push(token?);
        }
        Ok(tokens)
    }

    fn scan_token(&mut self) -> Option<anyhow::Result<Token>> {
        match self.advance()? {
            '\\' => Some(Ok(Token::Lambda)),
            '.' => Some(Ok(Token::Dot)),
            '=' => Some(Ok(Token::Equal)),
            '(' => Some(Ok(Token::LeftParen)),
            ')' => Some(Ok(Token::RightParen)),
            ':' => Some(Ok(Token::Colon)),
            '-' => {
                if let Some('>') = self.advance() {
                    Some(Ok(Token::Arrow))
                } else {
                    Some(Err(anyhow!("unexpected symbol `-`")))
                }
            }
            c => {
                if c.is_alphabetic() {
                    let start = self.pos - 1;
                    while matches!(self.advance(), Some(c) if !" .\\():-".contains(c)) {}
                    self.pos -= 1;
                    let ident = self.stream.get(start..self.pos)?;
                    let tok = match ident {
                        "true" => Token::True,
                        "false" => Token::False,
                        "if" => Token::If,
                        "then" => Token::Then,
                        "else" => Token::Else,
                        "bool" => Token::Bool,
                        _ => Token::Identifier(ident.to_string()),
                    };
                    Some(Ok(tok))
                } else if c.is_whitespace() {
                    while matches!(self.advance(), Some(c) if c.is_whitespace()) {}
                    self.pos -= 1;
                    self.scan_token()
                } else {
                    Some(Err(anyhow!("invalid identifier")))
                }
            }
        }
    }

    fn advance(&mut self) -> Option<char> {
        self.pos += 1;
        self.stream.chars().nth(self.pos - 1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn works() {
        let stream = String::from("(\\x:bool.x) true");
        let tokens = Scanner::scan(stream).unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::LeftParen,
                Token::Lambda,
                Token::Identifier("x".into()),
                Token::Colon,
                Token::Bool,
                Token::Dot,
                Token::Identifier("x".into()),
                Token::RightParen,
                Token::True,
            ]
        );
    }
}
