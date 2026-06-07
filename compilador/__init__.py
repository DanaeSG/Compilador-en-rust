"""Entradas públicas del compilador."""

from .compiler import (
    analyze_source,
    build_object_program,
    build_syntax_tree,
    compile_source,
    run_object_file,
    run_source,
    save_object_program,
)
from .vm import ObjectProgram, VirtualMachine, VirtualMachineError

__all__ = [
    "build_syntax_tree",
    "analyze_source",
    "compile_source",
    "build_object_program",
    "save_object_program",
    "run_source",
    "run_object_file",
    "ObjectProgram",
    "VirtualMachine",
    "VirtualMachineError",
]
