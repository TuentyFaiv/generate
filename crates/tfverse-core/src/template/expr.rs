//! Condition expressions: `name`, `!name`, `a == "x"`, `a != "x"`, `&&`, `||` and parentheses.

use crate::model::{Value, Values};

const MAX_LEN: usize = 256;
const MAX_DEPTH: usize = 32;

/// A parsed condition.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Expr {
	/// An input or fact, true when truthy.
	Var(String),
	/// `!expr`
	Not(Box<Self>),
	/// `left && right`
	And(Box<Self>, Box<Self>),
	/// `left || right`
	Or(Box<Self>, Box<Self>),
	/// `name == literal`, or `!=` when `equal` is false.
	Compare {
		/// Input or fact name.
		name: String,
		/// Value compared against.
		literal: Value,
		/// `==` when true, `!=` when false.
		equal: bool,
	},
}

/// Why a condition could not be parsed.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum ExprError {
	/// Longer than the supported limit.
	#[error("condition is longer than {MAX_LEN} characters")]
	TooLong,
	/// Nested too deeply.
	#[error("condition is nested more than {MAX_DEPTH} levels deep")]
	TooDeep,
	/// Unexpected character or token.
	#[error("invalid condition `{text}`: {reason}")]
	Syntax {
		/// The full condition.
		text: String,
		/// What went wrong.
		reason: String,
	},
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Token {
	Ident(String),
	Literal(Value),
	Not,
	And,
	Or,
	Equal,
	NotEqual,
	Open,
	Close,
}

impl Expr {
	/// Parses `text`.
	pub fn parse(text: &str) -> Result<Self, ExprError> {
		if text.len() > MAX_LEN {
			return Err(ExprError::TooLong);
		}
		let tokens = tokenize(text)?;
		let mut parser = Parser {
			text,
			tokens: &tokens,
			position: 0,
			depth: 0,
		};
		let expr = parser.or()?;
		if parser.position != tokens.len() {
			return Err(parser.error("unexpected trailing tokens"));
		}
		Ok(expr)
	}

	/// Evaluates against `values`; unknown names are false.
	#[must_use]
	pub fn eval(&self, values: &Values) -> bool {
		match self {
			Self::Var(name) => values.get(name).is_some_and(Value::is_truthy),
			Self::Not(inner) => !inner.eval(values),
			Self::And(left, right) => left.eval(values) && right.eval(values),
			Self::Or(left, right) => left.eval(values) || right.eval(values),
			Self::Compare { name, literal, equal } => (values.get(name) == Some(literal)) == *equal,
		}
	}

	/// Every name the condition reads.
	#[must_use]
	pub fn names(&self) -> Vec<&str> {
		let mut names = Vec::new();
		self.collect_names(&mut names);
		names
	}

	fn collect_names<'a>(&'a self, names: &mut Vec<&'a str>) {
		match self {
			Self::Var(name) | Self::Compare { name, .. } => names.push(name),
			Self::Not(inner) => inner.collect_names(names),
			Self::And(left, right) | Self::Or(left, right) => {
				left.collect_names(names);
				right.collect_names(names);
			}
		}
	}
}

fn tokenize(text: &str) -> Result<Vec<Token>, ExprError> {
	let syntax = |reason: &str| ExprError::Syntax {
		text: text.to_owned(),
		reason: reason.to_owned(),
	};
	let mut tokens = Vec::new();
	let mut chars = text.char_indices().peekable();

	while let Some((start, c)) = chars.next() {
		match c {
			c if c.is_whitespace() => {}
			'(' => tokens.push(Token::Open),
			')' => tokens.push(Token::Close),
			'!' if chars.next_if(|&(_, next)| next == '=').is_some() => tokens.push(Token::NotEqual),
			'!' => tokens.push(Token::Not),
			'=' if chars.next_if(|&(_, next)| next == '=').is_some() => tokens.push(Token::Equal),
			'&' if chars.next_if(|&(_, next)| next == '&').is_some() => tokens.push(Token::And),
			'|' if chars.next_if(|&(_, next)| next == '|').is_some() => tokens.push(Token::Or),
			'"' | '\'' => {
				let mut literal = String::new();
				loop {
					match chars.next() {
						Some((_, close)) if close == c => break,
						Some((_, inner)) => literal.push(inner),
						None => return Err(syntax("unterminated string")),
					}
				}
				tokens.push(Token::Literal(Value::String(literal)));
			}
			c if c.is_ascii_digit() || c == '-' => {
				let mut end = start + c.len_utf8();
				while let Some((index, digit)) = chars.next_if(|&(_, next)| next.is_ascii_digit()) {
					end = index + digit.len_utf8();
				}
				let number = text[start..end].parse().map_err(|_| syntax("invalid number"))?;
				tokens.push(Token::Literal(Value::Number(number)));
			}
			c if c.is_alphabetic() || c == '_' => {
				let mut end = start + c.len_utf8();
				while let Some((index, part)) =
					chars.next_if(|&(_, next)| next.is_alphanumeric() || matches!(next, '_' | '.' | '-'))
				{
					end = index + part.len_utf8();
				}
				tokens.push(match &text[start..end] {
					"true" => Token::Literal(Value::Bool(true)),
					"false" => Token::Literal(Value::Bool(false)),
					name => Token::Ident(name.to_owned()),
				});
			}
			_ => return Err(syntax(&format!("unexpected `{c}`"))),
		}
	}
	Ok(tokens)
}

struct Parser<'a> {
	text: &'a str,
	tokens: &'a [Token],
	position: usize,
	depth: usize,
}

impl Parser<'_> {
	fn error(&self, reason: &str) -> ExprError {
		ExprError::Syntax {
			text: self.text.to_owned(),
			reason: reason.to_owned(),
		}
	}

	fn peek(&self) -> Option<&Token> {
		self.tokens.get(self.position)
	}

	fn advance(&mut self) -> Option<&Token> {
		let token = self.tokens.get(self.position);
		self.position += 1;
		token
	}

	fn or(&mut self) -> Result<Expr, ExprError> {
		let mut left = self.and()?;
		while self.peek() == Some(&Token::Or) {
			self.position += 1;
			left = Expr::Or(Box::new(left), Box::new(self.and()?));
		}
		Ok(left)
	}

	fn and(&mut self) -> Result<Expr, ExprError> {
		let mut left = self.unary()?;
		while self.peek() == Some(&Token::And) {
			self.position += 1;
			left = Expr::And(Box::new(left), Box::new(self.unary()?));
		}
		Ok(left)
	}

	fn unary(&mut self) -> Result<Expr, ExprError> {
		self.depth += 1;
		if self.depth > MAX_DEPTH {
			return Err(ExprError::TooDeep);
		}
		let expr = if self.peek() == Some(&Token::Not) {
			self.position += 1;
			Expr::Not(Box::new(self.unary()?))
		} else {
			self.atom()?
		};
		self.depth -= 1;
		Ok(expr)
	}

	fn atom(&mut self) -> Result<Expr, ExprError> {
		match self.advance().cloned() {
			Some(Token::Open) => {
				let inner = self.or()?;
				match self.advance() {
					Some(Token::Close) => Ok(inner),
					_ => Err(self.error("missing `)`")),
				}
			}
			Some(Token::Ident(name)) => match self.peek() {
				Some(Token::Equal | Token::NotEqual) => {
					let equal = self.advance() == Some(&Token::Equal);
					match self.advance().cloned() {
						Some(Token::Literal(literal)) => Ok(Expr::Compare { name, literal, equal }),
						_ => Err(self.error("expected a value after the comparison")),
					}
				}
				_ => Ok(Expr::Var(name)),
			},
			_ => Err(self.error("expected a name or `(`")),
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	fn values() -> Values {
		Values::from([
			("styled".to_owned(), Value::Bool(true)),
			("tests".to_owned(), Value::Bool(false)),
			("style".to_owned(), Value::String("scss".to_owned())),
		])
	}

	fn eval(text: &str) -> bool {
		Expr::parse(text).unwrap().eval(&values())
	}

	#[test]
	fn var_should_read_bool_input() {
		assert!(eval("styled"));
	}

	#[test]
	fn not_should_negate() {
		assert!(eval("!tests"));
	}

	#[test]
	fn compare_should_match_string_literal() {
		assert!(eval("style == 'scss'"));
	}

	#[test]
	fn not_equal_should_reject_matching_literal() {
		assert!(!eval("style != \"scss\""));
	}

	#[test]
	fn and_should_bind_tighter_than_or() {
		assert!(eval("tests && styled || styled"));
	}

	#[test]
	fn parentheses_should_group() {
		assert!(!eval("tests && (styled || styled)"));
	}

	#[test]
	fn unknown_name_should_be_false() {
		assert!(!eval("missing"));
	}

	#[test]
	fn parse_should_reject_dangling_operator() {
		assert!(Expr::parse("styled &&").is_err());
	}

	#[test]
	fn parse_should_reject_unterminated_string() {
		assert!(Expr::parse("style == 'scss").is_err());
	}

	#[test]
	fn parse_should_reject_deep_nesting() {
		assert_eq!(Expr::parse(&format!("{}x", "!".repeat(40))), Err(ExprError::TooDeep));
	}

	#[test]
	fn names_should_list_every_read() {
		assert_eq!(Expr::parse("a && (b == 'x' || !c)").unwrap().names(), ["a", "b", "c"]);
	}
}
