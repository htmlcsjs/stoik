use std::{fmt::Display, iter::Peekable, str::Chars};

const NOT_OTHER: [char; 14] = [
    '(', '[', ')', ']', '0', '1', '2', '3', '4', '5', '6', '7', '8', '9',
];

pub struct TokenStream<'a> {
    iter: Peekable<Chars<'a>>,
    pos: usize,
}

impl Iterator for TokenStream<'_> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        // Yeet whitespace
        while let Some(c) = self.iter.peek() {
            if c.is_whitespace() {
                let _ = self.iter.next();
                self.pos += 1;
            } else {
                break;
            }
        }
        if self.iter.peek().is_none() {
            None
        } else if let Some(c) = self.iter.next() {
            // handle parens and brackets
            self.pos += 1;

            if c == '(' {
                Some(Token::OpenParen(TokenLoc::new(self.pos, 1)))
            } else if c == '[' {
                Some(Token::OpenBracket(TokenLoc::new(self.pos, 1)))
            } else if c == ')' {
                Some(Token::CloseParen(TokenLoc::new(self.pos, 1)))
            } else if c == ']' {
                Some(Token::CloseBracket(TokenLoc::new(self.pos, 1)))
            } else if let '0'..='9' = c {
                // handle numbers

                let mut count = c.to_string();
                while let Some(c) = self.iter.peek() {
                    if let '0'..='9' = c {
                        count += &self.iter.next().unwrap().to_string();
                    } else {
                        break;
                    }
                }
                let pos = self.pos;
                self.pos += count.len() - 1;
                Some(Token::Number(
                    count.parse::<i64>().unwrap(),
                    TokenLoc::new(pos, count.len()),
                ))
            } else if c.is_uppercase() {
                // handle alphabetic atoms

                let mut atom = c.to_string();
                while let Some(c) = self.iter.peek() {
                    if c.is_lowercase() {
                        atom += &self.iter.next().unwrap().to_string();
                    } else {
                        break;
                    }
                }
                let pos = self.pos;
                self.pos += atom.len() - 1;
                Some(Token::Atom(atom, TokenLoc::new(pos, self.pos - pos + 1)))
            } else {
                // idk how to handle other chars, get yeeted here lmao
                let mut output = c.to_string();
                while let Some(c) = self.iter.peek() {
                    if !NOT_OTHER.contains(c) && !c.is_uppercase() && !c.is_whitespace() {
                        output += &self.iter.next().unwrap().to_string();
                    } else {
                        break;
                    }
                }
                let pos = self.pos;
                self.pos += output.len() - 1;
                Some(Token::Other(output, TokenLoc::new(pos, self.pos - pos + 1)))
            }
        } else {
            None
        }
    }
}

impl<'a> TokenStream<'a> {
    pub fn new(formula: &'a str) -> Self {
        Self {
            iter: formula.chars().peekable(),
            pos: 0,
        }
    }
}

#[derive(Debug)]
pub enum Token {
    /// An opening square bracket - `[`
    OpenBracket(TokenLoc),
    /// A closing square bracket - `]`
    CloseBracket(TokenLoc),
    /// An opening parenthesis - `(`
    OpenParen(TokenLoc),
    /// A closing parenthesis - `)`
    CloseParen(TokenLoc),
    /// A i64 number
    Number(i64, TokenLoc),
    /// Any capital followed by 0 or more lowercase
    Atom(String, TokenLoc),
    /// Anything else that could not fit above, normally erronious
    Other(String, TokenLoc),
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::OpenBracket(_) => write!(f, "`[`"),
            Token::CloseBracket(_) => write!(f, "`]`"),
            Token::OpenParen(_) => write!(f, "`(`"),
            Token::CloseParen(_) => write!(f, "`)`"),
            Token::Number(n, _) => write!(f, "#`{n}`"),
            Token::Atom(s, _) => write!(f, "a`{s}`"),
            Token::Other(s, _) => write!(f, "o`{s}`"),
        }
    }
}

#[derive(Debug)]
pub struct TokenLoc {
    start: usize,
    len: usize,
}

impl TokenLoc {
    pub fn new(start: usize, len: usize) -> Self {
        Self { start, len }
    }

    pub fn print_msg(&self, formula: &str, msg: &str, diag: &str) {
        println!("{msg}: {formula}");
        println!(
            "{}{}",
            " ".repeat(msg.len() + 1 + self.start),
            "^".repeat(self.len)
        );
        println!("{}|", " ".repeat(msg.len() + 1 + self.start));
        for s in diag.split('\n') {
            println!("{}{}", " ".repeat(msg.len() + 1 + self.start), s);
        }
    }
}
