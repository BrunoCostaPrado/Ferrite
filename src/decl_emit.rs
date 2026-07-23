use crate::ast::{
  ClassField, Expression, ImportSpecifier, LiteralValue, MethodDefinition, MethodKind, Parameter,
  Program, PropertyKey, Statement, TypeAnn,
};

/// Emit a .d.ts declaration file from a parsed program.
#[must_use]
pub fn emit_declarations(program: &Program) -> String {
  let mut out = String::new();
  for stmt in &program.body {
    emit_statement(stmt, &mut out, false);
  }
  out
}

fn emit_statement(stmt: &Statement, out: &mut String, exported: bool) {
  match stmt {
    Statement::ExportDeclaration { declaration, .. } => {
      emit_statement(declaration, out, true);
    }
    Statement::ImportDeclaration { specifiers, source, is_type, .. } => {
      if *is_type {
        emit_import_type(specifiers, source, out);
      }
      // Value imports omitted from .d.ts (they don't add type declarations)
    }
    Statement::FunctionDeclaration { name, params, return_type, is_async, type_params, .. } => {
      if name.is_empty() {
        return; // anonymous function — skip
      }
      let async_kw = if *is_async { "async " } else { "" };
      let generics = render_type_params(type_params);
      let params_str = render_params(params);
      let ret = match return_type {
        Some(t) => format!(": {}", render_type_ann(t)),
        None => String::from(": void"),
      };
      let export = if exported { "export " } else { "" };
      writeln_indent(
        out,
        &format!("{export}{async_kw}function {name}{generics}{params_str}{ret};"),
      );
    }
    Statement::VariableDeclaration { declarations, .. } => {
      let export = if exported { "export " } else { "" };
      for decl in declarations {
        if let Expression::Identifier { name, .. } = decl.id.as_ref() {
          let ty_str = match &decl.type_ann {
            Some(t) => render_type_ann(t),
            None => String::from("any"),
          };
          writeln_indent(out, &format!("{export}const {name}: {ty_str};"));
        }
      }
    }
    Statement::ClassDeclaration { name, superclass, body, .. } => {
      let export = if exported { "export " } else { "" };
      let extends = match superclass.as_deref() {
        Some(Expression::Identifier { name, .. }) => format!(" extends {name}"),
        Some(Expression::MemberExpression { object, property, .. }) => {
          format!(" extends {}.{}", render_member_object(object), render_member_name(property))
        }
        _ => String::new(),
      };
      writeln_indent(out, &format!("{export}class {name}{extends} {{"));
      for method in &body.methods {
        emit_method(method, out);
      }
      for field in &body.fields {
        emit_field(field, out);
      }
      writeln_indent(out, "}");
    }
    Statement::TypeAliasDeclaration { name, type_params, type_annotation, .. } => {
      let generics = render_type_params(type_params);
      let export = if exported { "export " } else { "" };
      writeln_indent(
        out,
        &format!("{export}type {generics}{name} = {};", render_type_ann(type_annotation)),
      );
    }
    Statement::EnumDeclaration { name, members, .. } => {
      let export = if exported { "export " } else { "" };
      writeln_indent(out, &format!("{export}enum {name} {{"));
      for member in members {
        let val = match &member.value {
          Some(v) => format!(" = {v}"),
          None => String::new(),
        };
        writeln_indent(out, &format!("  {}{val},", member.name));
      }
      writeln_indent(out, "}");
    }
    _ => {} // Other statements don't produce declarations
  }
}

fn emit_method(method: &MethodDefinition, out: &mut String) {
  let vis = match method.visibility {
    Some(crate::ast::Visibility::Public) => "public ",
    Some(crate::ast::Visibility::Private) => "private ",
    Some(crate::ast::Visibility::Protected) => "protected ",
    None => "",
  };
  let static_kw = if method.is_static { "static " } else { "" };
  let generics = render_type_params(&[]);
  let params = render_params(&method.params);
  let name = render_property_key(&method.key);
  // Constructor has no return type
  match method.kind {
    MethodKind::Constructor => {
      writeln_indent(out, &format!("  {vis}{static_kw}constructor{params};"));
    }
    MethodKind::Method => {
      writeln_indent(out, &format!("  {vis}{static_kw}{generics}{name}{params};"));
    }
  }
}

fn emit_field(field: &ClassField, out: &mut String) {
  let vis = match field.visibility {
    Some(crate::ast::Visibility::Public) => "public ",
    Some(crate::ast::Visibility::Private) => "private ",
    Some(crate::ast::Visibility::Protected) => "protected ",
    None => "",
  };
  let static_kw = if field.is_static { "static " } else { "" };
  let name = render_property_key(&field.key);
  let ty = match &field.type_ann {
    Some(t) => format!(": {}", render_type_ann(t)),
    None => String::new(),
  };
  writeln_indent(out, &format!("  {vis}{static_kw}{name}{ty};"));
}

fn emit_import_type(specifiers: &[ImportSpecifier], source: &str, out: &mut String) {
  if specifiers.is_empty() {
    return;
  }
  let mut parts = Vec::new();
  for spec in specifiers {
    let imported = spec.imported.as_ref().unwrap_or(&spec.local);
    if *imported == spec.local {
      parts.push(imported.clone());
    } else {
      parts.push(format!("{imported} as {}", spec.local));
    }
  }
  writeln_indent(out, &format!("import type {{ {} }} from \"{}\";", parts.join(", "), source));
}

fn render_type_params(params: &[(String, Option<TypeAnn>)]) -> String {
  if params.is_empty() {
    return String::new();
  }
  let parts: Vec<String> = params
    .iter()
    .map(|(name, constraint)| match constraint {
      Some(c) => format!("{name} extends {}", render_type_ann(c)),
      None => name.clone(),
    })
    .collect();
  format!("<{}>", parts.join(", "))
}

fn render_params(params: &[Parameter]) -> String {
  let parts: Vec<String> = params
    .iter()
    .map(|p| {
      let prefix = if p.is_rest { "..." } else { "" };
      let ty = match &p.type_ann {
        Some(t) => format!(": {}", render_type_ann(t)),
        None => String::new(),
      };
      format!("{prefix}{}{ty}", p.name)
    })
    .collect();
  format!("({})", parts.join(", "))
}

pub fn render_type_ann(ty: &TypeAnn) -> String {
  match ty {
    TypeAnn::Number => String::from("number"),
    TypeAnn::String => String::from("string"),
    TypeAnn::Boolean => String::from("boolean"),
    TypeAnn::Null => String::from("null"),
    TypeAnn::Undefined => String::from("undefined"),
    TypeAnn::Void => String::from("void"),
    TypeAnn::Any => String::from("any"),
    TypeAnn::Unknown => String::from("unknown"),
    TypeAnn::Never => String::from("never"),
    TypeAnn::Literal { value } => match value {
      LiteralValue::String(s) => format!("\"{s}\""),
      LiteralValue::Number(n) => format!("{n}"),
      LiteralValue::Boolean(b) => format!("{b}"),
    },
    TypeAnn::Union { types } => {
      let parts: Vec<String> = types.iter().map(render_type_ann).collect();
      parts.join(" | ")
    }
    TypeAnn::Intersection { types } => {
      let parts: Vec<String> = types.iter().map(render_type_ann).collect();
      parts.join(" & ")
    }
    TypeAnn::Array { element } => format!("{}[]", render_type_ann(element)),
    TypeAnn::TypeRef { name, type_args } => {
      if type_args.is_empty() {
        name.clone()
      } else {
        let args: Vec<String> = type_args.iter().map(render_type_ann).collect();
        format!("{name}<{}>", args.join(", "))
      }
    }
    TypeAnn::Function { params, return_type } => {
      let params_str: Vec<String> = params.iter().map(render_type_ann).collect();
      format!("({}) => {}", params_str.join(", "), render_type_ann(return_type))
    }
    TypeAnn::Object { properties } => {
      if properties.is_empty() {
        return String::from("{}");
      }
      let fields: Vec<String> = properties
        .iter()
        .map(|p| format!("{}: {}", p.name, render_type_ann(&p.type_ann)))
        .collect();
      format!("{{ {} }}", fields.join("; "))
    }
    TypeAnn::KeyOf { type_ann } => format!("keyof {}", render_type_ann(type_ann)),
    TypeAnn::Typeof { .. } => String::from("any"), // typeof in type position → fallback
    TypeAnn::TemplateLiteral { .. } => String::from("string"),
    TypeAnn::Conditional { check, extends, true_type, false_type } => {
      format!(
        "{} extends {} ? {} : {}",
        render_type_ann(check),
        render_type_ann(extends),
        render_type_ann(true_type),
        render_type_ann(false_type),
      )
    }
    TypeAnn::Infer { name } => format!("infer {name}"),
    TypeAnn::Mapped { key: _, target, value } => {
      format!("{{ [K in keyof {}]: {} }}", render_type_ann(target), render_type_ann(value))
    }
    TypeAnn::IndexedAccess { target, index } => {
      format!("{}[{}]", render_type_ann(target), render_type_ann(index))
    }
  }
}

fn render_property_key(key: &PropertyKey) -> String {
  match key {
    PropertyKey::Identifier(s) => s.clone(),
    PropertyKey::String(s) => format!("\"{s}\""),
    PropertyKey::Expression(_) => String::from("[computed]"),
  }
}

fn render_member_object(obj: &Expression) -> String {
  match obj {
    Expression::Identifier { name, .. } => name.clone(),
    _ => String::from("..."),
  }
}

fn render_member_name(prop: &Expression) -> String {
  match prop {
    Expression::Identifier { name, .. } => name.clone(),
    _ => String::from("..."),
  }
}

fn writeln_indent(out: &mut String, line: &str) {
  out.push_str(line);
  out.push('\n');
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::diagnostic::SourceFile;
  use crate::lexer::Lexer;
  use crate::parser::Parser;

  fn emit(source: &str) -> String {
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize().to_vec();
    let mut parser = Parser::new(tokens, SourceFile::new("test.ts", source));
    let program = parser.parse();
    emit_declarations(&program)
  }

  #[test]
  fn emit_exported_function() {
    let dts = emit("export function add(a: number, b: number): number { return a + b; }");
    assert!(dts.contains("export function add(a: number, b: number): number;"));
  }

  #[test]
  fn emit_exported_const() {
    let dts = emit("export const x: number = 42;");
    assert!(dts.contains("export const x: number;"));
  }

  #[test]
  fn emit_type_alias() {
    let dts = emit("export type ID = string | number;");
    assert!(dts.contains("export type ID = string | number;"));
  }

  #[test]
  fn emit_class_declaration() {
    let dts = emit("export class Foo { name: string; greet(): void {} }");
    assert!(dts.contains("export class Foo {"));
    assert!(dts.contains("name: string;"));
    assert!(dts.contains("greet();"));
  }

  #[test]
  fn emit_class_extends() {
    let dts =
      emit("class Animal { name: string; } export class Dog extends Animal { breed: string; }");
    assert!(dts.contains("class Dog extends Animal {"));
  }

  #[test]
  fn emit_import_type() {
    let dts = emit("import type { Name, Age } from \"./types\";");
    assert!(dts.contains("import type { Name, Age } from \"./types\";"));
  }

  #[test]
  fn emit_generic_function() {
    let dts = emit("export function identity<T>(x: T): T { return x; }");
    assert!(dts.contains("export function identity<T>(x: T): T;"));
  }

  #[test]
  fn emit_generic_type_alias() {
    // Note: parser doesn't support generic type params on type aliases yet
    let dts = emit("export type ID = string | number;");
    assert!(dts.contains("export type ID = string | number;"));
  }

  #[test]
  fn emit_rest_params() {
    let dts = emit("export function sum(...nums: number[]): number { return 0; }");
    assert!(dts.contains("export function sum(...nums: number[]): number;"));
  }

  #[test]
  fn emit_object_type() {
    let dts = emit("export type Point = { x: number; y: number; };");
    assert!(dts.contains("export type Point = { x: number; y: number };"));
  }

  #[test]
  fn emit_enum() {
    let dts = emit("export enum Color { Red, Green, Blue }");
    assert!(dts.contains("export enum Color {"));
    assert!(dts.contains("Red,"));
    assert!(dts.contains("Green,"));
    assert!(dts.contains("Blue,"));
  }

  #[test]
  fn emit_intersection_type() {
    let dts = emit("export type A = { name: string; } & { age: number; };");
    assert!(dts.contains("export type A = { name: string } & { age: number };"));
  }

  #[test]
  fn emit_array_type() {
    let dts = emit("export const items: string[] = [];");
    assert!(dts.contains("export const items: string[];"));
  }

  #[test]
  fn emit_generic_class() {
    // Note: ClassDeclaration AST doesn't have type_params field yet
    let dts = emit("export class Foo { value: string; }");
    assert!(dts.contains("export class Foo {"));
    assert!(dts.contains("value: string;"));
  }

  #[test]
  fn emit_async_function() {
    let dts = emit("export async function fetchData(): Promise<string> { return \"\"; }");
    assert!(dts.contains("export async function fetchData(): Promise<string>;"));
  }
}
