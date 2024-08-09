mod ast;
mod data_struct;
mod parser;

use std::fs::File;
use std::io::Read;

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
	println!("{:#?}", brack);

	let tree = ast::parse_ast("samples/main.c");
	ast::print_tree(tree.root_node(), &code, 0);
	let variables = ast::parse_variables(tree, &code);
	for variable in variables {
		ast::print_variable(&variable, &code);
	}
}
