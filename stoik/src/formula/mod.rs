//! This module is the main module for parsing chemical equations
//!
//! See the documentaion for [`Molecule`], [`assemble_tree`], [`TokenStream`] for more info
//! ```
//! use stoik::formula::*;
//!
//! let mut ts = TokenStream::new("Rh2(SO4)3");
//! assert_eq!(Some(Token::Atom("Rh".to_owned(), TokenLoc::default())), ts.next());
//! let tree = assemble_tree(ts)?;
//! let mut mol = Molecule::construct_from_tree(tree)?;
//!
//! mol.increase_atom("O", 3);
//!
//! assert_eq!(Molecule::from_formula("2(SO5)3")?, mol);
//! # Ok::<(), stoik::StoikError>(())
//! ```
mod tokenstream;

use std::{
    collections::{HashMap, VecDeque},
    fmt::{Debug, Display},
};

use serde::{Deserialize, Serialize};
pub use tokenstream::*;

use crate::err::StoikError;

/// A node in a parsed chemical equation syntax tree
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SyntaxNode {
    /// A subcompund, like the `(SO4)` in `Rh2(SO4)3`
    Subcompound(Vec<SyntaxNode>),
    /// A node that has a multiplier. e.g. `O2` would become `{ node: Atom("O"), mul: 2 }`
    Multiplier {
        /// The contained sytntax node
        node: Box<SyntaxNode>,
        /// The multiplier for the node
        mul: i64,
    },
    /// A node that has contains a whole molecule. e.g. `2Fe` would become `{ node: Atom("Fe"), mul: 2 }`
    Mole {
        /// The contained sytntax node
        node: Box<SyntaxNode>,
        /// The multiplier for the node
        mul: i64,
    },
    /// An atom in a compound, like the `Rh` in `Rh2(SO4)3`
    Atom(String),
    /// Nothing
    Empty,
}

/// This assebles a tree of [`SyntaxNode`] from a token stream
///
/// It requires an iterator of [`Token`] passed to it, idealy to be
/// based off of [`TokenStream`]
///
/// # Examples
///
/// Basic usage:
/// ```
/// use stoik::formula::*;
///
/// let ts = TokenStream::new("O2");
/// let tree = assemble_tree(ts)?;
/// assert_eq!(tree, SyntaxNode::Multiplier { node: Box::new(SyntaxNode::Atom("O".to_string())), mul: 2 });
/// # Ok::<(), stoik::StoikError>(())
/// ```
pub fn assemble_tree(mut stream: impl Iterator<Item = Token>) -> Result<SyntaxNode, StoikError> {
    match stream.next() {
        Some(token) => match token {
            Token::Number(i, _) => Ok(SyntaxNode::Mole {
                node: Box::new(internal_tree(stream, None)?),
                mul: i,
            }),
            _ => Ok(internal_tree(stream, Some(token))?),
        },
        None => Err(StoikError::InvalidInput(
            "Empty iter cannot build a valid tree".to_string(),
        )),
    }
}

fn internal_tree(
    mut stream: impl Iterator<Item = Token>,
    start: Option<Token>,
) -> Result<SyntaxNode, StoikError> {
    let mut start = start.or_else(|| stream.next());
    if start.is_none() {
        return Ok(SyntaxNode::Empty);
    }

    let mut bracket_level = 0;
    let mut paren_level = 0;
    let mut nested_start = TokenLoc::new(0, 0);
    let mut nested_stack = Vec::new();

    let mut tree: Vec<SyntaxNode> = Vec::new();
    while let Some(token) = start {
        if paren_level == 0 && bracket_level == 0 {
            match token {
                Token::OpenBracket(loc) => {
                    bracket_level += 1;
                    nested_start = loc
                }
                Token::OpenParen(loc) => {
                    paren_level += 1;
                    nested_start = loc
                }
                Token::CloseBracket(loc) => return Err(StoikError::UnpairedBracket(loc)),
                Token::CloseParen(loc) => return Err(StoikError::UnpairedParenthesis(loc)),

                Token::Number(n, loc) => {
                    let last = tree.pop();
                    if let Some(last) = last {
                        tree.push(SyntaxNode::Multiplier {
                            node: Box::new(last),
                            mul: n,
                        })
                    } else {
                        return Err(StoikError::NumberFirst(loc));
                    }
                }
                Token::Atom(s, _) => tree.push(SyntaxNode::Atom(s)),
                Token::Other(_, loc) => return Err(StoikError::InvalidToken(loc)),
            }
        } else {
            match token {
                Token::OpenBracket(_) => bracket_level += 1,
                Token::CloseBracket(_) => bracket_level -= 1,
                Token::OpenParen(_) => paren_level += 1,
                Token::CloseParen(_) => paren_level -= 1,
                _ => {}
            }
            if bracket_level == 0 && paren_level == 0 {
                tree.push(internal_tree(nested_stack.into_iter(), None)?);
                nested_stack = Vec::new();
            } else {
                nested_stack.push(token);
            }
        }

        start = stream.next();
    }

    if bracket_level != 0 || paren_level != 0 {
        return Err(StoikError::UnpairedParenthesis(nested_start));
    }

    match tree.len() {
        0 => Ok(SyntaxNode::Empty),
        1 => Ok(tree[0].clone()),
        _ => Ok(SyntaxNode::Subcompound(tree)),
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
/// A repesentaion of a molecule used for stoichiometric puroposies
///
/// # Examples
///
/// ```
/// use stoik::formula::Molecule;
///
/// let mol = Molecule::from_formula("Rh2(SO4)3")?;
///
/// assert_eq!(mol.get_count("Rh"), 2);
/// assert_eq!(mol.get_count("O"), 12);
/// # Ok::<(), stoik::StoikError>(())
/// ```
pub struct Molecule {
    /// The mole count of the molecule
    pub moles: i64,
    map: HashMap<String, i64>,
}

#[allow(dead_code)]
impl Molecule {
    /// Increases the count of an atom in the molecule.
    ///
    /// Adds the count instead if the atom is not already present in the molecule.
    ///
    /// # Examples
    ///
    /// ```
    /// use stoik::formula::Molecule;
    ///
    /// let mut oxygen = Molecule::from_formula("O3")?;
    /// oxygen.increase_atom("O", -1);
    /// assert_eq!(oxygen.get_count("O"), 2);
    /// oxygen.increase_atom("Rh", 1);
    /// assert_eq!(oxygen.get_count("Rh"), 1);
    ///
    /// let mut mol = Molecule::from_formula("2 H2O")?;
    /// mol.increase_atom("O", 1);
    /// assert_eq!(mol.get_count("O"), 4);
    /// # Ok::<(), stoik::StoikError>(())
    /// ```
    pub fn increase_atom(&mut self, atom: &str, n: i64) {
        if let Some(count) = self.map.get_mut(atom) {
            *count += n;
        } else {
            self.map.insert(atom.to_string(), n);
        }
    }

    /// Construts a molecule from a sytnax tree
    ///
    /// See docs for [`from_formula`](Self::from_formula) to get usage
    pub fn construct_from_tree(mut root: SyntaxNode) -> Result<Self, StoikError> {
        let mut new = Self {
            moles: 1,
            map: HashMap::new(),
        };

        if let SyntaxNode::Mole { node, mul } = root {
            root = *node;
            new.moles = mul;
        } else if root == SyntaxNode::Empty {
            return Err(StoikError::EmptyMolecule);
        }

        let mut stack = VecDeque::new();
        stack.push_back(MoleculeStackItem::new(root, 1));

        while !stack.is_empty() {
            let MoleculeStackItem { node, mul } = stack.pop_front().unwrap();
            match node {
                SyntaxNode::Subcompound(nodes) => {
                    nodes
                        .into_iter()
                        .for_each(|x| stack.push_back(MoleculeStackItem::new(x, mul)));
                }
                SyntaxNode::Multiplier { node, mul: new_mul } => {
                    stack.push_front(MoleculeStackItem::new(*node, new_mul * mul));
                }
                SyntaxNode::Atom(atom) => new.increase_atom(&atom, mul),
                SyntaxNode::Empty => continue,
                SyntaxNode::Mole { .. } => return Err(StoikError::InvalidNode(node, new)),
            }
        }

        Ok(new)
    }

    /// Gets the count of an atom with respect to mole count
    ///
    /// # Examples
    /// ```
    /// use stoik::formula::Molecule;
    ///
    /// let oxygen = Molecule::from_formula("O2")?;
    /// assert_eq!(oxygen.get_count("O"), 2);
    ///
    /// let water = Molecule::from_formula("2 H2O")?;
    /// assert_eq!(water.get_count("H"), 4);
    /// # Ok::<(), stoik::StoikError>(())
    /// ```
    pub fn get_count(&self, atom: &str) -> i64 {
        *self.map.get(atom).unwrap_or(&0) * self.moles
    }

    /// Convenience function for construct a molicule direcly from a [`&str`]
    ///
    /// # Examples
    /// ```
    /// use stoik::formula::*;
    ///
    /// let ts = TokenStream::new("Rh2(SO4)3");
    /// let tree = assemble_tree(ts)?;
    /// let mol = Molecule::construct_from_tree(tree)?;
    ///
    /// assert_eq!(mol, Molecule::from_formula("Rh2(SO4)3")?);
    /// # Ok::<(), stoik::StoikError>(())
    /// ```
    pub fn from_formula(formula: &str) -> Result<Self, StoikError> {
        Self::construct_from_tree(assemble_tree(TokenStream::new(formula))?)
    }

    /// Gets the molecule in map form, taking into account `moles`
    ///
    /// # Examples
    /// ```
    /// use stoik::formula::Molecule;
    /// let mol = Molecule::from_formula("2 H2O")?;
    /// let map = mol.get_map();
    /// assert_eq!(Some(&4), map.get("H"));
    /// assert_eq!(None, map.get("S"));
    /// # Ok::<(), stoik::StoikError>(())
    pub fn get_map(&self) -> HashMap<String, i64> {
        self.map
            .iter()
            .map(|(s, x)| (s.clone(), x * self.moles))
            .collect()
    }
}

impl Display for Molecule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.moles != 1 {
            write!(f, "{} ", self.moles)?;
        }
        let pairs = &mut self.map.iter().collect::<Vec<_>>();
        pairs.sort_by(|(ak, _), (bk, _)| ak.cmp(bk));
        for (k, v) in pairs {
            if **v != 1 {
                write!(f, "{k}{v}")?;
            } else {
                write!(f, "{k}")?;
            }
        }

        Ok(())
    }
}

#[derive(Debug)]
struct MoleculeStackItem {
    node: SyntaxNode,
    mul: i64,
}

impl MoleculeStackItem {
    pub fn new(node: SyntaxNode, mul: i64) -> Self {
        Self { node, mul }
    }
}
