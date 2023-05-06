mod err;
mod formula;

use std::{env, time::Instant};

use crate::{
    err::StoikError,
    formula::{assemble_tree, Molecule, TokenStream},
};

fn main() {
    // troll Na15[Mo126Mo28O462H14(H2O)70][Mo124Mo28O457H14(H2O)68]

    let total_inst = Instant::now();

    let args: Vec<String> = env::args().skip(1).collect();
    let mut streams = Vec::new();

    let stream_inst = Instant::now();
    for i in args.iter() {
        streams.push(TokenStream::new(i).collect::<Vec<_>>().into_iter());
    }
    println!("Took {:.3?} to tokenise", stream_inst.elapsed());

    let mut trees = Vec::new();
    let tree_inst = Instant::now();
    for stream in streams {
        trees.push(assemble_tree(stream));
    }
    println!("Took {:.3?} to build trees", tree_inst.elapsed());

    let filtered_trees = trees
        .into_iter()
        .enumerate()
        .filter_map(|(i, tree)| match tree {
            Ok(tree) => {
                #[cfg(debug_assertions)]
                println!("{}: Syntax tree built", &args[i]);
                Some(tree)
            }
            Err(e) => {
                match e {
                    StoikError::InvalidToken(loc) => {
                        loc.print_msg(&args[i], "Malformed formula", "Illegal token")
                    }
                    StoikError::NumberFirst(loc) => loc.print_msg(
                        &args[i],
                        "Malformed formula",
                        "Compound groups cannot start\nwith numbers",
                    ),
                    StoikError::UnpairedParenthesis(loc) => {
                        loc.print_msg(&args[i], "Malformed formula", "Unpaired parenthesis")
                    }
                    StoikError::UnpairedBracket(loc) => {
                        loc.print_msg(&args[i], "Malformed formula", "Unpaired bracket")
                    }
                    e => println!("{e}"),
                };
                None
            }
        })
        .collect::<Vec<_>>();

    let mut molecules = Vec::new();
    let molar_inst = Instant::now();
    for tree in filtered_trees {
        molecules.push(Molecule::construct_from_tree(tree));
    }
    println!("Took {:.3?} to build molecules", molar_inst.elapsed());

    for (i, mole) in molecules.into_iter().enumerate() {
        match mole {
            Ok(mole) => println!("{} -> {mole:?}", args[i]),
            Err(e) => println!("Error with {}: {e}", args[i]),
        }
    }
    println!("Took {:.3?} in total", total_inst.elapsed());
}
