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
//!             println!("{}",
//!                 loc.format_msg(formula, "Malformed formula", "Illegal token")
//!             )
//!         }
//!         StoikError::NumberFirst(loc) => println!("{}",
//!             loc.format_msg(
//!                 formula,
//!                 "Malformed formula",
//!                 "Compound groups cannot start\nwith numbers",
//!             )
//!         ),
//!         StoikError::UnpairedParenthesis(loc) => {
//!             println!("{}",
//!                 loc.format_msg(formula, "Malformed formula", "Unpaired parenthesis")
//!             )
//!         }
//!         StoikError::UnpairedBracket(loc) => {
//!             println!("{}",
//!                 loc.format_msg(formula, "Malformed formula", "Unpaired bracket")
//!             )
//!         }
//!         e => println!("{e}"),
//!     },
//!     Ok(mol) => {
//!         println!("{formula} contains:")
//!         //...
//!     }
//! }
//! ```
#![warn(missing_docs)]

mod err;
pub mod formula;

pub use err::StoikError;

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::formula::Molecule;

    #[test]
    fn overall_test() {
        let mol = Molecule::from_formula("5(H2O)3((FeW)5CrMo2V)6CoMnSi");
        assert!(mol.is_ok());
        let mol = mol.unwrap();
        assert_eq!(mol.moles, 5);
        assert_eq!(
            mol.get_map(),
            HashMap::from([
                ("V".to_string(), 30,),
                ("Fe".to_string(), 150,),
                ("Mo".to_string(), 60,),
                ("Cr".to_string(), 30,),
                ("Mn".to_string(), 5,),
                ("Co".to_string(), 5,),
                ("Si".to_string(), 5,),
                ("O".to_string(), 15,),
                ("H".to_string(), 30,),
                ("W".to_string(), 150,),
            ])
        );
    }
}
