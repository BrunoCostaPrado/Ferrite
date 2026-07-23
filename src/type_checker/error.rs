use crate::token::Span;

#[derive(Debug, Clone)]
pub enum TypeError {
  CannotFindName(String, Span),
  NotAssignable { target: String, value: String, span: Span },
  DuplicateIdentifier(String, Span),
  ImplicitAny { name: String, span: Span },
  ArgumentCountMismatch { expected: usize, found: usize, span: Span },
  NotAFunction { name: String, span: Span },
  IndexError { target: String, span: Span },
}

impl std::fmt::Display for TypeError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      TypeError::CannotFindName(name, _) => write!(f, "Cannot find name '{name}'"),
      TypeError::NotAssignable { target, value, .. } => {
        write!(f, "Type '{value}' is not assignable to type '{target}'")
      }
      TypeError::DuplicateIdentifier(name, _) => write!(f, "Duplicate identifier '{name}'"),
      TypeError::ImplicitAny { name, .. } => {
        write!(f, "Parameter '{name}' implicitly has an 'any' type")
      }
      TypeError::ArgumentCountMismatch { expected, found, .. } => {
        write!(f, "Expected {expected} arguments, but got {found}")
      }
      TypeError::NotAFunction { name, .. } => {
        write!(f, "'{name}' is not callable")
      }
      TypeError::IndexError { target, .. } => {
        write!(f, "Cannot index into type '{target}'")
      }
    }
  }
}

impl TypeError {
  #[must_use]
  pub fn span(&self) -> Span {
    match self {
      TypeError::CannotFindName(_, span)
      | TypeError::NotAssignable { span, .. }
      | TypeError::DuplicateIdentifier(_, span)
      | TypeError::ImplicitAny { span, .. }
      | TypeError::ArgumentCountMismatch { span, .. }
      | TypeError::NotAFunction { span, .. }
      | TypeError::IndexError { span, .. } => *span,
    }
  }
}
