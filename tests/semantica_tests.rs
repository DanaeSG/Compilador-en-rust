// tests/semantica_tests.rs
// Pruebas unitarias para el analizador semantico.

use compilador::analizador_semantico::AnalizadorSemantico;
use compilador::parse;
use compilador::semantica::{
    CuboSemantico, DirectorioFunciones, ErrorSemantico, Operador, TablaVariables, TipoDato,
};

fn analizar(src: &str) -> AnalizadorSemantico {
    let ast = parse(src).expect("parse failed");
    let mut sem = AnalizadorSemantico::new();
    sem.analizar(&ast);
    sem
}

//  Cubo semantico 
#[test]
fn cs_01_int_plus_int() {
    let c = CuboSemantico::new();
    assert_eq!(
        c.consultar(&TipoDato::Entero, &TipoDato::Entero, &Operador::Suma)
            .unwrap(),
        TipoDato::Entero
    );
}

#[test]
fn cs_02_int_plus_float() {
    let c = CuboSemantico::new();
    assert_eq!(
        c.consultar(&TipoDato::Entero, &TipoDato::Flotante, &Operador::Suma)
            .unwrap(),
        TipoDato::Flotante
    );
}

#[test]
fn cs_03_relacional_result_es_entero() {
    let c = CuboSemantico::new();
    assert_eq!(
        c.consultar(&TipoDato::Flotante, &TipoDato::Flotante, &Operador::Mayor)
            .unwrap(),
        TipoDato::Entero
    );
}

#[test]
fn cs_04_asigna_float_to_float_ok() {
    let c = CuboSemantico::new();
    assert!(
        c.consultar(&TipoDato::Flotante, &TipoDato::Flotante, &Operador::Asigna)
            .is_ok()
    );
}

#[test]
fn cs_05_asigna_float_to_int_error() {
    let c = CuboSemantico::new();
    assert!(
        c.consultar(&TipoDato::Entero, &TipoDato::Flotante, &Operador::Asigna)
            .is_err()
    );
}

//  Tabla de Variables 
#[test]
fn tv_01_declarar_ok() {
    let mut t = TablaVariables::new();
    assert!(t.declarar("x", TipoDato::Entero, false, -1).is_ok());
}

#[test]
fn tv_02_doble_declaracion() {
    let mut t = TablaVariables::new();
    t.declarar("x", TipoDato::Entero, false, -1).unwrap();
    let r = t.declarar("x", TipoDato::Flotante, false, -1);
    assert!(matches!(r, Err(ErrorSemantico::VariableDoblementeDeclada(_))));
}

#[test]
fn tv_03_buscar_no_declarada() {
    let t = TablaVariables::new();
    assert!(matches!(
        t.buscar("x"),
        Err(ErrorSemantico::VariableNoDeclarada(_))
    ));
}

#[test]
fn tv_04_buscar_ok() {
    let mut t = TablaVariables::new();
    t.declarar("y", TipoDato::Flotante, false, -1).unwrap();
    assert_eq!(t.buscar("y").unwrap().tipo, TipoDato::Flotante);
}

// Directorio de Funciones 
#[test]
fn df_01_registrar_funcion() {
    let mut d = DirectorioFunciones::new("prog");
    assert!(
        d.registrar_funcion(
            "suma",
            TipoDato::Entero,
            vec![("a".to_string(), TipoDato::Entero)],
        )
        .is_ok()
    );
}

#[test]
fn df_02_funcion_doble_declaracion() {
    let mut d = DirectorioFunciones::new("prog");
    d.registrar_funcion("f", TipoDato::Nula, vec![]).unwrap();
    let r = d.registrar_funcion("f", TipoDato::Nula, vec![]);
    assert!(matches!(r, Err(ErrorSemantico::FuncionDoblementeDeclada(_))));
}

#[test]
fn df_03_buscar_funcion_no_declarada() {
    let d = DirectorioFunciones::new("prog");
    assert!(matches!(
        d.buscar_funcion("f"),
        Err(ErrorSemantico::FuncionNoDeclarada(_))
    ));
}

#[test]
fn df_04_resolver_variable_global() {
    let mut d = DirectorioFunciones::new("prog");
    d.declarar_variable("prog", "x", TipoDato::Entero, -1).unwrap();
    d.registrar_funcion("f", TipoDato::Nula, vec![]).unwrap();
    assert_eq!(d.resolver_variable("f", "x").unwrap(), TipoDato::Entero);
}

#[test]
fn df_05_var_local_oculta_global() {
    let mut d = DirectorioFunciones::new("prog");
    d.declarar_variable("prog", "x", TipoDato::Entero, -1).unwrap();
    d.registrar_funcion("f", TipoDato::Nula, vec![]).unwrap();
    d.declarar_variable("f", "x", TipoDato::Flotante, -1).unwrap();
    assert_eq!(d.resolver_variable("f", "x").unwrap(), TipoDato::Flotante);
}

//  Analisis semantico end-to-end 
#[test]
fn sem_01_programa_correcto() {
    let src = r#"programa t;
vars x, y : entero;
inicio { x = 1; y = x + 2; } fin"#;
    let sem = analizar(src);
    assert!(!sem.tiene_errores(), "{}", sem.reporte());
}

#[test]
fn sem_02_variable_no_declarada() {
    let src = "programa t; inicio { x = 1; } fin";
    let sem = analizar(src);
    assert!(sem.tiene_errores());
    assert!(sem.reporte().contains("no declarada"));
}

#[test]
fn sem_03_variable_doble_declaracion() {
    let src = "programa t; vars x:entero; vars x:flotante; inicio { } fin";
    let sem = analizar(src);
    assert!(sem.tiene_errores());
    assert!(sem.reporte().contains("doblemente declarada"));
}

#[test]
fn sem_04_asignacion_float_a_entero_error() {
    let src = "programa t; vars x:entero; vars y:flotante; inicio { x = y; } fin";
    let sem = analizar(src);
    assert!(sem.tiene_errores(), "deberia detectar asignacion flotante->entero");
}

#[test]
fn sem_05_asignacion_entero_a_float_ok() {
    let src = "programa t; vars x:flotante; vars y:entero; inicio { x = y; } fin";
    let sem = analizar(src);
    assert!(!sem.tiene_errores(), "{}", sem.reporte());
}

#[test]
fn sem_06_funcion_no_declarada() {
    let src = "programa t; inicio { foo(); } fin";
    let sem = analizar(src);
    assert!(sem.tiene_errores());
    assert!(sem.reporte().contains("no declarada"));
}

#[test]
fn sem_07_aridad_incorrecta() {
    let src = r#"programa t;
nula f(a:entero) { { escribe(a); } };
inicio { f(1, 2); } fin"#;
    let sem = analizar(src);
    assert!(sem.tiene_errores());
    assert!(sem.reporte().contains("args"));
}

#[test]
fn sem_08_funcion_doble_declaracion() {
    let src = r#"programa t;
nula f() { { escribe("a"); } };
nula f() { { escribe("b"); } };
inicio { } fin"#;
    let sem = analizar(src);
    assert!(sem.tiene_errores());
    assert!(sem.reporte().contains("doblemente declarada"));
}

#[test]
fn sem_09_llamada_correcta() {
    let src = r#"programa t;
vars r : entero;
entero doble(n:entero) { { regresa n + n; } };
inicio { r = doble(5); } fin"#;
    let sem = analizar(src);
    assert!(!sem.tiene_errores(), "{}", sem.reporte());
}

#[test]
fn sem_10_var_local_no_visible_fuera() {
    let src = r#"programa t;
nula f() { vars local:entero; { local = 1; } };
inicio { local = 2; } fin"#;
    let sem = analizar(src);
    assert!(sem.tiene_errores(), "local no deberia ser visible en main");
}

#[test]
fn sem_11_regresa_tipo_correcto() {
    let src = r#"programa t;
entero id(a:entero) { { regresa a; } };
inicio { } fin"#;
    let sem = analizar(src);
    assert!(!sem.tiene_errores(), "{}", sem.reporte());
}

#[test]
fn sem_12_regresa_tipo_incorrecto() {
    let src = r#"programa t;
entero id(a:flotante) { { regresa a; } };
inicio { } fin"#;
    let sem = analizar(src);
    assert!(sem.tiene_errores());
    assert!(sem.reporte().contains("Retorno incompatible"));
}

#[test]
fn sem_13_regresa_en_funcion_nula_error() {
    let src = r#"programa t;
nula f() { { regresa 1; } };
inicio { } fin"#;
    let sem = analizar(src);
    assert!(sem.tiene_errores());
    assert!(sem.reporte().contains("no debe tener 'regresa'"));
}

#[test]
fn sem_14_falta_regresa_en_funcion_tipada() {
    let src = r#"programa t;
entero f() { { escribe("hola"); } };
inicio { } fin"#;
    let sem = analizar(src);
    assert!(sem.tiene_errores());
    assert!(sem.reporte().contains("debe tener al menos un 'regresa'"));
}
