//! # Stoik
//! Stoicimetic utilities written in rust.
//!
//! To see an example how this is used in the real world, look at the [CLI](TODO)
//!
//! ## Example use
//!
//! ```
//! use stoik::formula::Molecule;
//! use stoik::StoikError;
//!
//! let formula = "Rh2(SO4)3";
//! match Molecule::from_formula(formula) {
//!     Err(e) => match e {
//!         StoikError::InvalidToken(loc) => {
//!             loc.print_msg(formula, "Malformed formula", "Illegal token")
//!         }
//!         StoikError::NumberFirst(loc) => loc.print_msg(
//!             formula,
//!             "Malformed formula",
//!             "Compound groups cannot start\nwith numbers",
//!         ),
//!         StoikError::UnpairedParenthesis(loc) => {
//!             loc.print_msg(formula, "Malformed formula", "Unpaired parenthesis")
//!         }
//!         StoikError::UnpairedBracket(loc) => {
//!             loc.print_msg(formula, "Malformed formula", "Unpaired bracket")
//!         }
//!         e => println!("{e}"),
//!     },
//!     Ok(mol) => {
//!         println!("{formula} contains:")
//!         
//!     },
//! }
//! ```
#![warn(missing_docs)]

mod err;
pub mod formula;

pub use err::StoikError;
