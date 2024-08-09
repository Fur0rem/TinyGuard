use tree_sitter::{self, Language, Parser, Tree};
use tree_sitter_c;

pub struct ProgramVariable {
	name: String,
	var_type: String,
	implications: Vec<(usize, usize)>, // (start_byte, end_byte)
	scope: (usize, usize),             // (start_byte, end_byte)
}

pub fn parse_ast(file: &str) -> Tree {
	let source_code = std::fs::read_to_string(file).unwrap();
	let language = Language::from(tree_sitter_c::language());
	let mut parser = Parser::new();
	parser.set_language(&language).unwrap();
	let tree = parser.parse(&source_code, None).unwrap();
	return tree;
}

pub fn print_tree(node: tree_sitter::Node, source_code: &str, depth: usize) {
	let start_byte = node.start_byte();
	let end_byte = node.end_byte();
	let start_position = node.start_position();
	let end_position = node.end_position();
	let kind = node.kind();
	let kind_id = node.kind_id();
	let is_named = node.is_named();
	let has_error = node.has_error();
	let child_count = node.child_count();
	let is_child = child_count > 0;

	let indent = "  ".repeat(depth);
	if is_child {
		println!(
			"{}{}: {}..{} ({}..{})",
			indent, kind, start_byte, end_byte, start_position, end_position
		);

		if is_named {
			let text = &source_code[start_byte..end_byte];
			println!("{}  text: {:?}", indent, text);
		}
	}

	for i in 0..child_count {
		let child = node.child(i).unwrap();
		print_tree(child, source_code, depth + 1);
	}
}

pub fn parse_variables(tree: Tree, source_code: &str) -> Vec<ProgramVariable> {
	let mut variables = Vec::new();
	let root_node = tree.root_node();
	let mut stack = Vec::new();
	stack.push(root_node);

	while let Some(node) = stack.pop() {
		let kind = node.kind();
		if kind == "declaration" {
			let mut variable = ProgramVariable {
				name: "".to_string(),
				var_type: "".to_string(),
				implications: Vec::new(),
				scope: (node.parent().unwrap().start_byte(), node.parent().unwrap().end_byte()),
			};
			let text = &source_code[node.start_byte()..node.end_byte()];

			let mut found_init_declarator = false;
			for i in 0..node.child_count() {
				let child = node.child(i).unwrap();
				let child_kind = child.kind();
				if child_kind == "init_declarator" {
					found_init_declarator = true;
					println!("init_declarator : {:?}", child);
					// find the call expression if it exists
					let mut found_call_expression = false;
					for j in 0..child.child_count() {
						let grandchild = child.child(j).unwrap();
						let grandchild_kind = grandchild.kind();
						if grandchild_kind == "call_expression" {
							found_call_expression = true;
							let mut call_expression_start_byte = grandchild.start_byte();
							// Type var = call_expression();
							let expr = &source_code[child.start_byte()..grandchild.start_byte()];
							let equal_sign = expr.rfind('=').unwrap();
							println!("expr : {:?}", expr);
							println!("text : {:?}", text);
							variable.name = expr.trim().to_string();
							// find the type by looking at the name
							let index_of_name = text.find(&variable.name).unwrap();
							println!("index_of_name : {:?}", index_of_name);
							let var_type = &source_code[node.start_byte()..node.start_byte() + index_of_name];
							variable.var_type = var_type.trim().to_string();

							variable.implications.push((call_expression_start_byte, grandchild.end_byte()));
						}
					}
					if !found_call_expression {
						let text = &source_code[node.start_byte()..node.end_byte()];
						println!("text not found : {:?}", text);
						let first_non_alphanumeric = text.find(|c: char| !(c.is_alphanumeric() || c == '_')).unwrap();
						let next_alphanumeric = text[first_non_alphanumeric..]
							.find(|c: char| c.is_alphanumeric() || c == '_')
							.unwrap();
						println!("first_non_alphanumeric : {:?}", first_non_alphanumeric);
						println!("next_alphanumeric : {:?}", next_alphanumeric);
						println!("text non_alphanumeric : {:?}", &text[first_non_alphanumeric..]);
						println!("text alphanumeric : {:?}", &text[..first_non_alphanumeric + next_alphanumeric]);
						let var_type = &text[..first_non_alphanumeric + next_alphanumeric];
						println!("var_type : {:?}", var_type);
						variable.var_type = var_type.trim().to_string();
						let equals_sign = text.find('=');
						variable.name = text[first_non_alphanumeric + next_alphanumeric..equals_sign.unwrap()]
							.trim()
							.to_string();
						variable.implications.push((node.start_byte(), node.end_byte()));
					}
				}
			}
			if !found_init_declarator {
				let text = &source_code[node.start_byte()..node.end_byte()];
				println!("declaration : {:?}", text);
				let last_space = text.rfind(|c: char| !c.is_alphanumeric() && c.is_whitespace()).unwrap();
				variable.name = text[last_space..text.len() - 1].trim().to_string();
				let var_type = &source_code[node.start_byte()..node.start_byte() + last_space];
				variable.var_type = var_type.trim().to_string();
			}
			variables.push(variable);
		}
		else {
			for i in 0..node.child_count() {
				stack.push(node.child(i).unwrap());
			}
		}
	}
	return variables;
}

pub fn print_variables(variable: &ProgramVariable, source_code: &str) {
	println!("{}: {} {:?}", variable.name, variable.var_type, variable.scope);
	for (start_byte, end_byte) in &variable.implications {
		let text = &source_code[*start_byte..*end_byte];
		println!("  Implication: {}", text);
	}
}
