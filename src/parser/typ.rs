use crate::ast::{Expression, LiteralValue, ObjectTypeProperty, TypeAnn};
use crate::token::TokenKind;

use super::Parser;

impl Parser {
  pub(super) fn parse_type_annotation(&mut self) -> Option<TypeAnn> {
    self.expect(TokenKind::Colon);
    self.parse_type()
  }

  pub(super) fn parse_type(&mut self) -> Option<TypeAnn> {
    let mut type_ann = self.parse_atom_type()?;
    // Handle dotted qualified names (e.g., z.infer)
    while self.peek().kind == TokenKind::Dot {
      if let TypeAnn::TypeRef { name, .. } = &mut type_ann {
        self.advance();
        if let TokenKind::Identifier(part) = &self.peek().kind {
          name.push('.');
          name.push_str(part);
          self.advance();
        }
      } else {
        break;
      }
    }
    // Handle generic type arguments (e.g., <string, number>)
    if self.peek().kind == TokenKind::Lt && self.peek_ahead(1).kind != TokenKind::Eq {
      self.advance(); // consume <
      let mut type_args = Vec::new();
      while self.peek().kind != TokenKind::Gt && !self.is_at_end() {
        if let Some(t) = self.parse_type() {
          type_args.push(t);
        } else {
          // Can't parse this token as a type — advance to avoid infinite loop
          self.advance();
        }
        if self.peek().kind == TokenKind::Comma {
          self.advance();
        }
      }
      self.advance(); // consume >
      if let TypeAnn::TypeRef { type_args: ref mut args, .. } = type_ann {
        *args = type_args;
      }
    }
    // Handle postfix brackets: T[K] (indexed access) or T[] (array)
    loop {
      if self.peek().kind == TokenKind::OpenBracket {
        if self.peek_ahead(1).kind == TokenKind::CloseBracket {
          // T[]
          self.advance();
          self.advance();
          type_ann = TypeAnn::Array { element: Box::new(type_ann) };
        } else if !self.is_at_end() {
          // T[K]
          self.advance(); // consume [
          if let Some(index) = self.parse_type()
            && self.peek().kind == TokenKind::CloseBracket
          {
            self.advance(); // consume ]
            type_ann =
              TypeAnn::IndexedAccess { target: Box::new(type_ann), index: Box::new(index) };
          }
        } else {
          break;
        }
      } else {
        break;
      }
    }
    // Handle union types
    if self.peek().kind == TokenKind::Pipe {
      let mut types = vec![type_ann];
      while self.peek().kind == TokenKind::Pipe {
        self.advance();
        if let Some(t) = self.parse_type() {
          types.push(t);
        }
      }
      Some(TypeAnn::Union { types })
    } else if self.peek().kind == TokenKind::Amp {
      let mut types = vec![type_ann];
      while self.peek().kind == TokenKind::Amp {
        self.advance();
        if let Some(t) = self.parse_atom_type() {
          types.push(t);
        }
      }
      Some(TypeAnn::Intersection { types })
    } else if self.peek().kind == TokenKind::Extends {
      // Conditional type: C extends T ? X : Y
      self.advance(); // consume 'extends'
      let extends_type = self.parse_type().unwrap_or(TypeAnn::Any);
      if self.peek().kind == TokenKind::Question {
        self.advance(); // consume '?'
        let true_type = self.parse_type().unwrap_or(TypeAnn::Any);
        if self.peek().kind == TokenKind::Colon {
          self.advance(); // consume ':'
          let false_type = self.parse_type().unwrap_or(TypeAnn::Any);
          Some(TypeAnn::Conditional {
            check: Box::new(type_ann),
            extends: Box::new(extends_type),
            true_type: Box::new(true_type),
            false_type: Box::new(false_type),
          })
        } else {
          Some(type_ann)
        }
      } else {
        Some(type_ann)
      }
    } else {
      Some(type_ann)
    }
  }

  pub(super) fn parse_atom_type(&mut self) -> Option<TypeAnn> {
    let kind = self.peek().kind.clone();
    match &kind {
      TokenKind::Identifier(name) => {
        self.advance();
        match name.as_str() {
          "number" => Some(TypeAnn::Number),
          "string" => Some(TypeAnn::String),
          "boolean" => Some(TypeAnn::Boolean),
          "any" => Some(TypeAnn::Any),
          "unknown" => Some(TypeAnn::Unknown),
          "never" => Some(TypeAnn::Never),
          "infer" => Some(TypeAnn::Infer {
            name: {
              // infer T — next identifier is the type parameter name
              if self.peek().kind == TokenKind::Lt {
                // Could be infer<T> but simpler: just next ident
                // Consume nothing extra for now — bare infer T
              }
              // peek next ident
              if let TokenKind::Identifier(n) = &self.peek().kind {
                let n = n.clone();
                self.advance();
                n
              } else {
                "_".to_string()
              }
            },
          }),
          _ => Some(TypeAnn::TypeRef { name: name.clone(), type_args: Vec::new() }),
        }
      }
      TokenKind::Null => {
        self.advance();
        Some(TypeAnn::Null)
      }
      TokenKind::Undefined => {
        self.advance();
        Some(TypeAnn::Undefined)
      }
      TokenKind::Void => {
        self.advance();
        Some(TypeAnn::Void)
      }
      TokenKind::Number(n) => {
        self.advance();
        Some(TypeAnn::Literal { value: LiteralValue::Number(*n) })
      }
      TokenKind::String(s) => {
        self.advance();
        Some(TypeAnn::Literal { value: LiteralValue::String(s.clone()) })
      }
      TokenKind::True => {
        self.advance();
        Some(TypeAnn::Literal { value: LiteralValue::Boolean(true) })
      }
      TokenKind::False => {
        self.advance();
        Some(TypeAnn::Literal { value: LiteralValue::Boolean(false) })
      }
      TokenKind::TypeOf => {
        self.advance();
        // Parse entity name path only (not full expression — avoids consuming generic '>')
        let mut name = String::new();
        if let TokenKind::Identifier(n) = &self.peek().kind {
          name.clone_from(n);
          self.advance();
        }
        while self.peek().kind == TokenKind::Dot
          && matches!(&self.peek_ahead(1).kind, TokenKind::Identifier(_))
        {
          self.advance();
          if let TokenKind::Identifier(part) = &self.peek().kind {
            name.push('.');
            name.push_str(part);
            self.advance();
          }
        }
        let span = self.tokens[self.cursor.max(1) - 1].span;
        Some(TypeAnn::Typeof { argument: Box::new(Expression::Identifier { name, span }) })
      }
      TokenKind::KeyOf => {
        self.advance();
        let inner = self.parse_atom_type()?;
        Some(TypeAnn::KeyOf { type_ann: Box::new(inner) })
      }
      TokenKind::NoSubstitutionTemplate(s) => {
        let s = s.clone();
        self.advance();
        Some(TypeAnn::TemplateLiteral { quasis: vec![s], types: Vec::new() })
      }
      TokenKind::TemplateHead(s) => {
        let s = s.clone();
        self.advance();
        let mut quasis = vec![s];
        let mut types = Vec::new();
        loop {
          if let Some(t) = self.parse_type() {
            types.push(t);
          }
          match &self.peek().kind {
            TokenKind::TemplateTail(s) => {
              quasis.push(s.clone());
              self.advance();
              break;
            }
            TokenKind::TemplateMiddle(s) => {
              quasis.push(s.clone());
              self.advance();
            }
            _ => break,
          }
        }
        Some(TypeAnn::TemplateLiteral { quasis, types })
      }
      TokenKind::OpenParen => {
        // Function type: (Type, Type) => ReturnType
        // Need to look ahead to distinguish from invalid type syntax
        // Try parsing comma-separated types until CloseParen, then check for =>
        let saved = self.cursor;
        self.advance(); // consume (
        let mut params = Vec::new();
        let mut valid = true;
        while self.peek().kind != TokenKind::CloseParen && !self.is_at_end() {
          if let Some(t) = self.parse_type() {
            params.push(t);
          } else {
            valid = false;
            break;
          }
          if self.peek().kind == TokenKind::Comma {
            self.advance();
          }
        }
        if valid && self.peek().kind == TokenKind::CloseParen {
          self.advance(); // consume )
          if self.peek().kind == TokenKind::Arrow {
            self.advance(); // consume =>
            let return_type = self.parse_type().unwrap_or(TypeAnn::Void);
            return Some(TypeAnn::Function { params, return_type: Box::new(return_type) });
          }
        }
        // Not a function type — backtrack
        self.cursor = saved;
        None
      }
      TokenKind::Function => {
        self.advance();
        self.expect(TokenKind::OpenParen);
        let mut params = Vec::new();
        while self.peek().kind != TokenKind::CloseParen && !self.is_at_end() {
          if let Some(t) = self.parse_type() {
            params.push(t);
          }
          if self.peek().kind == TokenKind::Comma {
            self.advance();
          }
        }
        self.expect(TokenKind::CloseParen);
        self.expect(TokenKind::Arrow);
        let return_type = self.parse_type().unwrap_or(TypeAnn::Void);
        Some(TypeAnn::Function { params, return_type: Box::new(return_type) })
      }
      TokenKind::OpenBrace => {
        self.advance(); // consume '{'
        // Check for mapped type: { [K in keyof T]: V }
        if self.peek().kind == TokenKind::OpenBracket
          && let Some(mapped) = self.parse_mapped_type_rest()
        {
          return Some(mapped);
        }
        let mut properties = Vec::new();
        while self.peek().kind != TokenKind::CloseBrace && !self.is_at_end() {
          let name = if let TokenKind::Identifier(n) = &self.peek().kind {
            n.clone()
          } else {
            break;
          };
          self.advance();
          self.expect(TokenKind::Colon);
          let type_ann = self.parse_type().unwrap_or(TypeAnn::Any);
          let span = self.peek().span;
          properties.push(ObjectTypeProperty { name, type_ann, span });
          if self.peek().kind == TokenKind::Comma || self.peek().kind == TokenKind::Semicolon {
            self.advance();
          }
        }
        self.expect(TokenKind::CloseBrace);
        Some(TypeAnn::Object { properties })
      }
      _ => None,
    }
  }

  /// Parse mapped type body: already consumed `{`, now parse `[K in keyof T]: V }`
  pub(super) fn parse_mapped_type_rest(&mut self) -> Option<TypeAnn> {
    // already consumed '{'
    // [
    if self.peek().kind != TokenKind::OpenBracket {
      return None;
    }
    self.advance();
    // key identifier
    let key = match &self.peek().kind {
      TokenKind::Identifier(n) => n.clone(),
      _ => return None,
    };
    self.advance();
    // in
    if self.peek().kind != TokenKind::In {
      return None;
    }
    self.advance();
    // keyof Target
    if self.peek().kind == TokenKind::KeyOf {
      self.advance();
    }
    let target_name = match &self.peek().kind {
      TokenKind::Identifier(n) => n.clone(),
      _ => return None,
    };
    self.advance();
    // ]
    if self.peek().kind != TokenKind::CloseBracket {
      return None;
    }
    self.advance();
    // :
    if self.peek().kind != TokenKind::Colon {
      return None;
    }
    self.advance();
    // value type
    let value = self.parse_type()?;
    // }
    if self.peek().kind == TokenKind::CloseBrace {
      self.advance();
    }
    let target = TypeAnn::TypeRef { name: target_name, type_args: vec![] };
    Some(TypeAnn::Mapped { key, target: Box::new(target), value: Box::new(value) })
  }
}
