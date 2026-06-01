// Entrada principal de la libreria.
// Exporta lexer, AST, semantica, analisis semantico y cuadruplos.

pub mod lexer;
pub mod ast;
pub mod semantica;
pub mod analizador_semantico;
pub mod cuadruplos;
lalrpop_util::lalrpop_mod!(pub gramatica, "/gramatica.rs");

use lexer::Lexer;

// Convierte codigo fuente a AST.
pub fn parse(src: &str) -> Result<ast::Programa, String> {
    let lexer = Lexer::new(src);

    gramatica::ProgramaParser::new()
        .parse(lexer)
        .map_err(|e| format!("{:?}", e)) 
}
