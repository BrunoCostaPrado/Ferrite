mod check;
mod infer_expr;
mod types;

#[cfg(test)]
mod tests {
  use crate::ast::LiteralValue;
  use crate::diagnostic::SourceFile;
  use crate::lexer::Lexer;
  use crate::parser::Parser;
  use crate::type_checker::TypeChecker;
  use crate::type_checker::env::TypeEnv;
  use crate::type_checker::error::TypeError;
  use crate::type_checker::ty::Type;

  fn check_program(source: &str) -> (TypeChecker, TypeEnv) {
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize().to_vec();
    let mut parser = Parser::new(tokens, SourceFile::new("test.ts", source));
    let program = parser.parse();
    let mut checker = TypeChecker::new();
    let mut env = TypeEnv::new();
    checker.check(&program, &mut env);
    (checker, env)
  }

  fn check_program_strict(source: &str) -> (TypeChecker, TypeEnv) {
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize().to_vec();
    let mut parser = Parser::new(tokens, SourceFile::new("test.ts", source));
    let program = parser.parse();
    let mut checker = TypeChecker::new();
    checker.options.strict = Some(true);
    let mut env = TypeEnv::new();
    checker.check(&program, &mut env);
    (checker, env)
  }

  #[test]
  fn infer_number_literal() {
    let (checker, _) = check_program("42;");
    assert!(checker.errors.is_empty());
  }

  #[test]
  fn infer_string_literal() {
    let (checker, _) = check_program("\"hello\";");
    assert!(checker.errors.is_empty());
  }

  #[test]
  fn typed_variable_ok() {
    let (checker, _) = check_program("let x: number = 42;");
    assert!(checker.errors.is_empty());
  }

  #[test]
  fn typed_variable_mismatch() {
    let (checker, _) = check_program("let x: number = \"hello\";");
    assert!(!checker.errors.is_empty());
  }

  #[test]
  fn typed_variable_union_ok() {
    let (checker, _) = check_program("let x: string | number = 42;");
    assert!(checker.errors.is_empty());
  }

  #[test]
  fn inferred_variable_type() {
    let (checker, env) = check_program("let x = 42;");
    assert!(checker.errors.is_empty());
    assert_eq!(env.lookup("x"), Some(Type::Literal(LiteralValue::Number(42.0))));
  }

  #[test]
  fn undeclared_identifier() {
    let (checker, _) = check_program("foo;");
    assert!(!checker.errors.is_empty());
  }

  #[test]
  fn variable_used_after_declaration() {
    let (checker, _) = check_program("let x: number = 42; x;");
    assert!(checker.errors.is_empty());
  }

  #[test]
  fn typed_variable_boolean_ok() {
    let (checker, _) = check_program("let x: boolean = true;");
    assert!(checker.errors.is_empty());
  }

  #[test]
  fn typed_variable_string_ok() {
    let (checker, _) = check_program("let x: string = \"hello\";");
    assert!(checker.errors.is_empty());
  }

  #[test]
  fn typed_variable_array_mismatch() {
    let (checker, _) = check_program("let x: number[] = 42;");
    assert!(!checker.errors.is_empty());
  }

  #[test]
  fn typed_variable_array_no_init() {
    let (checker, _) = check_program("let x: number[];");
    assert!(checker.errors.is_empty());
  }

  #[test]
  fn typed_variable_any_accepts_all() {
    let (checker, _) = check_program("let x: any = 42; let y: any = \"hello\";");
    assert!(checker.errors.is_empty());
  }

  #[test]
  fn binary_infers_number() {
    let (checker, _) = check_program("1 + 2;");
    assert!(checker.errors.is_empty());
  }

  #[test]
  fn comparison_infers_boolean() {
    let (checker, _) = check_program("1 === 2;");
    assert!(checker.errors.is_empty());
  }

  #[test]
  fn conditional_expression() {
    let (checker, _) = check_program("true ? 1 : 2;");
    assert!(checker.errors.is_empty());
  }

  #[test]
  fn undeclared_variable_used() {
    let (checker, _) = check_program("let x: number = y;");
    assert!(!checker.errors.is_empty());
  }

  #[test]
  fn string_concat_infers_string() {
    let (checker, _) = check_program(r#"let x: string = "hello" + 42;"#);
    assert!(checker.errors.is_empty());
  }

  #[test]
  fn string_concat_assign_number_errors() {
    let (checker, _) = check_program(r#"let x: number = "hello" + "world";"#);
    assert!(!checker.errors.is_empty());
  }

  #[test]
  fn number_add_still_infers_number() {
    let (checker, _) = check_program("let x: number = 1 + 2;");
    assert!(checker.errors.is_empty());
  }

  #[test]
  fn template_literal_infers_string() {
    let (checker, _) = check_program(r#"let x: string = `hello`;"#);
    assert!(checker.errors.is_empty());
  }

  #[test]
  fn template_literal_assign_number_errors() {
    let (checker, _) = check_program("let x: number = `hello`;");
    assert!(!checker.errors.is_empty());
  }

  #[test]
  fn template_with_interpolation_infers_string() {
    let (checker, _) = check_program(r#"let name = "world"; let x: string = `hello ${name}`;"#);
    assert!(checker.errors.is_empty());
  }

  #[test]
  fn object_literal_infers_fields() {
    let (checker, env) = check_program(r#"let obj = { name: "hello", age: 42 };"#);
    assert!(checker.errors.is_empty());
    let obj_type = env.lookup("obj").unwrap();
    match obj_type {
      Type::Object { fields } => {
        assert_eq!(fields.len(), 2);
        assert_eq!(fields[0].0, "name");
        assert_eq!(fields[0].1, Type::Literal(LiteralValue::String("hello".into())));
        assert_eq!(fields[1].0, "age");
        assert_eq!(fields[1].1, Type::Literal(LiteralValue::Number(42.0)));
      }
      other => panic!("expected Object, got {other:?}"),
    }
  }

  #[test]
  fn member_access_returns_field_type() {
    let (checker, env) = check_program(r#"let obj = { name: "hello" }; let x: string = obj.name;"#);
    assert!(checker.errors.is_empty());
    assert_eq!(env.lookup("x"), Some(Type::String));
  }

  #[test]
  fn member_access_type_mismatch() {
    let (checker, _) = check_program(r#"let obj = { name: "hello" }; let x: number = obj.name;"#);
    assert!(!checker.errors.is_empty());
  }

  #[test]
  fn computed_member_access_string_key() {
    let (checker, _) =
      check_program(r#"let obj = { name: "hello" }; let x: string = obj["name"];"#);
    assert!(checker.errors.is_empty());
  }

  #[test]
  fn nested_object_member_access() {
    let (checker, _) = check_program(r#"let obj = { a: { b: 42 } }; let x: number = obj.a.b;"#);
    assert!(checker.errors.is_empty());
  }

  #[test]
  fn function_call_return_type() {
    let (checker, env) = check_program(
      "function add(a: number, b: number): number { return a + b; } let x = add(1, 2);",
    );
    assert!(checker.errors.is_empty());
    assert_eq!(env.lookup("x"), Some(Type::Number));
  }

  #[test]
  fn function_call_arg_mismatch() {
    let (checker, _) = check_program(
      r#"function add(a: number, b: number): number { return a + b; } let x: number = add("hello", 2);"#,
    );
    assert!(!checker.errors.is_empty());
  }

  #[test]
  fn arrow_function_call_return_type() {
    let (checker, _) = check_program(
      r#"let add = (a: number, b: number): number => a + b; let x: number = add(1, 2);"#,
    );
    assert!(checker.errors.is_empty(), "errors: {:?}", checker.errors);
  }

  #[test]
  fn import_declares_names() {
    let (checker, env) = check_program(r#"import { z } from "zod"; z;"#);
    assert!(checker.errors.is_empty(), "errors: {:?}", checker.errors);
    assert_eq!(env.lookup("z"), Some(Type::Any));
  }

  #[test]
  fn import_default_name() {
    let (checker, env) = check_program(r#"import React from "react"; React;"#);
    assert!(checker.errors.is_empty(), "errors: {:?}", checker.errors);
    assert_eq!(env.lookup("React"), Some(Type::Any));
  }

  #[test]
  fn import_multiple_names() {
    let (checker, _) = check_program(r#"import { a, b, c } from "mod"; a; b; c;"#);
    assert!(checker.errors.is_empty());
  }

  #[test]
  fn type_alias_no_errors() {
    let (checker, _) = check_program("type ID = string;");
    assert!(checker.errors.is_empty());
  }

  #[test]
  fn export_function_no_errors() {
    let (checker, _) =
      check_program("export function add(a: number, b: number): number { return a + b; }");
    assert!(checker.errors.is_empty());
  }

  #[test]
  fn export_const_no_errors() {
    let (checker, _) = check_program("export const x = 42;");
    assert!(checker.errors.is_empty());
  }

  // === Phase 37 tests ===

  #[test]
  fn instanceof_returns_boolean() {
    let (checker, _) = check_program(
      r#"class Foo { x: number = 1; }
let y: boolean = new Foo() instanceof Foo;"#,
    );
    assert!(checker.errors.is_empty(), "errors: {:?}", checker.errors);
  }

  #[test]
  fn instanceof_type_guard_narrows() {
    let (checker, _) = check_program(
      r#"class Dog { breed: string = "lab"; }
function greet(x: any) {
  if (x instanceof Dog) {
    let b: string = x.breed;
  }
}"#,
    );
    assert!(checker.errors.is_empty(), "errors: {:?}", checker.errors);
  }

  #[test]
  fn instanceof_type_guard_wrong_field_errors() {
    let (checker, _) = check_program(
      r#"class Dog { breed: string = "lab"; }
function greet(x: any) {
  if (x instanceof Dog) {
    let n: number = x.breed;
  }
}"#,
    );
    assert!(!checker.errors.is_empty());
  }

  #[test]
  fn intersection_type_merges_fields() {
    let (checker, _) = check_program(
      r#"type A = { name: string };
type B = { age: number };
type C = A & B;
let x: C = { name: "hi", age: 42 };"#,
    );
    assert!(checker.errors.is_empty(), "errors: {:?}", checker.errors);
  }

  #[test]
  fn intersection_type_partial_match_errors() {
    let (checker, _) = check_program(
      r#"type A = { name: string };
type B = { age: number };
type C = A & B;
let x: C = { name: "hi" };"#,
    );
    assert!(!checker.errors.is_empty());
  }

  #[test]
  fn optional_member_access_union() {
    let (checker, _) = check_program(
      r#"let obj: { name: string } | undefined = undefined;
let x: string = obj?.name;"#,
    );
    // obj?.name returns string | undefined, assigning to string should error
    assert!(!checker.errors.is_empty(), "should error: optional access returns T | undefined");
  }

  #[test]
  fn optional_member_access_union_ok() {
    let (checker, _) = check_program(
      r#"let obj: { name: string } | undefined = undefined;
let x: string | undefined = obj?.name;"#,
    );
    assert!(checker.errors.is_empty(), "errors: {:?}", checker.errors);
  }

  #[test]
  fn if_block_scopes_leak() {
    // Variables declared inside if-block should not leak
    let (checker, _) = check_program(
      r#"if (true) { let x = 1; }
let x: string = "outer";"#,
    );
    assert!(checker.errors.is_empty(), "errors: {:?}", checker.errors);
  }

  #[test]
  fn partial_type_preserves_fields() {
    let (checker, _) = check_program(
      r#"type T = { name: string; age: number };
let x: Partial<T> = { name: "hi" };"#,
    );
    // Partial<T> should accept subset of fields
    // In our implementation it still returns full object type, so no error
    assert!(checker.errors.is_empty(), "errors: {:?}", checker.errors);
  }

  #[test]
  fn pick_type_filters_fields() {
    let (checker, _) = check_program(
      r#"type T = { name: string; age: number; email: string };
type P = Pick<T, "name" | "email">;
let x: P = { name: "hi", email: "a@b.com" };"#,
    );
    assert!(checker.errors.is_empty(), "errors: {:?}", checker.errors);
  }

  #[test]
  fn pick_type_wrong_field_errors() {
    let (checker, _env) = check_program(
      r#"type T = { name: string; age: number; email: string };
let x: Pick<T, "name" | "email"> = { name: "hi", email: "a@b.com" };"#,
    );
    assert!(checker.errors.is_empty(), "valid Pick should have no errors");
    // Now test missing field
    let (checker2, _env2) = check_program(
      r#"type T = { name: string; age: number; email: string };
let x: Pick<T, "name" | "email"> = { name: "hi" };"#,
    );
    assert!(!checker2.errors.is_empty(), "missing email field should error");
  }

  #[test]
  fn try_catch_param_is_unknown_by_default() {
    // Catch param defaults to unknown — assigning to number should error (unknown not assignable to number)
    let (checker, _) = check_program(r#"let x = 1; try { x; } catch (e) { let n: number = e; }"#);
    assert!(
      checker.errors.iter().any(|e| matches!(e, TypeError::NotAssignable { .. })),
      "e is unknown, assigning to number should error: {:?}",
      checker.errors
    );
  }

  #[test]
  fn try_catch_typed_param_string() {
    // Catch param typed as string — assigning to string is OK
    let (checker, _) =
      check_program(r#"let x = 1; try { x; } catch (e: string) { let s: string = e; }"#);
    assert!(checker.errors.is_empty(), "errors: {:?}", checker.errors);
  }

  #[test]
  fn try_catch_typed_param_string_mismatch() {
    // Catch param typed as string — assigning to number should error
    let (checker, _) =
      check_program(r#"let x = 1; try { x; } catch (e: string) { let n: number = e; }"#);
    assert!(
      checker.errors.iter().any(|e| matches!(e, TypeError::NotAssignable { .. })),
      "e is string, assigning to number should error: {:?}",
      checker.errors
    );
  }

  // -- for-of / for-in iteration typing --

  #[test]
  fn for_of_array_infers_element_type() {
    let (checker, _) = check_program(
      r#"let arr: number[] = [1, 2, 3]; for (const x of arr) { let n: number = x; }"#,
    );
    assert!(checker.errors.is_empty(), "x should be number: {:?}", checker.errors);
  }

  #[test]
  fn for_of_array_wrong_type_errors() {
    let (checker, _) = check_program(
      r#"let arr: number[] = [1, 2, 3]; for (const x of arr) { let s: string = x; }"#,
    );
    assert!(
      checker.errors.iter().any(|e| matches!(e, TypeError::NotAssignable { .. })),
      "x is number, assigning to string should error: {:?}",
      checker.errors
    );
  }

  #[test]
  fn for_of_string_infers_string() {
    let (checker, _) =
      check_program(r#"let s: string = "hello"; for (const c of s) { let x: string = c; }"#);
    assert!(checker.errors.is_empty(), "c should be string: {:?}", checker.errors);
  }

  #[test]
  fn for_of_string_wrong_type_errors() {
    let (checker, _) =
      check_program(r#"let s: string = "hello"; for (const c of s) { let n: number = c; }"#);
    assert!(
      checker.errors.iter().any(|e| matches!(e, TypeError::NotAssignable { .. })),
      "c is string, assigning to number should error: {:?}",
      checker.errors
    );
  }

  #[test]
  fn for_in_gives_string_key() {
    let (checker, _) = check_program(
      r#"let obj: { a: number } = { a: 1 }; for (const k in obj) { let s: string = k; }"#,
    );
    assert!(checker.errors.is_empty(), "k should be string: {:?}", checker.errors);
  }

  #[test]
  fn for_in_array_gives_string_key() {
    let (checker, _) =
      check_program(r#"let arr: number[] = [1, 2]; for (const k in arr) { let s: string = k; }"#);
    assert!(checker.errors.is_empty(), "k should be string: {:?}", checker.errors);
  }

  // -- switch statement typing --

  #[test]
  fn switch_matching_case_no_error() {
    let (checker, _) = check_program(
      r#"let x: number = 1;
switch (x) {
  case 1: break;
  case 2: break;
}"#,
    );
    assert!(
      checker.errors.is_empty(),
      "matching cases should have no errors: {:?}",
      checker.errors
    );
  }

  #[test]
  fn switch_mismatched_case_type_errors() {
    let (checker, _) = check_program(
      r#"let x: number = 1;
switch (x) {
  case "hello": break;
}"#,
    );
    assert!(
      checker.errors.iter().any(|e| matches!(e, TypeError::NotAssignable { .. })),
      "string case on number discriminant should error: {:?}",
      checker.errors
    );
  }

  #[test]
  fn switch_default_no_error() {
    let (checker, _) = check_program(
      r#"let x: string = "a";
switch (x) {
  case "a": break;
  default: break;
}"#,
    );
    assert!(checker.errors.is_empty(), "default case should have no errors: {:?}", checker.errors);
  }

  #[test]
  fn switch_boolean_case_on_string_errors() {
    let (checker, _) = check_program(
      r#"let x: string = "a";
switch (x) {
  case true: break;
}"#,
    );
    assert!(
      checker.errors.iter().any(|e| matches!(e, TypeError::NotAssignable { .. })),
      "boolean case on string discriminant should error: {:?}",
      checker.errors
    );
  }

  // -- async return type resolution --

  #[test]
  fn async_fn_promise_return_unwraps_in_body() {
    // async fn with Promise<string> return → body should check return against string
    let (checker, _) = check_program(r#"async function greet(): Promise<string> { return "hi"; }"#);
    assert!(checker.errors.is_empty(), "return string in Promise<string> fn: {:?}", checker.errors);
  }

  #[test]
  fn async_fn_promise_return_mismatch_errors() {
    // async fn with Promise<string> return → returning number should error
    let (checker, _) = check_program(r#"async function greet(): Promise<string> { return 42; }"#);
    assert!(
      checker.errors.iter().any(|e| matches!(e, TypeError::NotAssignable { .. })),
      "return number in Promise<string> fn should error: {:?}",
      checker.errors
    );
  }

  #[test]
  fn async_fn_await_unwraps_promise() {
    // await on Promise<string> should give string
    let (checker, _) = check_program(
      r#"async function get(): Promise<string> { return "hi"; }
async function use() {
  let s: string = await get();
}"#,
    );
    assert!(
      checker.errors.is_empty(),
      "await Promise<string> should be string: {:?}",
      checker.errors
    );
  }

  #[test]
  fn async_fn_await_wrong_type_errors() {
    // await on Promise<string> gives string → assigning to number should error
    let (checker, _) = check_program(
      r#"async function get(): Promise<string> { return "hi"; }
async function use() {
  let n: number = await get();
}"#,
    );
    assert!(
      checker.errors.iter().any(|e| matches!(e, TypeError::NotAssignable { .. })),
      "await Promise<string> assigned to number should error: {:?}",
      checker.errors
    );
  }

  #[test]
  fn sync_fn_return_type_checked() {
    // non-async fn return type also checked now
    let (checker, _) =
      check_program(r#"function add(a: number, b: number): number { return a + b; }"#);
    assert!(checker.errors.is_empty(), "valid return should have no errors: {:?}", checker.errors);
  }

  #[test]
  fn sync_fn_return_type_mismatch_errors() {
    let (checker, _) = check_program(r#"function greet(): string { return 42; }"#);
    assert!(
      checker.errors.iter().any(|e| matches!(e, TypeError::NotAssignable { .. })),
      "return number in string fn should error: {:?}",
      checker.errors
    );
  }

  // -- Feature 5: Map/Set/Tuple iteration typing --

  #[test]
  fn for_of_set_infers_element_type() {
    let (checker, _) =
      check_program(r#"let s: Set<number>; for (const x of s) { let n: number = x; }"#);
    assert!(checker.errors.is_empty(), "x from Set<number> should be number: {:?}", checker.errors);
  }

  #[test]
  fn for_of_set_wrong_type_errors() {
    let (checker, _) =
      check_program(r#"let s: Set<number>; for (const x of s) { let str: string = x; }"#);
    assert!(
      checker.errors.iter().any(|e| matches!(e, TypeError::NotAssignable { .. })),
      "x is number from Set, assigning to string should error: {:?}",
      checker.errors
    );
  }

  #[test]
  fn for_of_map_infers_tuple() {
    let (checker, _) = check_program(
      r#"let m: Map<string, number>;
for (const entry of m) { let e: [string, number] = entry; }"#,
    );
    assert!(
      checker.errors.is_empty(),
      "Map iteration should give [string, number]: {:?}",
      checker.errors
    );
  }

  #[test]
  fn map_type_annotation() {
    let (checker, env) = check_program(r#"let m: Map<string, number>;"#);
    assert!(checker.errors.is_empty(), "Map annotation should be valid: {:?}", checker.errors);
    assert_eq!(
      env.lookup("m"),
      Some(Type::Map { key: Box::new(Type::String), value: Box::new(Type::Number) })
    );
  }

  #[test]
  fn set_type_annotation() {
    let (checker, env) = check_program(r#"let s: Set<string>;"#);
    assert!(checker.errors.is_empty(), "Set annotation should be valid: {:?}", checker.errors);
    assert_eq!(env.lookup("s"), Some(Type::Set { value: Box::new(Type::String) }));
  }

  // -- Feature 6: Destructuring typing --

  #[test]
  fn object_destructure_infers_field_types() {
    let (checker, _) = check_program(
      r#"let obj: { name: string, age: number } = { name: "hi", age: 42 };
let { name, age } = obj;
let s: string = name;
let n: number = age;"#,
    );
    assert!(
      checker.errors.is_empty(),
      "destructured fields should have correct types: {:?}",
      checker.errors
    );
  }

  #[test]
  fn object_destructure_wrong_type_errors() {
    let (checker, _) = check_program(
      r#"let obj: { name: string, age: number } = { name: "hi", age: 42 };
let { name, age } = obj;
let n: number = name;"#,
    );
    assert!(
      checker.errors.iter().any(|e| matches!(e, TypeError::NotAssignable { .. })),
      "name is string, assigning to number should error: {:?}",
      checker.errors
    );
  }

  #[test]
  fn array_destructure_from_tuple() {
    let (checker, _) = check_program(
      r#"let t: [string, number] = ["hi", 42];
let [a, b] = t;
let s: string = a;
let n: number = b;"#,
    );
    assert!(
      checker.errors.is_empty(),
      "tuple destructure should give correct types: {:?}",
      checker.errors
    );
  }

  #[test]
  fn array_destructure_from_array() {
    let (checker, _) = check_program(
      r#"let arr: number[] = [1, 2, 3];
let [x, y] = arr;
let n: number = x;"#,
    );
    assert!(
      checker.errors.is_empty(),
      "array destructure should give element type: {:?}",
      checker.errors
    );
  }

  // -- Feature 7: Enum typing --

  #[test]
  fn enum_member_access() {
    let (checker, _) = check_program(
      r#"enum Color { Red, Green, Blue }
let r = Color.Red;"#,
    );
    assert!(checker.errors.is_empty(), "enum member access should be valid: {:?}", checker.errors);
  }

  #[test]
  fn enum_member_dot_access() {
    let (checker, _) = check_program(
      r#"enum Direction { Up, Down }
let d = Direction.Up;
let n: number = d;"#,
    );
    assert!(checker.errors.is_empty(), "Direction.Up should be number: {:?}", checker.errors);
  }

  #[test]
  fn enum_wrong_member_errors() {
    let (checker, _) = check_program(
      r#"enum Color { Red, Green }
let c = Color.Red;"#,
    );
    assert!(checker.errors.is_empty(), "Color.Red is valid: {:?}", checker.errors);
  }

  // -- Feature 8: Generics --

  #[test]
  fn generic_function_infers_type_param() {
    let (checker, _) = check_program(
      r#"function identity<T>(x: T): T { return x; }
let n = identity(42);
let s = identity("hello");"#,
    );
    assert!(checker.errors.is_empty(), "generic identity should work: {:?}", checker.errors);
  }

  #[test]
  fn generic_function_type_error() {
    let (checker, _) = check_program(
      r#"function identity<T>(x: T): T { return x; }
let n: number = identity("hello");"#,
    );
    assert!(
      checker.errors.iter().any(|e| matches!(e, TypeError::NotAssignable { .. })),
      "identity(\"hello\") returns string, assigning to number should error: {:?}",
      checker.errors
    );
  }

  #[test]
  fn generic_function_multi_param() {
    let (checker, _) = check_program(
      r#"function pair<A, B>(a: A, b: B): [A, B] { return [a, b]; }
let p = pair(42, "hello");"#,
    );
    assert!(checker.errors.is_empty(), "generic pair should work: {:?}", checker.errors);
  }

  // -- Feature 9: Class member return types --

  #[test]
  fn class_method_return_type_from_annotation() {
    let (checker, _) = check_program(
      r#"class Calc {
  add(a: number, b: number): number {
    return a + b;
  }
}
let c: Calc = new Calc();
let r: number = c.add(1, 2);"#,
    );
    assert!(
      checker.errors.is_empty(),
      "annotated method return should be checked: {:?}",
      checker.errors
    );
  }

  #[test]
  fn class_method_return_type_mismatch_errors() {
    let (checker, _) = check_program(
      r#"class Calc {
  getValue(): number {
    return "hello";
  }
}"#,
    );
    assert!(
      checker.errors.iter().any(|e| matches!(e, TypeError::NotAssignable { .. })),
      "return string in number method should error: {:?}",
      checker.errors
    );
  }

  #[test]
  fn class_method_return_void_with_bare_return() {
    let (checker, _) = check_program(
      r#"class Log {
  write(msg: string): void {
    return;
  }
}"#,
    );
    assert!(
      checker.errors.is_empty(),
      "bare return in void method should be ok: {:?}",
      checker.errors
    );
  }

  #[test]
  fn class_method_no_annotation_defaults_to_any() {
    let (checker, _) = check_program(
      r#"class Foo {
  getValue() {
    return 42;
  }
}
let f: Foo = new Foo();
let x = f.getValue();
let n: number = x;"#,
    );
    assert!(checker.errors.is_empty(), "no annotation = any, should accept: {:?}", checker.errors);
  }

  // -- Feature 10: Export enum registration --

  #[test]
  fn export_enum_extracted() {
    // Verify extract_export_name handles EnumDeclaration (export doesn't error)
    let (checker, _) = check_program(r#"export enum Color { Red, Green, Blue }"#);
    assert!(checker.errors.is_empty(), "exported enum should work: {:?}", checker.errors);
  }

  // -- Feature 11: Union type narrowing --

  #[test]
  fn truthiness_narrowing_removes_null() {
    let (checker, _) = check_program(
      r#"let x: string | null = null;
if (x) {
  let s: string = x;
}"#,
    );
    assert!(
      checker.errors.is_empty(),
      "truthiness should narrow x to string: {:?}",
      checker.errors
    );
  }

  #[test]
  fn truthiness_narrowing_removes_undefined() {
    let (checker, _) = check_program(
      r#"let x: number | undefined = undefined;
if (x) {
  let n: number = x;
}"#,
    );
    assert!(
      checker.errors.is_empty(),
      "truthiness should narrow x to number: {:?}",
      checker.errors
    );
  }

  #[test]
  fn equality_null_narrows_to_null() {
    let (checker, _) = check_program(
      r#"let x: string | null = null;
if (x === null) {
  let n: null = x;
}"#,
    );
    assert!(checker.errors.is_empty(), "x === null should narrow x to null: {:?}", checker.errors);
  }

  #[test]
  fn inequality_null_narrows_out_null() {
    let (checker, _) = check_program(
      r#"let x: string | null = null;
if (x !== null) {
  let s: string = x;
}"#,
    );
    assert!(
      checker.errors.is_empty(),
      "x !== null should narrow x to string: {:?}",
      checker.errors
    );
  }

  #[test]
  fn negation_narrowing_keeps_falsy() {
    let (checker, _) = check_program(
      r#"let x: string | null = null;
if (!x) {
  let n: null = x;
}"#,
    );
    assert!(checker.errors.is_empty(), "!x should narrow to null: {:?}", checker.errors);
  }

  // -- Feature 12: Infer keyword --

  #[test]
  fn infer_keyword_parsed_as_type_param() {
    let (checker, _) = check_program(
      r#"type Unwrap = string;
let x: Unwrap = "hello";"#,
    );
    assert!(checker.errors.is_empty(), "basic type alias should work: {:?}", checker.errors);
  }

  #[test]
  fn conditional_type_with_infer() {
    let (checker, _) = check_program(
      r#"type Id<T> = T extends infer U ? U : never;
let x: Id<number> = 42;"#,
    );
    assert!(checker.errors.is_empty(), "conditional with infer should work: {:?}", checker.errors);
  }

  // -- Strict mode: noImplicitAny --

  #[test]
  fn strict_function_param_no_type_errors() {
    let (checker, _) =
      check_program_strict("function add(a: number, b: number): number { return a + b; }");
    assert!(checker.errors.is_empty(), "typed params should pass strict: {:?}", checker.errors);
  }

  #[test]
  fn strict_implicit_any_param_errors() {
    let (checker, _) = check_program_strict("function add(a, b) { return a + b; }");
    assert_eq!(checker.errors.len(), 2, "two untyped params should error");
    assert!(matches!(&checker.errors[0], TypeError::ImplicitAny { name, .. } if name == "a"));
    assert!(matches!(&checker.errors[1], TypeError::ImplicitAny { name, .. } if name == "b"));
  }

  #[test]
  fn strict_implicit_any_arrow_errors() {
    let (checker, _) = check_program_strict("let f = (x) => x;");
    assert_eq!(checker.errors.len(), 1, "one untyped arrow param should error");
    assert!(matches!(&checker.errors[0], TypeError::ImplicitAny { name, .. } if name == "x"));
  }

  #[test]
  fn strict_implicit_any_method_errors() {
    let (checker, _) = check_program_strict("class Foo { greet(name) { return name; } }");
    assert_eq!(checker.errors.len(), 1, "untyped method param should error");
    assert!(matches!(&checker.errors[0], TypeError::ImplicitAny { name, .. } if name == "name"));
  }

  #[test]
  fn strict_typed_arrow_no_errors() {
    let (checker, _) = check_program_strict("let f = (x: number): number => x;");
    assert!(checker.errors.is_empty(), "typed arrow should pass strict: {:?}", checker.errors);
  }

  #[test]
  fn non_strict_implicit_any_ok() {
    let (checker, _) = check_program("function add(a, b) { return a + b; }");
    assert!(
      checker.errors.is_empty(),
      "non-strict should allow implicit any: {:?}",
      checker.errors
    );
  }
}
