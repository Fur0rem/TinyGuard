mod ast;
mod data_struct;
mod parser;

use std::fs::File;
use std::io::Read;

use ast::entry_point_c;
use data_struct::Program;
use parser::parse_bracketed;
fn read_file(file_path: &str) -> String {
	let mut file = File::open(file_path).expect("File not found");
	let mut contents = String::new();
	file.read_to_string(&mut contents).expect("Failed to read file");
	return contents;
}

fn main() {
	let code = read_file("samples/main.c");
	let guard_file = read_file("samples/vector.tngd");
	let brack = parse_bracketed(&guard_file);
	let brack = brack
		.iter()
		.map(|brack| parser::fill_blanks(brack, &guard_file))
		.collect::<Vec<_>>();
	println!("BRACK : {:#?}", brack);

	let tree = ast::parse_ast("samples/main.c");
	ast::print_tree(tree.root_node(), &code, 0);
	let variables = ast::parse_variables(&tree, &code);
	for variable in &variables {
		ast::print_variables(variable, &code);
	}
	let entry_point = entry_point_c(&tree, &code);
	println!(
		"ENTRY POINT: {:?}, {}",
		entry_point,
		code[entry_point.start_byte()..entry_point.end_byte()].to_string()
	);

	for variable in &variables {
		println!("variable: {}", variable.name);
		for impli in &variable.implications {
			if impli.0 >= entry_point.start_byte() && entry_point.end_byte() >= impli.1 {
				println!("impli: {}", code[impli.0..impli.1].to_string());
			}
		}
	}

	let program = Program::from_vars(variables);
	println!("PROGRAM: {:#?}", program);
}
