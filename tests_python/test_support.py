"""Helpers compartidos para la suite de pruebas."""

from compilador.compiler import build_syntax_tree
from compilador.lexer import PatitoLexer


def token_types(src: str) -> list[str]:
    """Extrae solo los nombres de token para asserts simples."""

    lexer = PatitoLexer()
    return [tok.type for tok in lexer.tokenize(src)]


def parse_ok(src: str):
    """Construye el CST o lanza la excepción del parser."""

    return build_syntax_tree(src)
