"""CLI principal del compilador."""

import sys
from pathlib import Path

from compilador.compiler import (
    analyze_source,
    build_syntax_tree,
    compile_source,
    run_object_file,
    run_source,
    save_object_program,
)
from compilador.lexer import LexicalError, iter_tokens
from compilador.vm import VirtualMachineError


TEST_PROGRAMS = [
    "programas_prueba/p1_asignaciones.patito",
    "programas_prueba/p2_relacionales.patito",
    "programas_prueba/p3_parentesis.patito",
    "programas_prueba/p4_imprime.patito",
    "programas_prueba/p5_unarios.patito",
    "programas_prueba/p6_stress_lineal.patito",
    "programas_prueba/p7_nonlineal.patito",
    "programas_prueba/p8_si_sino.patito",
    "programas_prueba/p9_mientras_anidado.patito",
    "programas_prueba/p10_funciones_return.patito",
    "programas_prueba/p11_control_y_funcion.patito",
    "programas_prueba/p12.patito",
    "programas_prueba/p13_factorial_main.patito",
    "programas_prueba/p14_factorial_funcion.patito",
    "programas_prueba/p15_fibonacci_main.patito",
    "programas_prueba/p16_fibonacci_funcion.patito",
]


def dump_lexer(src: str) -> None:
    """Imprime tokens con su rango aproximado."""

    try:
        for start, token, end in iter_tokens(src):
            print(f"[{start}..{end}] {token.type}({token.value!r})")
    except LexicalError as exc:
        print(f"LEX_ERR {exc}")


def run_compiler(
    src: str,
    show_st: bool,
    show_lex: bool,
    show_sem: bool,
    show_quad: bool,
    run_vm: bool,
    emit_obj: str | None,
) -> int:
    """Ejecuta los pasos visibles del compilador."""

    try:
        if show_lex:
            print("Lexer:")
            dump_lexer(src)

        tree = None
        if show_st:
            tree = build_syntax_tree(src)
            print("Árbol sintáctico:")
            print(tree)

        result = analyze_source(src)
        if show_sem:
            print(result.report())

        if result.has_errors():
            return 1

        if show_quad:
            compiled = compile_source(src)
            print("Cuadruplos:")
            if compiled.quadruple_generator is not None:
                print(compiled.quadruple_generator.quadruples.dump(), end="")
        if emit_obj is not None:
            save_object_program(src, emit_obj)
            print(f"Archivo objeto generado en: {emit_obj}")
        if run_vm:
            print("VM:")
            for line in run_source(src):
                print(line)
        return 0
    except (SyntaxError, LexicalError, VirtualMachineError, ValueError) as exc:
        print(str(exc), file=sys.stderr)
        return 1


def main(argv: list[str] | None = None) -> int:
    args = list(sys.argv[1:] if argv is None else argv)
    path: str | None = None
    obj_path: str | None = None
    show_st = False
    show_lex = False
    show_sem = False
    show_quad = False
    run_vm = False
    has_output_flag = False
    emit_obj: str | None = None

    index = 0
    while index < len(args):
        arg = args[index]
        if arg == "--st":
            show_st = True
            has_output_flag = True
        elif arg == "--lex":
            show_lex = True
            has_output_flag = True
        elif arg == "--sem":
            show_sem = True
            has_output_flag = True
        elif arg == "--quad":
            show_quad = True
            has_output_flag = True
        elif arg == "--run":
            run_vm = True
            has_output_flag = True
        elif arg == "--run-obj":
            index += 1
            if index >= len(args):
                print("Falta la ruta del archivo objeto para --run-obj", file=sys.stderr)
                return 1
            obj_path = args[index]
        elif arg == "--emit-obj":
            index += 1
            if index >= len(args):
                print("Falta la ruta de salida para --emit-obj", file=sys.stderr)
                return 1
            emit_obj = args[index]
        elif path is None:
            path = arg
        index += 1

    if obj_path is not None:
        try:
            for line in run_object_file(obj_path):
                print(line)
            return 0
        except (VirtualMachineError, ValueError) as exc:
            print(str(exc), file=sys.stderr)
            return 1

    if not has_output_flag:
        run_vm = True

    if path is not None:
        src = Path(path).read_text(encoding="utf-8")
        return run_compiler(src, show_st, show_lex, show_sem, show_quad, run_vm, emit_obj)

    exit_code = 0
    for test_path in TEST_PROGRAMS:
        print(f"== {test_path} ==")
        src = Path(test_path).read_text(encoding="utf-8")
        exit_code = max(
            exit_code,
            run_compiler(src, show_st, show_lex, show_sem, show_quad, run_vm, emit_obj),
        )
    return exit_code


if __name__ == "__main__":
    raise SystemExit(main())
