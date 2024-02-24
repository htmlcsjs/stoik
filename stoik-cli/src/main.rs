use std::{
    collections::{hash_map::Entry, HashMap},
    env,
    time::Instant,
};

use stoik::{
    formula::{self, Molecule, TokenStream},
    StoikError,
};

const HELP_MSG: &str = include_str!("help_msg.txt");

fn main() {
    let mut time_mode = false;
    let mut all_moles = false;
    let mut equation = String::new();
    let mut time_table = ["Formula", "Tokenise", "Tree building", "Parsing", "Total"]
        .iter()
        .map(|x| vec![x.to_string()])
        .collect::<Vec<_>>();

    for arg in env::args().skip(1) {
        if arg == "--time" || arg == "-t" {
            time_mode = true;
        } else if arg == "--help" || arg == "-h" {
            println!("{}", HELP_MSG);
            return;
        } else if arg == "--all-moles" || arg == "-a" {
            all_moles = true;
        } else {
            equation = format!("{equation} {arg}");
        }
    }
    equation = equation.trim().to_string();

    if !(equation.contains("->") || equation.contains("=>")) {
        println!("Products are not given, please use `=>` or `->` to seperate the two sides");
        return;
    }

    let temp_equ = equation.replace("=>", "->");
    let (mut reactants_str, mut products_str) = temp_equ.split_once("->").unwrap();

    reactants_str = reactants_str.trim();
    products_str = products_str.trim();

    let mut reactants = Vec::new();
    for formula in reactants_str.split('+').map(|x| x.trim()) {
        match construct_mole(formula, time_mode, &mut time_table) {
            Ok(mol) => reactants.push((mol, formula.to_string())),
            Err(e) => {
                println!("{}", generate_error_msg(e, formula));
                return;
            }
        }
    }

    let mut products = Vec::new();
    for formula in products_str.split('+').map(|x| x.trim()) {
        match construct_mole(formula, time_mode, &mut time_table) {
            Ok(mol) => products.push((mol, formula.to_string())),
            Err(e) => {
                println!("{}", generate_error_msg(e, formula));
                return;
            }
        }
    }

    let mut lhs = HashMap::new();
    for mol in reactants {
        extend_mol_map(&mut lhs, mol.0.get_map());
    }

    let mut rhs = HashMap::new();
    for mol in products {
        extend_mol_map(&mut rhs, mol.0.get_map());
    }

    let mut keys = lhs.keys().collect::<Vec<_>>();
    let mut balanced = HashMap::new();
    keys.extend(rhs.keys());
    keys.dedup();

    for key in keys {
        balanced.insert(key.to_string(), lhs.get(key) == rhs.get(key));
    }

    let is_balanced = balanced.values().all(|x| *x);

    if is_balanced {
        println!("`{equation}` is balanced")
    } else {
        println!("`{equation}` is not balanced")
    }

    if !is_balanced || all_moles {
        let mut table = ["Element", "Reactants", "Products", "Balanced"]
            .iter()
            .map(|x| vec![x.to_string()])
            .collect::<Vec<_>>();
        for (element, bal) in balanced {
            if !bal || all_moles {
                table[1].push(lhs.get(&element).unwrap_or(&0).to_string());
                table[2].push(rhs.get(&element).unwrap_or(&0).to_string());
                table[0].push(element);
                table[3].push(bal.to_string());
            }
        }
        print_table(table);
    }

    if time_mode {
        println!("\nTime summary");
        print_table(time_table);
    }
}

// Move to a pub func from being copy pasted here and in stoik-gui/src/main.rs
fn generate_error_msg(e: StoikError, formula: &str) -> String {
    match e {
        StoikError::InvalidToken(loc) => {
            loc.format_msg(formula, "Malformed formula", "Illegal token")
        }
        StoikError::NumberFirst(loc) => loc.format_msg(
            formula,
            "Malformed formula",
            "Compound groups cannot start with numbers",
        ),
        StoikError::UnpairedParenthesis(loc) => {
            loc.format_msg(formula, "Malformed formula", "Unpaired parenthesis")
        }
        StoikError::UnpairedBracket(loc) => {
            loc.format_msg(formula, "Malformed formula", "Unpaired bracket")
        }
        e => e.to_string(),
    }
}

// same as generate_error_msg
fn extend_mol_map(main: &mut HashMap<String, i64>, mol: HashMap<String, i64>) {
    for (key, mol_val) in mol {
        if let Entry::Occupied(mut entry) = main.entry(key.clone()) {
            *entry.get_mut() += mol_val;
        } else {
            main.insert(key, mol_val);
        }
    }
}

// This is some *sus* code
// im too tired to write nice code for it
fn print_table(table: Vec<Vec<String>>) {
    let widths = table
        .iter()
        .map(|x| x.iter().max_by(|x, y| x.len().cmp(&y.len())).unwrap())
        .map(|x| x.len())
        .collect::<Vec<usize>>();
    println!(
        "{}",
        "╔═".to_string()
            + &widths
                .iter()
                .map(|x| "═".repeat(*x))
                .collect::<Vec<String>>()
                .join("═╦═")
            + "═╗"
    );
    println!(
        "{}",
        "║ ".to_string()
            + &table
                .iter()
                .enumerate()
                .map(|(pos, x)| pad_string(&x[0], widths[pos]))
                .collect::<Vec<String>>()
                .join(" ║ ")
            + " ║"
    );
    println!(
        "{}",
        "╠═".to_string()
            + &widths
                .iter()
                .map(|x| "═".repeat(*x))
                .collect::<Vec<String>>()
                .join("═╬═")
            + "═╣"
    );
    for i in 1..table[0].len() {
        println!(
            "{}",
            "║ ".to_string()
                + &table
                    .iter()
                    .enumerate()
                    .map(|(pos, x)| pad_string(&x[i], widths[pos]))
                    .collect::<Vec<String>>()
                    .join(" ║ ")
                + " ║"
        );
    }
    println!(
        "{}",
        "╚═".to_string()
            + &widths
                .iter()
                .map(|x| "═".repeat(*x))
                .collect::<Vec<String>>()
                .join("═╩═")
            + "═╝"
    );
}

fn pad_string(input: &String, length: usize) -> String {
    let pad_len = length - input.chars().count();
    input.to_string() + &" ".repeat(pad_len)
}

fn construct_mole(
    formula: &str,
    time_mode: bool,
    time_table: &mut [Vec<String>],
) -> Result<Molecule, StoikError> {
    if time_mode {
        time_table[0].push(formula.to_string());

        let tokenise_inst = Instant::now();
        let tokenstream = TokenStream::new(formula);
        time_table[1].push(format!("{:>09.3?}", tokenise_inst.elapsed()));

        let tree_inst = Instant::now();
        let root = formula::assemble_tree(tokenstream)?;
        time_table[2].push(format!("{:>09.3?}", tree_inst.elapsed()));

        let mol_inst = Instant::now();
        let mol = Molecule::construct_from_tree(root)?;
        time_table[3].push(format!("{:>09.3?}", mol_inst.elapsed()));

        time_table[4].push(format!("{:>09.3?}", tokenise_inst.elapsed()));
        Ok(mol)
    } else {
        Molecule::from_formula(formula)
    }
}
