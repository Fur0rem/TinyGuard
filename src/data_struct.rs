#[derive(Debug, PartialEq)]
enum VariableTypes {
	Bool,
	Number,
	String,
}

#[derive(Debug, PartialEq)]
enum Variable {
	Bool(bool),
	Number(i32),
	String(String),
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
	Operand(Variable),
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
				tokens.push(Token::Operand(Variable::Number(number.parse().unwrap())));
			} else if c.is_alphabetic() {
				let mut string = c.to_string();
				i += 1;
				while i < expr.len() && expr.chars().nth(i).unwrap().is_alphabetic() {
					string.push(expr.chars().nth(i).unwrap());
					i += 1;
				}
				tokens.push(Token::Operand(Variable::String(string)));
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

fn evaluate_rpn(tokens: Vec<Token>) -> Variable {
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
							(Operator::UnaryMinus, Variable::Number(n)) => stack.push(Variable::Number(-n)),
							(Operator::Not, Variable::Bool(b)) => stack.push(Variable::Bool(!b)),
							_ => panic!("Invalid unary operation {:?} {:?}", op, operand),
						}
					}
					ArityAndTypes::Binary(_, _) => {
						let operand2 = stack.pop().unwrap();
						let operand1 = stack.pop().unwrap();
						match (&op, &operand1, &operand2) {
							(Operator::Addition, Variable::Number(n1), Variable::Number(n2)) => stack.push(Variable::Number(n1 + n2)),
							(Operator::Substraction, Variable::Number(n1), Variable::Number(n2)) => stack.push(Variable::Number(n1 - n2)),
							(Operator::Multiplication, Variable::Number(n1), Variable::Number(n2)) => stack.push(Variable::Number(n1 * n2)),
							(Operator::Division, Variable::Number(n1), Variable::Number(n2)) => stack.push(Variable::Number(n1 / n2)),
							(Operator::Equals, Variable::Number(n1), Variable::Number(n2)) => stack.push(Variable::Bool(n1 == n2)),
							(Operator::And, Variable::Bool(b1), Variable::Bool(b2)) => stack.push(Variable::Bool(*b1 && *b2)),
							(Operator::Or, Variable::Bool(b1), Variable::Bool(b2)) => stack.push(Variable::Bool(*b1 || *b2)),
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
			Token::Operand(Variable::Bool(b)) => print!("{} ", b),
			Token::Operand(Variable::Number(n)) => print!("{} ", n),
			Token::Operand(Variable::String(s)) => print!("{} ", s),
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
				Token::Operand(Variable::String("a".to_string())), // TODO : count a as a variable, not a string
				Token::Operation(Operator::Addition),
				Token::Operand(Variable::Number(5)),
				Token::Parenthesis(')'),
				Token::Operation(Operator::Equals),
				Token::Operand(Variable::String("c".to_string())),
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
			vec![Token::Operation(Operator::UnaryMinus), Token::Operand(Variable::Number(5)),]
		);
	}

	#[test]
	fn test_unary_minus_in_expression() {
		let expression = "5*-5";
		let tokens = Token::from_string(expression);
		assert_eq!(
			tokens,
			vec![
				Token::Operand(Variable::Number(5)),
				Token::Operation(Operator::Multiplication),
				Token::Operation(Operator::UnaryMinus),
				Token::Operand(Variable::Number(5)),
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
				Token::Operand(Variable::String("a".to_string())),
				Token::Operation(Operator::Addition),
				Token::Operand(Variable::Number(5)),
				Token::Operation(Operator::Addition),
				Token::Operand(Variable::String("b".to_string())),
			]
		);
	}

	#[test]
	fn test_expr_tokens_to_rpn() {
		let tokens = vec![
			Token::Operand(Variable::Number(5)),
			Token::Operation(Operator::Addition),
			Token::Operand(Variable::Number(3)),
			Token::Operation(Operator::Multiplication),
			Token::Operand(Variable::Number(2)),
		];
		let rpn = expr_tokens_to_rpn(tokens);
		assert_eq!(
			rpn,
			vec![
				Token::Operand(Variable::Number(5)),
				Token::Operand(Variable::Number(3)),
				Token::Operand(Variable::Number(2)),
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
		assert_eq!(result, Variable::Number(11));
	}
}
