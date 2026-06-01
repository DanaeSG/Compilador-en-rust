// tests/parser_tests.rs
// Pruebas unitarias para el parser.

use compilador::parse;

#[test]
fn p_01_minimal_program() {
    assert!(parse("programa p; inicio { } fin").is_ok());
}

#[test]
fn p_02_var_declarations() {
    let ast = parse("programa v; vars a, b : entero; inicio { } fin").unwrap();
    assert_eq!(ast.vars.len(), 1);
}

#[test]
fn p_03_assignment() {
    assert!(parse("programa p; inicio { x = 10; } fin").is_ok());
}

#[test]
fn p_04_arithmetic_expressions() {
    assert!(parse("programa p; inicio { x = 1 + 2 * 3; } fin").is_ok());
}

#[test]
fn p_05_relational_expression() {
    assert!(parse("programa rel; inicio { r = x > y; } fin").is_ok());
}

#[test]
fn p_06_if_without_else() {
    let ast = parse(
        "programa p; inicio { si (x > 0) { x = 1; }; } fin"
    ).unwrap();

    match &ast.cuerpo[0] {
        compilador::ast::Estatuto::Condicion { sino, .. } => {
            assert!(sino.is_none());
        }
        _ => panic!("Expected Condicion"),
    }
}

#[test]
fn p_07_if_else() {
    let ast = parse(
        "programa p; inicio { si (x > 0) { x = 1; } sino { x = 2; }; } fin"
    ).unwrap();

    match &ast.cuerpo[0] {
        compilador::ast::Estatuto::Condicion { sino, .. } => {
            assert!(sino.is_some());
        }
        _ => panic!("Expected Condicion"),
    }
}

#[test]
fn p_08_while_loop() {
    assert!(parse(
        "programa p; inicio { mientras (x < 10) haz { x = x + 1; }; } fin"
    ).is_ok());
}

#[test]
fn p_09_write_statement() {
    assert!(parse(r#"programa imp; inicio { escribe("res: ", x); } fin"#).is_ok());
}

#[test]
fn p_10_function_definition() {
    let ast = parse(
        "programa fn1; nula suma(a : entero, b : entero) { { escribe(a); } }; inicio { } fin"
    ).unwrap();

    assert_eq!(ast.funcs.len(), 1);
    assert_eq!(ast.funcs[0].params.len(), 2);
}

#[test]
fn p_11_function_call() {
    assert!(parse(
        "programa fn2; entero doble(n : entero) { { r = n + n; } }; inicio { resultado = doble(5); } fin"
    ).is_ok());
}

#[test]
fn p_12_parenthesized_expression() {
    assert!(parse("programa par; inicio { z = (a + b) * c; } fin").is_ok());
}

#[test]
fn p_13_unary_negation() {
    assert!(parse("programa neg; inicio { z = -x; } fin").is_ok());
}

#[test]
fn p_14_block_statement() {
    assert!(parse("programa blq; inicio { [ x = 1; y = 2; ] } fin").is_ok());
}

#[test]
fn p_15_function_call_statement() {
    assert!(parse(
        "programa call; nula greet() { { escribe(\"hi\"); } }; inicio { greet(); } fin"
    ).is_ok());
}

#[test]
fn p_16_return_statement() {
    assert!(parse(
        "programa ret; entero id(a:entero) { { regresa a; } }; inicio { } fin"
    ).is_ok());
}
