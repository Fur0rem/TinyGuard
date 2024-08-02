#[derive(Debug, PartialEq)]
enum VariableTypes {
	Bool,
	Number,
	String,
}

#[derive(Debug, PartialEq)]
enum Constant {
	Bool(bool),
	Number(i32),
	String(String),
}

#[derive(Debug, PartialEq)]
enum Identifier {
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

#[derive(Debug, PartialEq)]
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

#[derive(Debug, PartialEq)]
enum ArityAndTypes {
	Unary(VariableTypes),
	Binary(VariableTypes, VariableTypes),
}

impl Operator {
	fn arity_and_types(&self) -> ArityAndTypes {
		use Operator::*;
		use VariableTypes::*;
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

#[derive(Debug, PartialEq)]
enum Token {
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
			} else if c == '"' {
				let mut string = String::new();
				i += 1;
				while i < expr.len() && expr.chars().nth(i).unwrap() != '"' {
					string.push(expr.chars().nth(i).unwrap());
					i += 1;
				}
				tokens.push(Token::Operand(Identifier::Constant(Constant::String(string))));
				i += 1;
			} else if c.is_alphabetic() {
				let mut string = c.to_string();
				i += 1;
				while i < expr.len() && expr.chars().nth(i).unwrap().is_alphabetic() {
					string.push(expr.chars().nth(i).unwrap());
					i += 1;
				}
				tokens.push(Token::Operand(Identifier::Variable { name: string }));
			} else if c == '(' || c == ')' {
				tokens.push(Token::Parenthesis(c));
				i += 1;
			} else {
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

fn expr_tokens_to_rpn(tokens: Vec<Token>) -> Vec<Token> {
	let mut output = Vec::new();
	let mut stack = Vec::new();
	for token in tokens {
		match token {
			Token::Operand(_) => output.push(token),
			Token::Operation(ref op) => {
				while let Some(&ref top) = stack.last() {
					if let Token::Operation(top_op) = top {
						if top_op.priority() >= op.priority() {
							output.push(stack.pop().unwrap());
						} else {
							break;
						}
					} else {
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
					} else {
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

fn evaluate_rpn(tokens: Vec<Token>) -> Identifier {
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
}
