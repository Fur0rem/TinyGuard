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
				Token::Operand(Variable::String("a".to_string())),
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
}
