// src/ast.rs
// Definición del Árbol de Sintaxis Abstracta (AST)
// Cada nodo corresponde directamente a un no terminal de la gramática (CFG)
// definida. El diseño prioriza tipado fuerte y facilidad de recorrido.

// Programa (símbolo inicial)
#[derive(Debug, Clone)]
pub struct Programa {
    pub nombre: String,        // Identificador del programa (después de 'programa')
    pub vars: Vec<DeclVars>,   // Declaraciones globales (<VARS>)
    pub funcs: Vec<Funcion>,   // Definición de funciones (<FUNCS>)
    pub cuerpo: Vec<Estatuto>, // Bloque principal (<CUERPO>: inicio … fin)
}

// Declaraciones de variables
#[derive(Debug, Clone)]
pub struct DeclVars {
    pub ids: Vec<String>,  // Lista de identificadores: id, id, ...
    pub tipo: Tipo,        // Tipo asociado a todos los identificadores
}

// Tipos primitivos soportados por el lenguaje
#[derive(Debug, Clone)]
pub enum Tipo {
    Entero,
    Flotante,
}

// Funciones
#[derive(Debug, Clone)]
pub struct Funcion {
    pub tipo_retorno: TipoFunc, // Tipo de retorno (o nula)
    pub nombre: String,         // Identificador de la función
    pub params: Vec<Param>,     // Parámetros formales
    pub vars: Vec<DeclVars>,    // Variables locales
    pub cuerpo: Vec<Estatuto>,  // Cuerpo de la función
}

// Tipo de función: puede ser nula (void) o tipada
#[derive(Debug, Clone)]
pub enum TipoFunc {
    Nula,
    Tipo(Tipo),
}

// Parámetro formal (nombre + tipo)
#[derive(Debug, Clone)]
pub struct Param {
    pub nombre: String,
    pub tipo: Tipo,
}

// Estatutos (sentencias)
#[derive(Debug, Clone)]
pub enum Estatuto {
    // Asignación: id = expresión;
    Asigna(String, Box<Expresion>),

    // Condicional: si (cond) { ... } [sino { ... }]
    Condicion {
        cond: Box<Expresion>,
        entonces: Vec<Estatuto>,
        sino: Option<Vec<Estatuto>>, // None = sin else
    },

    // Ciclo while: mientras (cond) haz { ... }
    Ciclo {
        cond: Box<Expresion>,
        cuerpo: Vec<Estatuto>,
    },

    // Llamada a función como sentencia
    Llamada(Llamada),

    // Instrucción de impresión: mezcla de expresiones y literales
    Imprime(Vec<ImprimeAlt>),

    // Bloque explícito: [ ... ]
    Bloque(Vec<Estatuto>),
}

// Alternativas dentro de imprime (expresiones o strings)
#[derive(Debug, Clone)]
pub enum ImprimeAlt {
    Expr(Expresion), // Evaluar e imprimir expresión
    Letrero(String), // Literal string
}

// Llamada a función
#[derive(Debug, Clone)]
pub struct Llamada {
    pub nombre: String,        // Nombre de la función
    pub args: Vec<Expresion>,  // Argumentos
}

// Expresiones

// Expresión relacional opcional: Exp [opRel Exp]
#[derive(Debug, Clone)]
pub struct Expresion {
    pub izq: Exp,                          // Parte izquierda
    pub op: Option<(OpRel, Exp)>,          // Operador relacional + operando derecho
}

// Operadores relacionales soportados
#[derive(Debug, Clone)]
pub enum OpRel {
    Gt,   // >
    Lt,   // <
    Neq,  // !=
    EqEq, // ==
}

// Expresión aritmética (suma/resta)
// Representa: Termino ( ( + | - ) Termino )*
#[derive(Debug, Clone)]
pub struct Exp {
    pub termino: Termino,                  // Primer término
    pub cont: Vec<(OpArit, Termino)>,      // Lista de operaciones adicionales
}

// Operadores aritméticos de bajo nivel
#[derive(Debug, Clone)]
pub enum OpArit {
    Plus,  // +
    Minus, // -
}

// Término (multiplicación/división)
// Representa: Factor ( ( * | / ) Factor )*
#[derive(Debug, Clone)]
pub struct Termino {
    pub factor: Factor,                    // Primer factor
    pub cont: Vec<(OpMul, Factor)>,        // Operaciones adicionales
}

// Operadores de multiplicación/división
#[derive(Debug, Clone)]
pub enum OpMul {
    Star,  // *
    Slash, // /
}

// Factor (unidad básica de expresión)
#[derive(Debug, Clone)]
pub enum Factor {
    Paren(Box<Expresion>), // ( expresión )
    PosId(String),         // +id
    NegId(String),         // -id
    Id(String),            // identificador
    Cte(Constante),        // constante numérica
    Llamada(Llamada),      // llamada a función como expresión
}

// Constantes numéricas
#[derive(Debug, Clone)]
pub enum Constante {
    Entero(i64),
    Flotante(f64),
}