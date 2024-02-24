# Stoik

Stoicimetic utilities written in rust.

To see an example how this is used in the real world, look at the [CLI](https://github.com/htmlcsjs/stoik/tree/main/stoik-cli)

## Example use

```rust
use stoik::formula::Molecule;
use stoik::StoikError;
let formula = "Rh2(SO4)3";
match Molecule::from_formula(formula) {
    Err(e) => match e {
        StoikError::InvalidToken(loc) => {
            println!("{}",
                loc.format_msg(formula, "Malformed formula", "Illegal token")
            )
        }
        StoikError::NumberFirst(loc) => println!("{}",
            loc.format_msg(
                formula,
                "Malformed formula",
                "Compound groups cannot start\nwith numbers",
            )
        ),
        StoikError::UnpairedParenthesis(loc) => {
            println!("{}",
                loc.format_msg(formula, "Malformed formula", "Unpaired parenthesis")
            )
        }
        StoikError::UnpairedBracket(loc) => {
            println!("{}",
                loc.format_msg(formula, "Malformed formula", "Unpaired bracket")
            )
        }
        e => println!("{e}"),
    },
    Ok(mol) => {
        println!("{formula} contains:")
        //...
    }
}
```
