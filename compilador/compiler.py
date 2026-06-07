"""Orquestación del compilador.

Pipeline:
1. análisis léxico y sintáctico para construir el CST,
2. análisis semántico sobre el CST,
3. generación de cuádruplos sobre el CST validado.
"""

from dataclasses import dataclass
from pathlib import Path

from .parser import parse_source
from .quadruples import QuadrupleGenerator
from .semantics import SemanticAnalyzer
from .syntax_tree import SyntaxNode
from .vm import ObjectProgram, VirtualMachine


@dataclass
class CompilationResult:
    """Resultado final del pipeline de compilación."""

    tree: SyntaxNode
    semantic: SemanticAnalyzer
    quadruple_generator: QuadrupleGenerator | None = None

    def has_errors(self) -> bool:
        return self.semantic.has_errors()

    def report(self) -> str:
        return self.semantic.report()


def build_syntax_tree(src: str) -> SyntaxNode:
    """Construye el árbol sintáctico concreto a partir del código fuente."""

    return parse_source(src)


def analyze_source(src: str) -> CompilationResult:
    """Parsea y ejecuta el análisis semántico."""

    tree = build_syntax_tree(src)
    semantic = SemanticAnalyzer()
    semantic.analyze(tree)
    return CompilationResult(tree=tree, semantic=semantic, quadruple_generator=None)


def compile_source(src: str) -> CompilationResult:
    """Ejecuta el pipeline completo hasta cuádruplos."""

    result = analyze_source(src)
    if result.has_errors():
        return result
    generator = QuadrupleGenerator(result.tree, result.semantic.directorio, result.semantic.cubo)
    generator.generate()
    result.quadruple_generator = generator
    return result


def build_object_program(src: str) -> ObjectProgram:
    """Compila el código fuente y construye su representación objeto."""

    result = compile_source(src)
    if result.has_errors() or result.quadruple_generator is None:
        raise ValueError(result.report())
    return ObjectProgram.from_generator(result.quadruple_generator)


def save_object_program(src: str, path: str | Path) -> ObjectProgram:
    """Compila y serializa un archivo objeto en formato JSON."""

    program = build_object_program(src)
    program.save(path)
    return program


def run_source(src: str) -> list[str]:
    """Compila y ejecuta un programa fuente completo."""

    return VirtualMachine(build_object_program(src)).run()


def run_object_file(path: str | Path) -> list[str]:
    """Carga y ejecuta un archivo objeto previamente serializado."""

    return VirtualMachine(ObjectProgram.load(path)).run()
