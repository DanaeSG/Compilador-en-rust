// src/lexer.rs
// Analizador léxico (scanner) implementado con Logos.
// Convierte la entrada en una secuencia de tokens tipados según la especificación.
// La salida es consumida por LALRPOP en forma de triples (start, token, end).

use logos::Logos;

/// Definición de todos los tokens del lenguaje.
///
/// Diseño:
/// - Se utiliza `#[logos(skip ...)]` para ignorar whitespace automáticamente.
/// - Orden de definición:
///     - Palabras reservadas antes que identificadores
///     - Flotantes antes que enteros
/// - Los tokens con datos (payload) usan callbacks para parsear valores.
#[derive(Logos, Debug, Clone, PartialEq)]
#[logos(skip r"[ \t\r\n]+")] // Ignora espacios, tabs y saltos de línea
pub enum Token {

    // Palabras reservadas 
    #[token("programa")]  Programa,
    #[token("inicio")]    Inicio,
    #[token("fin")]       Fin,
    #[token("nula")]      Nula,
    #[token("escribe")]   Escribe,
    #[token("mientras")]  Mientras,
    #[token("haz")]       Haz,
    #[token("si")]        Si,
    #[token("sino")]      Sino,
    #[token("vars")]      Vars,
    #[token("entero")]    Entero,
    #[token("flotante")]  Flotante,

    // Identificadores
    // (letter | _)(letter | _ | digit)*
    // Deben ir después de palabras reservadas para respetar prioridad.
    #[regex(r"[a-zA-Z_][a-zA-Z_0-9]*", |lex| lex.slice().to_string())]
    Id(String),

    // Constantes numéricas
    // Nota: flotantes antes que enteros para evitar tokenización incorrecta.
    #[regex(r"[0-9]+\.[0-9]+", |lex| lex.slice().parse::<f64>().ok())]
    CteFlot(f64),

    #[regex(r"[0-9]+", |lex| lex.slice().parse::<i64>().ok())]
    CteEnt(i64),

    // Strings
    // Formato: " ... "
    // Se eliminan las comillas en el callback.
    #[regex(r#""[^"]*""#, |lex| {
        let s = lex.slice();
        s[1..s.len()-1].to_string() 
    })]
    Letrero(String),

    // Operadores
    #[token("+")]   Plus,
    #[token("-")]   Minus,
    #[token("*")]   Star,
    #[token("/")]   Slash,
    #[token("==")]  EqEq,
    #[token("!=")]  Neq,
    #[token("<")]   Lt,
    #[token(">")]   Gt,
    #[token("=")]   Eq,

    // Puntuación
    #[token(";")]   Semi,
    #[token(",")]   Comma,
    #[token(":")]   Colon,
    #[token("(")]   LParen,
    #[token(")")]   RParen,
    #[token("{")]   LBrace,
    #[token("}")]   RBrace,
    #[token("[")]   LBracket,
    #[token("]")]   RBracket,
}

pub struct Lexer<'input> {
    inner: logos::SpannedIter<'input, Token>,
    src: &'input str,
}

impl<'input> Lexer<'input> {
    /// Crear lexer desde string fuente
    pub fn new(src: &'input str) -> Self {
        Self {
            inner: Token::lexer(src).spanned(),
            src,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LexError {
    pub span: (usize, usize),
    pub slice: String,
}

impl std::fmt::Display for LexError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Token no reconocido: {:?} en bytes {}..{}",
            self.slice, self.span.0, self.span.1
        )
    }
}

impl<'input> Iterator for Lexer<'input> {
    type Item = Result<(usize, Token, usize), LexError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|(result, span)| match result {
            Ok(tok) => Ok((span.start, tok, span.end)),

            Err(_) => {
                let slice = self.src[span.start..span.end].to_string();

                Err(LexError {
                    span: (span.start, span.end),
                    slice,
                })
            }
        })
    }
}