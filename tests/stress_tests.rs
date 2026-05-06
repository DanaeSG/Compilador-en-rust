// tests/stress_tests.rs
// Pruebas de estrés para el compilador.

use compilador::parse;

#[test]
fn s_01_deep_nesting() {
    let mut src = String::from("programa p;\ninicio {");

    for _ in 0..25 {
        src.push_str(" si (x > 0) {");
    }

    for _ in 0..25 {
        src.push_str(" };");
    }

    src.push_str(" } fin");

    assert!(parse(&src).is_ok());
}

#[test]
fn s_02_large_var_list() {
    let vars: String = (0..100)
        .map(|i| if i == 0 {
            "a".to_string()
        } else {
            format!("v{}", i)
        })
        .collect::<Vec<_>>()
        .join(", ");

    let src = format!(
        "programa p; vars {} : entero; inicio {{ }} fin",
        vars
    );

    assert!(parse(&src).is_ok());
}

#[test]
fn s_03_complex_expression() {
    let src = "programa p; inicio { x = 1 + 2 * 3 - 4 / (2 + 1) == 5; } fin";
    assert!(parse(src).is_ok());
}