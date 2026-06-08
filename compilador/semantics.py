"""Análisis semántico del compilador.

Este módulo aplica la semántica después del parser, recorriendo
el árbol sintáctico del programa.

Los comentarios con `PN-S##` señalan los puntos neurálgicos documentados.
"""

from dataclasses import dataclass, field
from enum import Enum

from .syntax_tree import (
    SyntaxLeaf,
    SyntaxNode,
    assign_expression,
    assign_target_name,
    block_statements,
    body_statements,
    call_args_node,
    call_name,
    else_body,
    func_body_node,
    func_name,
    func_params_node,
    func_return_type_node,
    func_vars_node,
    if_condition,
    if_else_node,
    if_then_body,
    leaf_value,
    program_body,
    program_funcs,
    program_name,
    program_vars,
    return_expression,
    while_body,
    while_condition,
)


DIR_SIN_ASIGNAR = -1


class TipoDato(str, Enum):
    """Tipos internos usados durante semántica y generación."""

    ENTERO = "entero"
    FLOTANTE = "flotante"
    NULA = "nula"

    @classmethod
    def from_tipo_node(cls, tipo_node: SyntaxNode) -> "TipoDato":
        # PN-S02: Conversión de tipos.
        if tipo_node.children[0].symbol == "entero":
            return cls.ENTERO
        return cls.FLOTANTE

    @classmethod
    def from_tipo_func_node(cls, tipo_func_node: SyntaxNode) -> "TipoDato":
        # PN-S02: Conversión de tipos.
        first = tipo_func_node.children[0]
        if isinstance(first, SyntaxLeaf) and first.symbol == "nula":
            return cls.NULA
        return cls.from_tipo_node(first)

    def __str__(self) -> str:
        return self.value


class Operador(str, Enum):
    SUMA = "Suma"
    RESTA = "Resta"
    MUL = "Mul"
    DIV = "Div"
    MAYOR = "Mayor"
    MENOR = "Menor"
    IGUAL = "Igual"
    DIFERENTE = "Diferente"
    ASIGNA = "Asigna"


class SemanticError(Exception):
    """Error semántico del compilador."""

    def __init__(self, kind: str, **data):
        self.kind = kind
        self.data = data
        super().__init__(str(self))

    def __str__(self) -> str:
        data = self.data
        if self.kind == "VariableDoblementeDeclada":
            return f"Variable doblemente declarada: '{data['nombre']}'"
        if self.kind == "VariableNoDeclarada":
            return f"Variable no declarada: '{data['nombre']}'"
        if self.kind == "FuncionDoblementeDeclada":
            return f"Función doblemente declarada: '{data['nombre']}'"
        if self.kind == "FuncionNoDeclarada":
            return f"Función no declarada: '{data['nombre']}'"
        if self.kind == "AmbitoNoEncontrado":
            return f"Ámbito no encontrado: '{data['nombre']}'"
        if self.kind == "TipoIncompatible":
            return f"Tipo incompatible: {data['izq']} {data['op']} {data['der']}"
        if self.kind == "ArityMismatch":
            return (
                f"Función '{data['funcion']}': se esperaban {data['esperados']} args, "
                f"se recibieron {data['recibidos']}"
            )
        if self.kind == "AsignacionTipoIncompatible":
            return (
                f"No se puede asignar '{data['expr_tipo']}' a variable "
                f"'{data['var']}' de tipo '{data['var_tipo']}'"
            )
        if self.kind == "RegresaEnFuncionNula":
            return f"La función nula '{data['funcion']}' no debe tener 'regresa'"
        if self.kind == "RegresaFueraDeFuncion":
            return "Uso de 'regresa' fuera de una función"
        if self.kind == "RetornoTipoIncompatible":
            return (
                f"Retorno incompatible en '{data['funcion']}': se esperaba "
                f"'{data['esperado']}' y se recibió '{data['recibido']}'"
            )
        if self.kind == "FaltaRegresa":
            return (
                f"La función '{data['funcion']}' debe tener al menos un "
                f"'regresa' de tipo '{data['esperado']}'"
            )
        if self.kind == "UsoFuncionNulaEnExpresion":
            return f"La función nula '{data['funcion']}' no puede usarse dentro de una expresión"
        if self.kind == "CondicionNoBooleana":
            return f"Condición no booleana en '{data['contexto']}': se recibió '{data['tipo']}'"
        if self.kind == "FuncionSinDirInicio":
            return f"La función '{data['funcion']}' no tiene dir_inicio asignada"
        return self.kind


@dataclass
class CuboSemantico:
    """Cubo semántico para validar operaciones y asignaciones."""

    tabla: dict[tuple[TipoDato, TipoDato, Operador], TipoDato] = field(default_factory=dict)

    def __post_init__(self):
        for op in (Operador.SUMA, Operador.RESTA, Operador.MUL, Operador.DIV):
            self.tabla[(TipoDato.ENTERO, TipoDato.ENTERO, op)] = TipoDato.ENTERO
            self.tabla[(TipoDato.ENTERO, TipoDato.FLOTANTE, op)] = TipoDato.FLOTANTE
            self.tabla[(TipoDato.FLOTANTE, TipoDato.ENTERO, op)] = TipoDato.FLOTANTE
            self.tabla[(TipoDato.FLOTANTE, TipoDato.FLOTANTE, op)] = TipoDato.FLOTANTE
        for op in (Operador.MAYOR, Operador.MENOR, Operador.IGUAL, Operador.DIFERENTE):
            self.tabla[(TipoDato.ENTERO, TipoDato.ENTERO, op)] = TipoDato.ENTERO
            self.tabla[(TipoDato.ENTERO, TipoDato.FLOTANTE, op)] = TipoDato.ENTERO
            self.tabla[(TipoDato.FLOTANTE, TipoDato.ENTERO, op)] = TipoDato.ENTERO
            self.tabla[(TipoDato.FLOTANTE, TipoDato.FLOTANTE, op)] = TipoDato.ENTERO
        self.tabla[(TipoDato.ENTERO, TipoDato.ENTERO, Operador.ASIGNA)] = TipoDato.ENTERO
        self.tabla[(TipoDato.FLOTANTE, TipoDato.ENTERO, Operador.ASIGNA)] = TipoDato.FLOTANTE
        self.tabla[(TipoDato.FLOTANTE, TipoDato.FLOTANTE, Operador.ASIGNA)] = TipoDato.FLOTANTE

    def consultar(self, izquierdo: TipoDato, derecho: TipoDato, operador: Operador) -> TipoDato:
        key = (izquierdo, derecho, operador)
        if key not in self.tabla:
            raise ValueError(
                f"Operación semántica inválida: {operador!r} entre '{izquierdo}' y '{derecho}'"
            )
        return self.tabla[key]


@dataclass
class EntradaVariable:
    tipo: TipoDato
    es_param: bool
    dir_virtual: int = DIR_SIN_ASIGNAR


@dataclass
class TablaVariables:
    """Tabla de símbolos para un ámbito concreto."""

    variables: dict[str, EntradaVariable] = field(default_factory=dict)

    def declarar(self, nombre: str, tipo: TipoDato, es_param: bool, dir_virtual: int) -> None:
        if nombre in self.variables:
            raise SemanticError("VariableDoblementeDeclada", nombre=nombre)
        self.variables[nombre] = EntradaVariable(tipo, es_param, dir_virtual)

    def buscar(self, nombre: str) -> EntradaVariable:
        if nombre not in self.variables:
            raise SemanticError("VariableNoDeclarada", nombre=nombre)
        return self.variables[nombre]

    def asignar_direccion(self, nombre: str, dir_virtual: int) -> None:
        self.buscar(nombre).dir_virtual = dir_virtual


@dataclass
class EntradaFuncion:
    tipo_retorno: TipoDato
    num_params: int
    dir_inicio: int
    tipos_params: list[TipoDato]
    nombres_params: list[str]
    tabla_vars: TablaVariables = field(default_factory=TablaVariables)


@dataclass
class DirectorioFunciones:
    """Directorio que agrupa funciones y el ámbito global."""

    nombre_prog: str
    funciones: dict[str, EntradaFuncion] = field(default_factory=dict)

    @classmethod
    def nuevo(cls, nombre_prog: str) -> "DirectorioFunciones":
        # PN-S01: Inicio de <Programa>.
        directorio = cls(nombre_prog=nombre_prog)
        directorio.funciones[nombre_prog] = EntradaFuncion(
            tipo_retorno=TipoDato.NULA,
            num_params=0,
            dir_inicio=0,
            tipos_params=[],
            nombres_params=[],
            tabla_vars=TablaVariables(),
        )
        return directorio

    def registrar_funcion(self, nombre: str, tipo_retorno: TipoDato, params: list[tuple[str, TipoDato]]) -> None:
        # PN-S04: Predeclaración de funciones.
        if nombre in self.funciones:
            raise SemanticError("FuncionDoblementeDeclada", nombre=nombre)
        tabla = TablaVariables()
        for nombre_param, tipo_param in params:
            tabla.declarar(nombre_param, tipo_param, True, DIR_SIN_ASIGNAR)
        if tipo_retorno != TipoDato.NULA:
            self.funciones[self.nombre_prog].tabla_vars.declarar(
                nombre, tipo_retorno, False, DIR_SIN_ASIGNAR
            )
        self.funciones[nombre] = EntradaFuncion(
            tipo_retorno=tipo_retorno,
            num_params=len(params),
            dir_inicio=0,
            tipos_params=[tipo for _, tipo in params],
            nombres_params=[nombre_param for nombre_param, _ in params],
            tabla_vars=tabla,
        )

    def declarar_variable(self, ambito: str, nombre: str, tipo: TipoDato, dir_virtual: int) -> None:
        if ambito not in self.funciones:
            raise SemanticError("AmbitoNoEncontrado", nombre=ambito)
        self.funciones[ambito].tabla_vars.declarar(nombre, tipo, False, dir_virtual)

    def buscar_funcion(self, nombre: str) -> EntradaFuncion:
        if nombre not in self.funciones:
            raise SemanticError("FuncionNoDeclarada", nombre=nombre)
        return self.funciones[nombre]

    def resolver_variable(self, ambito_local: str, nombre: str) -> TipoDato:
        # PN-S06: resolver primero en local y luego en global.
        if ambito_local in self.funciones and nombre in self.funciones[ambito_local].tabla_vars.variables:
            return self.funciones[ambito_local].tabla_vars.variables[nombre].tipo
        if ambito_local != self.nombre_prog and nombre in self.funciones[self.nombre_prog].tabla_vars.variables:
            return self.funciones[self.nombre_prog].tabla_vars.variables[nombre].tipo
        raise SemanticError("VariableNoDeclarada", nombre=nombre)

    def resolver_dir_variable(self, ambito_local: str, nombre: str) -> int:
        if ambito_local in self.funciones and nombre in self.funciones[ambito_local].tabla_vars.variables:
            return self.funciones[ambito_local].tabla_vars.variables[nombre].dir_virtual
        if ambito_local != self.nombre_prog and nombre in self.funciones[self.nombre_prog].tabla_vars.variables:
            return self.funciones[self.nombre_prog].tabla_vars.variables[nombre].dir_virtual
        raise SemanticError("VariableNoDeclarada", nombre=nombre)

    def asignar_dir_variable(self, ambito: str, nombre: str, dir_virtual: int) -> None:
        self.buscar_funcion(ambito).tabla_vars.asignar_direccion(nombre, dir_virtual)

    def asignar_dir_inicio_funcion(self, nombre: str, dir_inicio: int) -> None:
        self.buscar_funcion(nombre).dir_inicio = dir_inicio


class SemanticAnalyzer:
    """Recorre el CST y valida la semántica estática del programa."""

    def __init__(self):
        self.directorio = DirectorioFunciones.nuevo("__global__")
        self.cubo = CuboSemantico()
        self.errors: list[str] = []

    def add_error(self, error: Exception | str) -> None:
        self.errors.append(str(error))

    def analyze(self, tree: SyntaxNode) -> None:
        # PN-S01: inicializar el directorio con el nombre del programa.
        nombre_prog = program_name(tree)
        self.directorio = DirectorioFunciones.nuevo(nombre_prog)

        vars_node = program_vars(tree)
        funcs_node = program_funcs(tree)
        cuerpo_node = program_body(tree)

        # PN-S03: registrar variables globales.
        self._registrar_declaraciones(vars_node, nombre_prog)

        # PN-S04: registrar firmas de funciones antes de revisar cuerpos.
        for func_node in self._iter_funcs(funcs_node):
            self._predeclarar_funcion(func_node)

        # PN-S05 / PN-S16: revisar variables locales y cuerpo de cada función.
        for func_node in self._iter_funcs(funcs_node):
            self._analizar_funcion(func_node)

        self._analizar_cuerpo(cuerpo_node, nombre_prog)

    def has_errors(self) -> bool:
        return bool(self.errors)

    def report(self) -> str:
        if not self.errors:
            return "Análisis semántico: OK — sin errores."
        lines = [f"Análisis semántico: {len(self.errors)} error(es)"]
        lines.extend(f"  [{index}] {message}" for index, message in enumerate(self.errors, start=1))
        return "\n".join(lines)

    def _predeclarar_funcion(self, func_node: SyntaxNode) -> None:
        tipo_retorno = TipoDato.from_tipo_func_node(func_return_type_node(func_node))
        nombre = func_name(func_node)
        params = self._extraer_parametros(func_params_node(func_node))
        try:
            self.directorio.registrar_funcion(nombre, tipo_retorno, params)
        except SemanticError as exc:
            self.add_error(exc)

    def _analizar_funcion(self, func_node: SyntaxNode) -> None:
        nombre = func_name(func_node)
        vars_node = func_vars_node(func_node)
        cuerpo_node = func_body_node(func_node)

        # PN-S05: variables locales.
        self._registrar_declaraciones(vars_node, nombre)

        # PN-S16: al menos un `regresa` para funciones no nulas.
        self._analizar_cuerpo(cuerpo_node, nombre)
        try:
            entrada = self.directorio.buscar_funcion(nombre)
            if entrada.tipo_retorno != TipoDato.NULA and not self._contiene_regresa(cuerpo_node):
                self.add_error(
                    SemanticError("FaltaRegresa", funcion=nombre, esperado=str(entrada.tipo_retorno))
                )
        except SemanticError as exc:
            self.add_error(exc)

    def _registrar_declaraciones(self, vars_node: SyntaxNode, ambito: str) -> None:
        for decl in vars_node.children:
            # children[3]: tipo declarado; children[1]: lista de identificadores.
            tipo = TipoDato.from_tipo_node(decl.children[3])
            ids = self._extraer_ids(decl.children[1])
            for nombre in ids:
                try:
                    self.directorio.declarar_variable(ambito, nombre, tipo, DIR_SIN_ASIGNAR)
                except SemanticError as exc:
                    self.add_error(exc)

    def _analizar_cuerpo(self, cuerpo_node: SyntaxNode, ambito: str) -> None:
        for stmt in body_statements(cuerpo_node):
            self._analizar_estatuto(stmt, ambito)

    def _analizar_estatuto(self, stmt: SyntaxNode, ambito: str) -> None:
        symbol = stmt.symbol
        if symbol == "Asigna":
            self._validar_asignacion(stmt, ambito)
        elif symbol == "Condicion":
            self._validar_condicion(stmt, ambito)
        elif symbol == "Ciclo":
            self._validar_ciclo(stmt, ambito)
        elif symbol == "LlamadaStmt":
            self._inferir_llamada(stmt.children[0], ambito)
        elif symbol == "Regresa":
            self._validar_regresa(stmt, ambito)
        elif symbol == "Imprime":
            for alt in self._iter_imprime_alts(stmt):
                if alt.symbol == "ImprimeExpr":
                    self._inferir_expresion(alt.children[0], ambito)
        elif symbol == "Bloque":
            for nested in block_statements(stmt):
                self._analizar_estatuto(nested, ambito)

    def _validar_asignacion(self, stmt: SyntaxNode, ambito: str) -> None:
        # PN-S09: asignaciones.
        nombre = assign_target_name(stmt)
        try:
            tipo_var = self.directorio.resolver_variable(ambito, nombre)
            tipo_expr = self._inferir_expresion(assign_expression(stmt), ambito)
            self.cubo.consultar(tipo_var, tipo_expr, Operador.ASIGNA)
        except SemanticError as exc:
            self.add_error(exc)
        except ValueError:
            try:
                tipo_var = self.directorio.resolver_variable(ambito, nombre)
            except SemanticError as exc:
                self.add_error(exc)
                return
            self.add_error(
                SemanticError(
                    "AsignacionTipoIncompatible",
                    var=nombre,
                    var_tipo=str(tipo_var),
                    expr_tipo=str(self._inferir_expresion(assign_expression(stmt), ambito)),
                )
            )

    def _validar_condicion(self, stmt: SyntaxNode, ambito: str) -> None:
        # PN-S15: condición de `si`.
        tipo_cond = self._inferir_expresion(if_condition(stmt), ambito)
        if tipo_cond != TipoDato.ENTERO:
            self.add_error(SemanticError("CondicionNoBooleana", contexto=ambito, tipo=str(tipo_cond)))
        self._analizar_cuerpo(if_then_body(stmt), ambito)
        maybe_else_body = else_body(if_else_node(stmt))
        if maybe_else_body is not None:
            self._analizar_cuerpo(maybe_else_body, ambito)

    def _validar_ciclo(self, stmt: SyntaxNode, ambito: str) -> None:
        # PN-S15: condición de `mientras`.
        tipo_cond = self._inferir_expresion(while_condition(stmt), ambito)
        if tipo_cond != TipoDato.ENTERO:
            self.add_error(SemanticError("CondicionNoBooleana", contexto=ambito, tipo=str(tipo_cond)))
        self._analizar_cuerpo(while_body(stmt), ambito)

    def _validar_regresa(self, stmt: SyntaxNode, ambito: str) -> None:
        # PN-S13 / PN-S14: contexto y tipo del `regresa`.
        if ambito == self.directorio.nombre_prog:
            self.add_error(SemanticError("RegresaFueraDeFuncion"))
            return
        try:
            entrada = self.directorio.buscar_funcion(ambito)
            if entrada.tipo_retorno == TipoDato.NULA:
                self.add_error(SemanticError("RegresaEnFuncionNula", funcion=ambito))
                return
            tipo_expr = self._inferir_expresion(return_expression(stmt), ambito)
            self.cubo.consultar(entrada.tipo_retorno, tipo_expr, Operador.ASIGNA)
        except SemanticError as exc:
            self.add_error(exc)
        except ValueError:
            try:
                esperado = self.directorio.buscar_funcion(ambito).tipo_retorno
            except SemanticError as exc:
                self.add_error(exc)
                return
            self.add_error(
                SemanticError(
                    "RetornoTipoIncompatible",
                    funcion=ambito,
                    esperado=str(esperado),
                    recibido=str(self._inferir_expresion(return_expression(stmt), ambito)),
                )
            )

    def _inferir_llamada(self, llamada_node: SyntaxNode, ambito: str) -> TipoDato:
        # PN-S10 / PN-S11 / PN-S12: existencia, aridad y tipos de llamada.
        nombre = call_name(llamada_node)
        try:
            entrada = self.directorio.buscar_funcion(nombre)
        except SemanticError as exc:
            self.add_error(exc)
            return TipoDato.NULA

        args = self._extraer_argumentos(call_args_node(llamada_node))
        if len(args) != entrada.num_params:
            self.add_error(
                SemanticError(
                    "ArityMismatch",
                    funcion=nombre,
                    esperados=entrada.num_params,
                    recibidos=len(args),
                )
            )
        for index, (arg_node, tipo_param) in enumerate(zip(args, entrada.tipos_params), start=1):
            tipo_arg = self._inferir_expresion(arg_node, ambito)
            try:
                self.cubo.consultar(tipo_param, tipo_arg, Operador.ASIGNA)
            except ValueError:
                self.add_error(
                    SemanticError(
                        "TipoIncompatible",
                        op=f"arg {index} de '{nombre}'",
                        izq=str(tipo_param),
                        der=str(tipo_arg),
                    )
                )
        return entrada.tipo_retorno

    def _inferir_expresion(self, expr_node: SyntaxNode, ambito: str) -> TipoDato:
        # children[0]: parte aritmética izquierda; children[1]: operador relacional opcional.
        tipo_izq = self._inferir_exp(expr_node.children[0], ambito)
        exp_op = expr_node.children[1]
        if not exp_op.children:
            return tipo_izq
        # children[0]: operador; children[1]: parte aritmética derecha.
        tipo_der = self._inferir_exp(exp_op.children[1], ambito)
        op = {
            ">": Operador.MAYOR,
            "<": Operador.MENOR,
            "==": Operador.IGUAL,
            "!=": Operador.DIFERENTE,
        }[leaf_value(exp_op.children[0])]
        try:
            # PN-S08: operaciones relacionales.
            return self.cubo.consultar(tipo_izq, tipo_der, op)
        except ValueError:
            self.add_error(
                SemanticError("TipoIncompatible", op=op.value, izq=str(tipo_izq), der=str(tipo_der))
            )
            return tipo_izq

    def _inferir_exp(self, exp_node: SyntaxNode, ambito: str) -> TipoDato:
        # children[0]: primer término; children[1]: lista de repeticiones (+/- término).
        tipo = self._inferir_termino(exp_node.children[0], ambito)
        for item in exp_node.children[1].children:
            # children[0]: operador; children[1]: término siguiente.
            tipo_der = self._inferir_termino(item.children[1], ambito)
            op = Operador.SUMA if leaf_value(item.children[0]) == "+" else Operador.RESTA
            try:
                # PN-S07: operaciones aritméticas.
                tipo = self.cubo.consultar(tipo, tipo_der, op)
            except ValueError:
                self.add_error(
                    SemanticError("TipoIncompatible", op=op.value, izq=str(tipo), der=str(tipo_der))
                )
        return tipo

    def _inferir_termino(self, term_node: SyntaxNode, ambito: str) -> TipoDato:
        # children[0]: primer factor; children[1]: lista de repeticiones (*// factor).
        tipo = self._inferir_factor(term_node.children[0], ambito)
        for item in term_node.children[1].children:
            # children[0]: operador; children[1]: factor siguiente.
            tipo_der = self._inferir_factor(item.children[1], ambito)
            op = Operador.MUL if leaf_value(item.children[0]) == "*" else Operador.DIV
            try:
                # PN-S07: operaciones aritméticas.
                tipo = self.cubo.consultar(tipo, tipo_der, op)
            except ValueError:
                self.add_error(
                    SemanticError("TipoIncompatible", op=op.value, izq=str(tipo), der=str(tipo_der))
                )
        return tipo

    def _inferir_factor(self, factor_node: SyntaxNode, ambito: str) -> TipoDato:
        symbol = factor_node.symbol
        if symbol == "FactorParen":
            # children[1]: expresión dentro de paréntesis.
            return self._inferir_expresion(factor_node.children[1], ambito)
        if symbol == "FactorPosId" or symbol == "FactorNegId":
            # PN-S06: resolución de identificadores.
            # children[1]: nombre del identificador.
            return self.directorio.resolver_variable(ambito, leaf_value(factor_node.children[1]))
        if symbol == "FactorPosCte" or symbol == "FactorNegCte":
            # children[1]: nodo constante envuelto.
            cte = factor_node.children[1]
            return TipoDato.ENTERO if cte.symbol == "CteEnt" else TipoDato.FLOTANTE
        if symbol == "FactorId":
            # children[0]: nombre del identificador.
            return self.directorio.resolver_variable(ambito, leaf_value(factor_node.children[0]))
        if symbol == "FactorLlamada":
            # children[0]: nodo de llamada a función.
            tipo = self._inferir_llamada(factor_node.children[0], ambito)
            if tipo == TipoDato.NULA:
                self.add_error(
                    SemanticError(
                        "UsoFuncionNulaEnExpresion",
                        funcion=call_name(factor_node.children[0]),
                    )
                )
            return tipo
            # children[0]: nodo CteEnt o CteFlt.
        cte = factor_node.children[0]
        return TipoDato.ENTERO if cte.symbol == "CteEnt" else TipoDato.FLOTANTE

    def _contiene_regresa(self, cuerpo_node: SyntaxNode) -> bool:
        for stmt in body_statements(cuerpo_node):
            if stmt.symbol == "Regresa":
                return True
            if stmt.symbol == "Condicion":
                if self._contiene_regresa(if_then_body(stmt)):
                    return True
                maybe_else_body = else_body(if_else_node(stmt))
                if maybe_else_body is not None and self._contiene_regresa(maybe_else_body):
                    return True
            if stmt.symbol == "Ciclo" and self._contiene_regresa(while_body(stmt)):
                return True
            if stmt.symbol == "Bloque":
                bloque_cuerpo = SyntaxNode("Cuerpo", [stmt.children[0], stmt.children[1], stmt.children[2]])
                if self._contiene_regresa(bloque_cuerpo):
                    return True
        return False

    def _iter_funcs(self, funcs_node: SyntaxNode) -> list[SyntaxNode]:
        return [child for child in funcs_node.children if isinstance(child, SyntaxNode)]

    def _iter_imprime_alts(self, imprime_node: SyntaxNode) -> list[SyntaxNode]:
        alts: list[SyntaxNode] = []
        for child in imprime_node.children[2:-2]:
            if isinstance(child, SyntaxNode) and child.symbol in {"ImprimeExpr", "ImprimeStr"}:
                alts.append(child)
        return alts

    def _extraer_ids(self, id_list_node: SyntaxNode) -> list[str]:
        ids: list[str] = []
        for child in id_list_node.children:
            if isinstance(child, SyntaxLeaf) and child.symbol == "id":
                ids.append(child.value)
            elif isinstance(child, SyntaxNode):
                ids.extend(self._extraer_ids(child))
        return ids

    def _extraer_parametros(self, params_node: SyntaxNode) -> list[tuple[str, TipoDato]]:
        params: list[tuple[str, TipoDato]] = []
        for child in params_node.children:
            if child.symbol == "Param":
                # children[0]: tipo; children[2]: nombre del parámetro.
                params.append((leaf_value(child.children[0]), TipoDato.from_tipo_node(child.children[2])))
            elif child.symbol == "ParamListItem":
                # children[1]: nombre del parámetro; children[3]: tipo del parámetro.
                params.append((leaf_value(child.children[1]), TipoDato.from_tipo_node(child.children[3])))
        return params

    def _extraer_argumentos(self, args_node: SyntaxNode) -> list[SyntaxNode]:
        args: list[SyntaxNode] = []
        for child in args_node.children:
            if isinstance(child, SyntaxNode) and child.symbol == "Expresion":
                args.append(child)
            elif isinstance(child, SyntaxNode) and child.symbol == "ArgsList":
                args.extend(self._extraer_argumentos(child))
        return args
