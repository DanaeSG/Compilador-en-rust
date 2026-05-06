// tests/error_tests.rs
// Pruebas de errores para el compilador.

use compilador::parse;

#[test]
fn e_01_missing_fin() {
    assert!(parse("programa p; inicio { x = 1; }").is_err());
}

#[test]
fn e_02_missing_semicolon() {
    assert!(parse("programa p; inicio { x = 1 } fin").is_err());
}

#[test]
fn e_03_mismatched_parentheses() {
    assert!(parse("programa p; inicio { x = (1 + 2; } fin").is_err());
}

#[test]
fn e_04_invalid_type() {
    assert!(parse("programa p; vars x : real; inicio { } fin").is_err());
}

#[test]
fn e_05_else_without_if() {
    assert!(parse("programa p; inicio { sino { x = 1; } } fin").is_err());
}

#[test]
fn e_06_invalid_call() {
    assert!(parse("programa p; inicio { f(,1); } fin").is_err());
}

#[test]
fn e_07_empty_program() {
    assert!(parse("").is_err());
}