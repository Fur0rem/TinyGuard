use either::Either;

use crate::{ast::ProgramVariable, parser::Bracketed};

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum VariableType {
	Bool,
	Number,
	String,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Constant {
	Bool(bool),
	Number(i32),
	String(String),
}

#[derive(Debug, PartialEq, Clone)]
pub enum Identifier {
	Constant(Constant),
	Variable { name: String },
}

impl Identifier {
	fn var(name: &str) -> Identifier {
		Identifier::Variable { name: name.to_string() }
	}

	fn const_bool(b: bool) -> Identifier {
		Identifier::Constant(Constant::Bool(b))
	}

	fn const_num(n: i32) -> Identifier {
		Identifier::Constant(Constant::Number(n))
	}

	fn const_str(s: &str) -> Identifier {
		Identifier::Constant(Constant::String(s.to_string()))
	}
}

#[derive(Debug, PartialEq, Copy, Clone)]
enum Operator {
	Addition,
	Substraction,
	Multiplication,
	Division,
	UnaryMinus,
	Equals,
	Not,
	And,
	Or,
}

#[derive(Debug, PartialEq, Copy, Clone)]
enum ArityAndTypes {
	Unary(VariableType),
	Binary(VariableType, VariableType),
}

impl Operator {
	fn arity_and_types(&self) -> ArityAndTypes {
		use Operator::*;
		use VariableType::*;
		match self {
			Addition | Substraction | Multiplication | Division => ArityAndTypes::Binary(Number, Number),
			UnaryMinus => ArityAndTypes::Unary(Number),
			Equals => ArityAndTypes::Binary(Number, Number),
			Not => ArityAndTypes::Unary(Bool),
			And | Or => ArityAndTypes::Binary(Bool, Bool),
		}
	}

	fn priority(&self) -> i32 {
		use Operator::*;
		match self {
			Addition | Substraction => 100,
			UnaryMinus => 300,
			Not => 300,
			Multiplication | Division => 200,
			Equals => 50,
			And => 25,
			Or => 20,
		}
	}

	fn from_string(op: &str) -> Result<Operator, String> {
		use Operator::*;
		match op {
			"+" => Ok(Addition),
			"-" => Ok(Substraction),
			"*" => Ok(Multiplication),
			"/" => Ok(Division),
			"==" => Ok(Equals),
			"!" => Ok(Not),
			"&&" => Ok(And),
			"||" => Ok(Or),
			_ => Err(format!("Unknown operator: {}", op)),
		}
	}
}

#[derive(Debug, PartialEq, Clone)]
pub enum Token {
	Parenthesis(char),
	Operand(Identifier),
	Operation(Operator),
}

impl Token {
	fn is_operand(&self) -> bool {
		match self {
			Token::Operand(_) => true,
			_ => false,
		}
	}

	fn parse(expr: &str) -> Vec<Token> {
		// remove all whitespaces
		let expr = expr.replace(" ", "");
		let mut tokens = Vec::new();
		let mut i = 0;
		while i < expr.len() {
			let c = expr.chars().nth(i).unwrap();
			if c.is_digit(10) {
				let mut number = c.to_string();
				i += 1;
				while i < expr.len() && expr.chars().nth(i).unwrap().is_digit(10) {
					number.push(expr.chars().nth(i).unwrap());
					i += 1;
				}
				tokens.push(Token::Operand(Identifier::Constant(Constant::Number(number.parse().unwrap()))));
			}
			else if c == '"' {
				let mut string = String::new();
				i += 1;
				while i < expr.len() && expr.chars().nth(i).unwrap() != '"' {
					string.push(expr.chars().nth(i).unwrap());
					i += 1;
				}
				tokens.push(Token::Operand(Identifier::Constant(Constant::String(string))));
				i += 1;
			}
			else if c.is_alphabetic() {
				let mut string = c.to_string();
				i += 1;
				while i < expr.len() && expr.chars().nth(i).unwrap().is_alphabetic() {
					string.push(expr.chars().nth(i).unwrap());
					i += 1;
				}
				tokens.push(Token::Operand(Identifier::Variable { name: string }));
			}
			else if c == '(' || c == ')' {
				tokens.push(Token::Parenthesis(c));
				i += 1;
			}
			else {
				let mut op = c.to_string();
				let begin = i;
				let mut end = i + 1;
				while end < expr.len() {
					match Operator::from_string(expr[begin..end].as_ref()) {
						Ok(_) => {
							op = expr[begin..end].to_string();
							break;
						}
						Err(_) => {
							end += 1;
						}
					}
				}
				match Operator::from_string(&op) {
					Ok(op) => tokens.push(Token::Operation(op)),
					Err(e) => panic!("{}", e),
				}
				i = end;
			}
		}
		return tokens;
	}

	fn from_string(expr: &str) -> Vec<Token> {
		let mut tokens = Token::parse(expr);
		let mut i = 0;
		while i < tokens.len() {
			if let Token::Operation(Operator::Substraction) = tokens[i] {
				if i == 0 || !tokens[i - 1].is_operand() {
					tokens[i] = Token::Operation(Operator::UnaryMinus);
				}
			}
			i += 1;
		}
		return tokens;
	}
}

pub type EvaluableExpr = Vec<Token>;

fn expr_tokens_to_rpn(tokens: Vec<Token>) -> EvaluableExpr {
	let mut output = Vec::new();
	let mut stack = Vec::new();
	for token in tokens {
		match token {
			Token::Operand(x) => match x {
				Identifier::Constant(_) => output.push(Token::Operand(x)),
				Identifier::Variable { name } => match name.as_str() {
					"true" => output.push(Token::Operand(Identifier::const_bool(true))),
					"false" => output.push(Token::Operand(Identifier::const_bool(false))),
					_ => output.push(Token::Operand(Identifier::Variable { name })),
				},
			},
			Token::Operation(ref op) => {
				while let Some(&ref top) = stack.last() {
					if let Token::Operation(top_op) = top {
						if top_op.priority() >= op.priority() {
							output.push(stack.pop().unwrap());
						}
						else {
							break;
						}
					}
					else {
						break;
					}
				}
				stack.push(token);
			}
			Token::Parenthesis('(') => stack.push(token),
			Token::Parenthesis(')') => {
				while let Some(top) = stack.pop() {
					if let Token::Parenthesis('(') = top {
						break;
					}
					else {
						output.push(top);
					}
				}
			}
			_ => panic!("Unknown token {:?}", token),
		}
	}
	while let Some(top) = stack.pop() {
		output.push(top);
	}
	return output;
}

fn evaluate_rpn(tokens: EvaluableExpr) -> Identifier {
	let mut stack = Vec::new();
	for token in tokens {
		match token {
			Token::Operand(operand) => stack.push(operand),
			Token::Operation(op) => {
				let arity_and_types = op.arity_and_types();
				match arity_and_types {
					ArityAndTypes::Unary(_) => {
						let operand = stack.pop().unwrap();
						match (&op, &operand) {
							(Operator::UnaryMinus, Identifier::Constant(Constant::Number(n))) => {
								stack.push(Identifier::Constant(Constant::Number(-n)))
							}
							(Operator::Not, Identifier::Constant(Constant::Bool(b))) => {
								stack.push(Identifier::Constant(Constant::Bool(!b)))
							}
							_ => panic!("Invalid unary operation {:?} {:?}", op, operand),
						}
					}
					ArityAndTypes::Binary(_, _) => {
						let operand2 = stack.pop().unwrap();
						let operand1 = stack.pop().unwrap();
						match (&op, &operand1, &operand2) {
							(
								Operator::Addition,
								Identifier::Constant(Constant::Number(n1)),
								Identifier::Constant(Constant::Number(n2)),
							) => stack.push(Identifier::Constant(Constant::Number(n1 + n2))),
							(
								Operator::Substraction,
								Identifier::Constant(Constant::Number(n1)),
								Identifier::Constant(Constant::Number(n2)),
							) => stack.push(Identifier::Constant(Constant::Number(n1 - n2))),
							(
								Operator::Multiplication,
								Identifier::Constant(Constant::Number(n1)),
								Identifier::Constant(Constant::Number(n2)),
							) => stack.push(Identifier::Constant(Constant::Number(n1 * n2))),
							(
								Operator::Division,
								Identifier::Constant(Constant::Number(n1)),
								Identifier::Constant(Constant::Number(n2)),
							) => stack.push(Identifier::Constant(Constant::Number(n1 / n2))),
							(Operator::Equals, Identifier::Constant(Constant::Number(n1)), Identifier::Constant(Constant::Number(n2))) => {
								stack.push(Identifier::Constant(Constant::Bool(n1 == n2)))
							}
							(Operator::And, Identifier::Constant(Constant::Bool(b1)), Identifier::Constant(Constant::Bool(b2))) => {
								stack.push(Identifier::Constant(Constant::Bool(*b1 && *b2)))
							}
							(Operator::Or, Identifier::Constant(Constant::Bool(b1)), Identifier::Constant(Constant::Bool(b2))) => {
								stack.push(Identifier::Constant(Constant::Bool(*b1 || *b2)))
							}

							_ => panic!("Invalid binary operation {:?} {:?} {:?}", op, operand1, operand2),
						}
					}
				}
			}
			_ => panic!("Invalid token {:?}", token),
		}
	}
	return stack.pop().unwrap();
}

pub struct MetaData {
	pub name: String,
	pub data: Constant,
}

impl MetaData {
	pub fn from_string(s: &str) -> MetaData {
		// name: type = value
		let parts: Vec<&str> = s.split(":").collect();
		let name = parts[0].trim().to_string();
		let parts: Vec<&str> = parts[1].split("=").collect();
		let data_type = parts[0].trim();
		let data = parts[1].trim();
		let data = if data.starts_with('"') {
			Constant::String(data[1..data.len() - 1].to_string())
		}
		else if data == "true" {
			Constant::Bool(true)
		}
		else if data == "false" {
			Constant::Bool(false)
		}
		else {
			Constant::Number(data.parse().unwrap())
		};
		if data_type == "bool" && !matches!(data, Constant::Bool(_)) {
			panic!("Invalid data type for {}: {:?}", name, data);
		}
		if data_type == "number" && !matches!(data, Constant::Number(_)) {
			panic!("Invalid data type for {}: {:?}", name, data);
		}
		if data_type == "string" && !matches!(data, Constant::String(_)) {
			panic!("Invalid data type for {}: {:?}", name, data);
		}
		return MetaData { name, data };
	}
}

#[derive(Debug, PartialEq, Clone)]
pub struct Assignement {
	pub name: String,
	pub value: EvaluableExpr,
}

impl Assignement {
	pub fn from_string(s: &str) -> Assignement {
		let parts: Vec<&str> = s.split("=").collect();
		let name = parts[0].trim().to_string();
		let value = parts[1].trim();
		let tokens = Token::from_string(value);
		let rpn = expr_tokens_to_rpn(tokens);
		return Assignement { name, value: rpn };
	}
}

#[derive(Debug, PartialEq, Clone)]
pub struct Warning {
	pub test: EvaluableExpr,
	pub message: Option<String>,
	pub hint: Option<String>,
}

impl Warning {
	/*Format:
	{
		Test: "a + 5 == c",
		Message: "a + 5 should be equal to c",
		Hint: "Check the value of a and c"
	}
	Message and Hints are optional
	*/
	pub fn from_string(s: &str) -> Warning {
		// let parts: Vec<&str> = s.split(",").collect();
		let mut parts = Vec::new();
		// find all the commas, if they are inside a string, ignore them
		let mut start = 0;
		let mut inside_string = false;
		for (i, c) in s.chars().enumerate() {
			if c == '"' {
				inside_string = !inside_string;
			}
			if c == ',' && !inside_string {
				parts.push(&s[start..i]);
				start = i + 1;
			}
		}
		parts.push(&s[start..]);

		let mut test = None;
		let mut message = None;
		let mut hint = None;
		for part in &parts {
			let part = part.trim();
			if part.starts_with("Test") {
				let split = part.split(":").collect::<Vec<&str>>();
				let test_expr = split[1].trim();
				let tokens = Token::from_string(test_expr);
				let rpn = expr_tokens_to_rpn(tokens);
				println!("RPN: {:#?}", rpn);
				if test.is_some() {
					panic!("Multiple tests in warning");
				}
				test = Some(rpn);
			}
			if part.starts_with("Message") {
				let split = part.split(":").collect::<Vec<&str>>();
				if message.is_some() {
					panic!("Multiple messages in warning");
				}
				message = Some(split[1].trim().to_string());
				message = Some(message.as_ref().unwrap()[1..message.as_ref().unwrap().len() - 1].to_string());
			}
			if part.starts_with("Hint") {
				let split = part.split(":").collect::<Vec<&str>>();
				if hint.is_some() {
					panic!("Multiple hints in warning");
				}
				hint = Some(split[1].trim().to_string());
				// remove the first " and last "
				hint = Some(hint.as_ref().unwrap()[1..hint.as_ref().unwrap().len() - 1].to_string());
			}
		}
		println!("TEST: {:#?}", test);
		println!("MESSAGE: {:#?}", message);
		println!("HINT: {:#?}", hint);
		return Warning {
			test: test.unwrap(),
			message,
			hint,
		};
	}
}

pub type Deny = Warning;

pub struct Call {
	pub name: String,
	pub does: Option<Vec<Assignement>>,
	pub warn: Option<Vec<Warning>>,
	pub deny: Option<Vec<Deny>>,
}

impl Call {
	/*
	IntVector_push_back(&$self, ...) {
			Does {
				sorted = false
			}
		}
	 */
	/*
		IntVector_search(&$self, ...) {
			Warn {
				{
					Test: sorted == true,
					Message: "Using search on an sorted vector is very inefficient",
					Hint: "Use IntVector_binary_search(&$self, ...) instead"
				}
			}
		}
	*/
	pub fn from_bracketed(bracketed: Bracketed) -> Call {
		println!(" FROM BRACKETED ");
		let mut does = None;
		let mut warn = None;
		let mut deny = None;
		if let Either::Right(v) = bracketed.content {
			for b in &v {
				match b.beginning_line.line.trim() {
					bs if bs.starts_with("Does") => {
						let mut does_vec = Vec::new();
						let content = &b.content;
						match content {
							Either::Right(v) => {
								panic!("Does content is not a string");
							}
							Either::Left(s) => {
								let lines = s.lines().map(|line| line.to_string()).collect::<Vec<_>>();
								for line in lines {
									does_vec.push(Assignement::from_string(&line));
								}
							}
						}
						does = Some(does_vec);
					}
					bs if bs.starts_with("Warn") => {
						let mut warn_vec = Vec::new();
						match &b.content {
							Either::Right(v) => {
								for b in v {
									for x in v {
										match &x.content {
											Either::Right(v) => {
												panic!("Warn content is not a string");
											}
											Either::Left(s) => {
												warn_vec.push(Warning::from_string(s));
											}
										}
									}
								}
							}
							Either::Left(s) => panic!("Warn content is not a string"),
						}
						warn = Some(warn_vec);
					}
					_ => panic!("Unknown bracketed content: {:?}", b.beginning_line.line),
				}
			}
		}
		return Call {
			name: bracketed.beginning_line.line.clone(),
			does,
			warn,
			deny,
		};
	}
}

pub struct DataStruct {
	pub name: String,
	pub constructors: Vec<String>,
	pub destructors: Vec<String>,
	pub meta_data: Vec<MetaData>,
	pub calls: Vec<Call>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Environment {
	pub assocs: Vec<(String, Identifier)>,
}

impl Environment {
	pub fn new() -> Environment {
		Environment { assocs: Vec::new() }
	}

	pub fn evaluate_assignement(&mut self, assignement: Assignement) {
		let result = self.evaluate_rpn(assignement.value);
		for i in 0..self.assocs.len() {
			if self.assocs[i].0 == assignement.name {
				self.assocs[i].1 = result;
				return;
			}
		}
		self.assocs.push((assignement.name, result));
	}

	pub fn fetch(&self, name: &str) -> Identifier {
		for (n, id) in &self.assocs {
			if n == name {
				return id.clone();
			}
		}
		panic!("Variable {} not found", name);
	}

	pub fn evaluate_rpn(&mut self, tokens: EvaluableExpr) -> Identifier {
		let mut stack = Vec::new();
		for token in tokens {
			match token {
				Token::Operand(operand) => stack.push(operand),
				Token::Operation(op) => {
					let arity_and_types = op.arity_and_types();
					println!("op:{:?} stack:{:?} arity:{:?}", op, stack, arity_and_types);
					match arity_and_types {
						ArityAndTypes::Unary(_) => {
							let operand = stack.pop().unwrap();
							match (&op, &operand) {
								(Operator::UnaryMinus, Identifier::Constant(Constant::Number(n))) => {
									stack.push(Identifier::Constant(Constant::Number(-n)))
								}
								(Operator::Not, Identifier::Constant(Constant::Bool(b))) => {
									stack.push(Identifier::Constant(Constant::Bool(!b)))
								}
								_ => panic!("Invalid unary operation {:?} {:?}", op, operand),
							}
						}
						ArityAndTypes::Binary(_, _) => {
							let mut operand2 = stack.pop().unwrap();
							let mut operand1 = stack.pop().unwrap();

							if let Identifier::Variable { name } = operand1 {
								operand1 = self.fetch(&name);
							}
							if let Identifier::Variable { name } = operand2 {
								operand2 = self.fetch(&name);
							}

							match (&op, &operand1, &operand2) {
								(
									Operator::Addition,
									Identifier::Constant(Constant::Number(n1)),
									Identifier::Constant(Constant::Number(n2)),
								) => stack.push(Identifier::Constant(Constant::Number(n1 + n2))),
								(
									Operator::Substraction,
									Identifier::Constant(Constant::Number(n1)),
									Identifier::Constant(Constant::Number(n2)),
								) => stack.push(Identifier::Constant(Constant::Number(n1 - n2))),
								(
									Operator::Multiplication,
									Identifier::Constant(Constant::Number(n1)),
									Identifier::Constant(Constant::Number(n2)),
								) => stack.push(Identifier::Constant(Constant::Number(n1 * n2))),
								(
									Operator::Division,
									Identifier::Constant(Constant::Number(n1)),
									Identifier::Constant(Constant::Number(n2)),
								) => stack.push(Identifier::Constant(Constant::Number(n1 / n2))),
								(
									Operator::Equals,
									Identifier::Constant(Constant::Number(n1)),
									Identifier::Constant(Constant::Number(n2)),
								) => stack.push(Identifier::Constant(Constant::Bool(n1 == n2))),
								(Operator::And, Identifier::Constant(Constant::Bool(b1)), Identifier::Constant(Constant::Bool(b2))) => {
									stack.push(Identifier::Constant(Constant::Bool(*b1 && *b2)))
								}
								(Operator::Or, Identifier::Constant(Constant::Bool(b1)), Identifier::Constant(Constant::Bool(b2))) => {
									stack.push(Identifier::Constant(Constant::Bool(*b1 || *b2)))
								}

								_ => panic!("Invalid binary operation {:?} {:?} {:?}", op, operand1, operand2),
							}
						}
					}
				}
				_ => panic!("Invalid token {:?}", token),
			}
		}
		return stack.pop().unwrap();
	}
}

#[derive(Debug, Clone, PartialEq)]
pub struct Program {
	pub checks: Vec<(ProgramVariable, Environment)>,
}

impl Program {
	pub fn from_vars(variables: Vec<ProgramVariable>) -> Program {
		let mut v = Vec::new();
		for variable in variables {
			v.push((variable, Environment::new()));
		}
		return Program { checks: v };
	}
}

fn print_tokens(tokens: Vec<Token>) {
	for token in tokens {
		match token {
			Token::Operand(Identifier::Constant(c)) => match c {
				Constant::Bool(b) => print!("{} ", b),
				Constant::Number(n) => print!("{} ", n),
				Constant::String(s) => print!("{} ", s),
			},
			Token::Operand(Identifier::Variable { name }) => print!("{} ", name),
			Token::Operation(op) => print!("{:?} ", op),
			Token::Parenthesis(p) => print!("{:?} ", p),
		}
		print!(" ");
	}
	println!();
}

#[cfg(test)]
mod tests {
	use crate::parser::{fill_blanks, parse_bracketed, Bracketed};

	use super::*;

	#[test]
	fn test_tokenise() {
		let expression = "!((a+5)==c)";
		let tokens = Token::from_string(expression);
		assert_eq!(
			tokens,
			vec![
				Token::Operation(Operator::Not),
				Token::Parenthesis('('),
				Token::Parenthesis('('),
				Token::Operand(Identifier::var("a")),
				Token::Operation(Operator::Addition),
				Token::Operand(Identifier::const_num(5)),
				Token::Parenthesis(')'),
				Token::Operation(Operator::Equals),
				Token::Operand(Identifier::var("c")),
				Token::Parenthesis(')'),
			]
		);
	}

	#[test]
	fn test_unary_minus_begin() {
		let expression = "-5";
		let tokens = Token::from_string(expression);
		assert_eq!(
			tokens,
			vec![Token::Operation(Operator::UnaryMinus), Token::Operand(Identifier::const_num(5)),]
		);
	}

	#[test]
	fn test_unary_minus_in_expression() {
		let expression = "5*-5";
		let tokens = Token::from_string(expression);
		assert_eq!(
			tokens,
			vec![
				Token::Operand(Identifier::const_num(5)),
				Token::Operation(Operator::Multiplication),
				Token::Operation(Operator::UnaryMinus),
				Token::Operand(Identifier::const_num(5)),
			]
		);
	}

	#[test]
	fn test_tokenise_variables() {
		let expression = "a + 5 + b";
		let tokens = Token::from_string(expression);
		assert_eq!(
			tokens,
			vec![
				Token::Operand(Identifier::var("a")),
				Token::Operation(Operator::Addition),
				Token::Operand(Identifier::const_num(5)),
				Token::Operation(Operator::Addition),
				Token::Operand(Identifier::var("b")),
			]
		);
	}

	#[test]
	fn test_expr_tokens_to_rpn() {
		let tokens = vec![
			Token::Operand(Identifier::const_num(5)),
			Token::Operation(Operator::Addition),
			Token::Operand(Identifier::const_num(3)),
			Token::Operation(Operator::Multiplication),
			Token::Operand(Identifier::const_num(2)),
		];
		let rpn = expr_tokens_to_rpn(tokens);
		assert_eq!(
			rpn,
			vec![
				Token::Operand(Identifier::const_num(5)),
				Token::Operand(Identifier::const_num(3)),
				Token::Operand(Identifier::const_num(2)),
				Token::Operation(Operator::Multiplication),
				Token::Operation(Operator::Addition),
			]
		);
	}

	#[test]
	fn test_evaluate() {
		let expr = "5 + 3 * 2";
		let tokens = Token::from_string(expr);
		let rpn = expr_tokens_to_rpn(tokens);
		let result = evaluate_rpn(rpn);
		assert_eq!(result, Identifier::const_num(11));
	}

	#[test]
	fn test_parse_meta_data() {
		let meta_data = MetaData::from_string("name: number = 5");
		assert_eq!(meta_data.name, "name");
		assert_eq!(meta_data.data, Constant::Number(5));
	}

	#[test]
	fn test_parse_call() {
		// let call = Call::fr("IntVector_push_back(&$self, ...) { Does { sorted = false } }");
		let brack = parse_bracketed("IntVector_push_back(&$self, ...) {\n Does {\n sorted = false\n }\n }\n");
		let mut brack = brack[0].clone();
		println!("BRACK BEFORE {:?}", &brack);
		brack = fill_blanks(&brack, "IntVector_push_back(&$self, ...) {\n Does {\n sorted = false\n }\n }\n");
		println!("BRACK AFTER {:?}", &brack);
		let call = Call::from_bracketed(brack);
		assert_eq!(call.name, "IntVector_push_back(&$self, ...) {");
		assert_eq!(call.does, Some(vec![Assignement::from_string("sorted = false")]));

		let warned = "        IntVector_search(&$self, ...) {
            Warn { 
                {
                    Test: sorted == true,
                    Message: \"Using search on an sorted vector is very inefficient\",
                    Hint: \"Use IntVector_binary_search(&$self, ...) instead\",
                }
            }
        }";
		let brack = parse_bracketed(warned);
		let mut brack = brack[0].clone();
		brack = fill_blanks(&brack, warned);
		println!("BRACK WARNED {:#?}", &brack);
		let call = Call::from_bracketed(brack);
		assert_eq!(call.name, "IntVector_search(&$self, ...) {");
		assert_eq!(
			call.warn,
			Some(vec![Warning {
				test: vec![
					Token::Operand(Identifier::Variable {
						name: "sorted".to_string()
					}),
					Token::Operand(Identifier::const_bool(true)),
					Token::Operation(Operator::Equals),
				],
				message: Some("Using search on an sorted vector is very inefficient".to_string()),
				hint: Some("Use IntVector_binary_search(&$self, ...) instead".to_string()),
			}])
		);
	}

	#[test]
	fn test_parse_assignement() {
		let assignement = Assignement::from_string("name = 5 + 3 * 2");
		assert_eq!(assignement.name, "name");
		assert_eq!(
			assignement.value,
			vec![
				Token::Operand(Identifier::const_num(5)),
				Token::Operand(Identifier::const_num(3)),
				Token::Operand(Identifier::const_num(2)),
				Token::Operation(Operator::Multiplication),
				Token::Operation(Operator::Addition),
			]
		);
	}

	#[test]
	fn test_parse_warning() {
		let warning =
			Warning::from_string("Test: a + 5 == c, Message: \"a + 5 should be equal to c\", Hint: \"Check the value of a and c\"");
		assert_eq!(
			warning.test,
			vec![
				Token::Operand(Identifier::Variable { name: "a".to_string() }),
				Token::Operand(Identifier::const_num(5)),
				Token::Operation(Operator::Addition),
				Token::Operand(Identifier::Variable { name: "c".to_string() }),
				Token::Operation(Operator::Equals),
			]
		);
		assert_eq!(warning.message, Some("a + 5 should be equal to c".to_string()));
		assert_eq!(warning.hint, Some("Check the value of a and c".to_string()));
	}

	#[test]
	fn test_parse_does() {
		let does = Assignement::from_string("name = 5 + 3 * 2");
		assert!(does.name == "name");
		assert_eq!(
			does.value,
			vec![
				Token::Operand(Identifier::const_num(5)),
				Token::Operand(Identifier::const_num(3)),
				Token::Operand(Identifier::const_num(2)),
				Token::Operation(Operator::Multiplication),
				Token::Operation(Operator::Addition),
			]
		);
	}

	#[test]
	fn test_evaluate_env() {
		let mut env = Environment::new();
		let expr_1 = "a = 5";
		let tokens_1 = Assignement::from_string(expr_1);
		let expr_2 = "b = 3";
		let tokens_2 = Assignement::from_string(expr_2);
		let expr_3 = "a == (b + 2)";
		let tokens_3 = Token::from_string(expr_3);
		let rpn = expr_tokens_to_rpn(tokens_3);
		env.evaluate_assignement(tokens_1);
		env.evaluate_assignement(tokens_2);
		let result = env.evaluate_rpn(rpn);
		assert_eq!(result, Identifier::const_bool(true));
	}
}
