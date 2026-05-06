// tests/lexer_tests.rs
// Pruebas unitarias para el lexer.

use compilador::lexer::Token;
use logos::Logos;

/// Convierte tokens a strings para asserts
fn lex_tokens(src: &str) -> Vec<String> {
    Token::lexer(src)
        .map(|r| match r {
            Ok(t) => format!("{:?}", t),
            Err(_) => "ERR".to_string(),
        })
        .collect()
}

#[test]
fn l_01_reserved_words() {
    let src = "programa inicio fin vars entero flotante si sino mientras haz nula escribe";
    let tokens = lex_tokens(src);

    for kw in &[
        "Programa", "Inicio", "Fin", "Vars",
        "Entero", "Flotante", "Si", "Sino",
        "Mientras", "Haz", "Nula", "Escribe",
    ] {
        assert!(tokens.contains(&kw.to_string()));
    }
}

#[test]
fn l_02_identifiers() {
    for id in &["miVar", "_x", "a1", "Z"] {
        let toks = lex_tokens(id);
        assert_eq!(toks.len(), 1);
        assert!(toks[0].starts_with("Id("));
    }
}

#[test]
fn l_03_reserved_not_identifier() {
    assert_eq!(lex_tokens("si"), vec!["Si"]);
}

#[test]
fn l_04_integer_constants() {
    for n in &["0", "42", "999"] {
        let toks = lex_tokens(n);
        assert_eq!(toks.len(), 1);
        assert!(toks[0].starts_with("CteEnt("));
    }
}

#[test]
fn l_05_float_constants() {
    for n in &["3.14", "0.0", "100.001"] {
        let toks = lex_tokens(n);
        assert_eq!(toks.len(), 1);
        assert!(toks[0].starts_with("CteFlot("));
    }
}

#[test]
fn l_06_string_literal() {
    let toks = lex_tokens(r#""hola mundo 123""#);
    assert_eq!(toks.len(), 1);
    assert!(toks[0].starts_with("Letrero("));
}

#[test]
fn l_07_operators() {
    let src = "+ - * / == != < > =";
    let toks = lex_tokens(src);

    for op in &["Plus", "Minus", "Star", "Slash", "EqEq", "Neq", "Lt", "Gt", "Eq"] {
        assert!(toks.iter().any(|t| t.starts_with(op)));
    }
}

#[test]
fn l_08_punctuation() {
    let src = "; , : ( ) { } [ ]";
    let toks = lex_tokens(src);

    for p in &[
        "Semi", "Comma", "Colon",
        "LParen", "RParen",
        "LBrace", "RBrace",
        "LBracket", "RBracket",
    ] {
        assert!(toks.iter().any(|t| t.starts_with(p)));
    }
}

#[test]
fn l_09_unknown_character() {
    assert!(lex_tokens("@").contains(&"ERR".to_string()));
}