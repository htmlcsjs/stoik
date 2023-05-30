use std::{fmt::Display, iter::Peekable, str::Chars};

const NOT_OTHER: [char; 14] = [
    '(', '[', ')', ']', '0', '1', '2', '3', '4', '5', '6', '7', '8', '9',
];

/// This is an iterator over [`Token`] that is constructed from a given formula
///
/// # Examples
///
/// ```
/// use stoik::formula::{TokenStream, Token, TokenLoc};
///
/// let mut ts = TokenStream::new("O2");
/// // The token's locaton is not used to check for equality, so we use the default
/// assert_eq!(Some(Token::Atom("O".to_string(), TokenLoc::default())), ts.next());
/// assert_eq!(Some(Token::Number(2, TokenLoc::default())), ts.next());
/// assert_eq!(None, ts.next());
/// ```
///
/// You can also use normal [`Iterator`] functions on a token stream
///
/// ```
/// use stoik::formula::{TokenStream, Token};
///
/// let mut ts = TokenStream::new("Am(SUS)2[g]");
/// let folded = ts.fold("".to_string(), |s, x| format!("{s} {x}"));
/// assert_eq!(" aAm ( aS aU aS ) #2 [ og ]", folded);
/// ```
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
    /// Creates a new tokenstream from a formula, using [`chars`](str::chars)
    /// as the backing iterator
    pub fn new(formula: &'a str) -> Self {
        Self {
            iter: formula.chars().peekable(),
            pos: 0,
        }
    }
}

#[derive(Debug)]
/// One "lexical" token in a formula. It carries along its location in a formula using [`TokenLoc`]
/// This is intended to be generated with [`TokenStream`]
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

#[allow(dead_code)]
impl Token {
    /// Gets the location of a token.
    ///
    /// # Examples
    /// ```
    /// use stoik::formula::{TokenStream, TokenLoc};
    /// let token = TokenStream::new("A").next().unwrap();
    /// // Token::get_loc returns a poiner so we pass a referance.
    /// assert_eq!(&TokenLoc::new(1, 1), token.get_loc());
    /// ```
    pub fn get_loc(&self) -> &TokenLoc {
        match self {
            Token::OpenBracket(loc) => loc,
            Token::CloseBracket(loc) => loc,
            Token::OpenParen(loc) => loc,
            Token::CloseParen(loc) => loc,
            Token::Number(_, loc) => loc,
            Token::Atom(_, loc) => loc,
            Token::Other(_, loc) => loc,
        }
    }
}

impl PartialEq for Token {
    fn eq(&self, other: &Self) -> bool {
        // ignore tokenloc for equalites
        match (self, other) {
            (Self::OpenBracket(_), Self::OpenBracket(_)) => true,
            (Self::CloseBracket(_), Self::CloseBracket(_)) => true,
            (Self::OpenParen(_), Self::OpenParen(_)) => true,
            (Self::CloseParen(_), Self::CloseParen(_)) => true,
            (Self::Number(lhs, _), Self::Number(rhs, _)) => lhs == rhs,
            (Self::Atom(lhs, _), Self::Atom(rhs, _)) => lhs == rhs,
            (Self::Other(lhs, _), Self::Other(rhs, _)) => lhs == rhs,
            _ => false,
        }
    }
}
impl Eq for Token {}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::OpenBracket(_) => write!(f, "["),
            Token::CloseBracket(_) => write!(f, "]"),
            Token::OpenParen(_) => write!(f, "("),
            Token::CloseParen(_) => write!(f, ")"),
            Token::Number(n, _) => write!(f, "#{n}"),
            Token::Atom(s, _) => write!(f, "a{s}"),
            Token::Other(s, _) => write!(f, "o{s}"),
        }
    }
}

#[derive(Debug, Default, PartialEq, Eq)]
/// The location of one token in a formula. Used for debugging and error reporting
///
/// # Examples
///
/// ```
/// use stoik::formula::{TokenStream, TokenLoc};
///
/// let mut ts = TokenStream::new("Rh2(SO4)3");
/// let paren = ts.find(|x| x.to_string() == "(").unwrap();
/// // Token::get_loc returns a poiner so we pass a referance.
/// assert_eq!(&TokenLoc::new(4, 1), paren.get_loc());
/// ```
pub struct TokenLoc {
    start: usize,
    len: usize,
}

impl TokenLoc {
    /// Constructs a new tokenloc from a start pos and a length
    pub fn new(start: usize, len: usize) -> Self {
        Self { start, len }
    }

    /// Prints a message with the relevant bit of the formula highlighted
    ///
    /// # Examples
    ///
    /// ```
    /// use stoik::formula::TokenLoc;
    ///
    /// let msg1 = TokenLoc::new(1, 1).format_msg("12345", "numbers", "one");
    /// assert_eq!("numbers: 12345
    ///          ^
    ///          one", msg1);
    ///
    /// println!("{}", TokenLoc::new(3, 2).format_msg("12345", "numbers", "hey look\n3+4=7"));
    /// // This prints
    /// // numbers: 12345
    /// //            ^^
    /// //            hey look
    /// //            3+4=7
    pub fn format_msg(&self, formula: &str, msg: &str, diag: &str) -> String {
        let lines = diag.lines().collect::<Vec<_>>();
        let mut return_msg = format!("{msg}: {formula}");
        return_msg += &format!(
            "\n{}{}",
            " ".repeat(msg.len() + 1 + self.start),
            "^".repeat(self.len)
        );
        for s in lines {
            return_msg += &format!("\n{}{}", " ".repeat(msg.len() + 1 + self.start), s);
        }
        return_msg
    }
}
