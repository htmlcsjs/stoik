use std::{error::Error, fmt::Display};

use crate::formula::{Molecule, SyntaxNode, TokenLoc};

#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// The error type for this crate
pub enum StoikError {
    /// A token is in an invalid location
    InvalidToken(TokenLoc),
    /// An input is invalid, this is used as a generic error
    InvalidInput(String),
    /// When a number is the first element in a compenet formula
    /// e.g. in this `Cr2(5SO4)3` 5 would cause this, but the 3 in `3H2O` wont
    NumberFirst(TokenLoc),
    /// Unpared parenthesis
    UnpairedParenthesis(TokenLoc),
    /// Unpaired square bracket
    UnpairedBracket(TokenLoc),
    /// Empty molecule
    EmptyMolecule,
    /// Invalid syntax node and the half build molecule
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
