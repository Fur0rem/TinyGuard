use either::{self, Either};

#[derive(Debug, Clone)]
pub struct FileLine {
	line_number: usize,
	line: String,
}

impl FileLine {
	pub fn new(line_number: usize, line: String) -> FileLine {
		FileLine { line_number, line }
	}

	pub fn trimmed(&self) -> FileLine {
		FileLine {
			line_number: self.line_number,
			line: self.line.trim().to_string(),
		}
	}
}

#[derive(Debug)]
pub struct Bracketed {
	beginning_line: FileLine,
	ending_line: FileLine,
	pub content: Either<String, Vec<Bracketed>>,
}

impl Bracketed {
	fn new(beginning_line: FileLine, ending_line: FileLine, content: Either<String, Vec<Bracketed>>) -> Bracketed {
		Bracketed {
			beginning_line,
			ending_line,
			content,
		}
	}
}

pub fn parse_bracketed(file: &str) -> Vec<Bracketed> {
	let mut stack = Vec::new();
	let mut bracketed_vec = Vec::new();
	for (line_number, line) in file.lines().enumerate() {
		if line.contains("{") {
			stack.push(FileLine::new(line_number, line.to_string()));
			bracketed_vec.push(Bracketed::new(
				FileLine::new(line_number, line.to_string()),
				FileLine::new(line_number, line.to_string()),
				Either::Left("".to_string()),
			));
		}
		else if line.contains("}") {
			let beginning_line = stack.pop().unwrap();
			let ending_line = FileLine::new(line_number, line.to_string());
			let mut content = Vec::new();
			while let Some(bracketed) = bracketed_vec.pop() {
				if bracketed.beginning_line.line_number == beginning_line.line_number {
					bracketed_vec.push(Bracketed::new(beginning_line, ending_line, Either::Right(content)));
					break;
				}
				else {
					content.push(bracketed);
				}
			}
		}
	}
	return bracketed_vec;
}

pub fn fill_blanks(bracketed: &Bracketed, text: &str) -> Bracketed {
	match bracketed.content {
		Either::Left(_) => {
			let lines = text.lines().map(|line| line.to_string()).collect::<Vec<_>>();
			let mut new_text = String::new();
			for line in lines
				.iter()
				.skip(bracketed.beginning_line.line_number + 1)
				.take(bracketed.ending_line.line_number - bracketed.beginning_line.line_number - 1)
				.map(|line| line.trim())
			{
				new_text.push_str(line);
				new_text.push_str("\n");
			}
			Bracketed::new(
				bracketed.beginning_line.clone().trimmed(),
				bracketed.ending_line.clone().trimmed(),
				Either::Left(new_text),
			)
		}
		Either::Right(ref vec) => {
			let mut new_vec = Vec::new();
			if vec.is_empty() {
				let lines = text.lines().map(|line| line.to_string()).collect::<Vec<_>>();
				let mut new_text = String::new();
				for line in lines
					.iter()
					.skip(bracketed.beginning_line.line_number + 1)
					.take(bracketed.ending_line.line_number - bracketed.beginning_line.line_number - 1)
					.map(|line| line.trim())
				{
					new_text.push_str(line);
					new_text.push_str("\n");
				}
				return Bracketed::new(
					bracketed.beginning_line.clone().trimmed(),
					bracketed.ending_line.clone().trimmed(),
					Either::Left(new_text),
				);
			}
			for b in vec {
				new_vec.push(fill_blanks(b, text));
			}
			Bracketed::new(
				bracketed.beginning_line.clone().trimmed(),
				bracketed.ending_line.clone().trimmed(),
				Either::Right(new_vec),
			)
		}
	}
}
