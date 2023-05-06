use std::{error::Error, fmt::Display};

use crate::formula::{Molecule, SyntaxNode, TokenLoc};

#[derive(Debug)]
pub enum StoikError {
    InvalidToken(TokenLoc),
    InvalidInput(String),
    NumberFirst(TokenLoc),
    UnpairedParenthesis(TokenLoc),
    UnpairedBracket(TokenLoc),
    EmptyMolecule,
    InvalidNode(SyntaxNode, Molecule),
}

impl Display for StoikError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StoikError::InvalidInput(s) => write!(f, "Invalid input: {s}"),
            StoikError::NumberFirst(_) => write!(f, "A number cannot be the first element in a component formula (`2H2O` is legal, `Cr2(*5*SO4)3` is not)"),
            StoikError::InvalidToken(_) => write!(f, "Invalid token"),
            StoikError::UnpairedBracket(_) => write!(f, "Unpaired bracket"),
            StoikError::UnpairedParenthesis(_) => write!(f, "Unpaired parenthesis"),
            StoikError::EmptyMolecule => write!(f, "Cannot have an empty molicule"),
            StoikError::InvalidNode(node, s) => write!(f, "Invalid syntax node {node:?}. Molicule: {s}"),
        }
    }
}

#[allow(clippy::match_single_binding)]
impl Error for StoikError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            _ => None,
        }
    }
}
