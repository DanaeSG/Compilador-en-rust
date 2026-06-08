"""Cuádruplos, memoria virtual y generación intermedia.

Distribución de memoria virtual:
- Variables globales enteras:   1000+
- Variables globales flotantes: 2000+
- Variables locales enteras:    3000+
- Variables locales flotantes:  4000+
- Temporales enteros:           5000+
- Temporales flotantes:         6000+
- Constantes enteras:           8000+
- Constantes flotantes:         9000+
- Constantes string:            10000+

Los comentarios `PN-L##` y `PN-NL##` marcan los puntos neurálgicos de
generación de código intermedio.
"""

from dataclasses import dataclass, field

from .semantics import CuboSemantico, DirectorioFunciones, Operador, SemanticError, TipoDato
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
    if_condition,
    if_else_node,
    if_then_body,
    leaf_value,
    program_body,
    program_funcs,
    program_name,
    return_expression,
    while_body,
    while_condition,
)


@dataclass
class Quadruple:
    """Representa una instrucción intermedia en forma de cuádruplo."""

    op: str
    arg1: int | None = None
    arg2: int | None = None
    res: int | None = None

    def __str__(self) -> str:
        a1 = "_" if self.arg1 is None else str(self.arg1)
        a2 = "_" if self.arg2 is None else str(self.arg2)
        result = "_" if self.res is None else str(self.res)
        return f"({self.op}, {a1}, {a2}, {result})"


@dataclass
class QuadrupleQueue:
    """Contenedor lineal de cuádruplos en orden de emisión."""

    items: list[Quadruple] = field(default_factory=list)

    def push(self, quad: Quadruple) -> int:
        self.items.append(quad)
        return len(self.items) - 1

    def dump(self) -> str:
        return "".join(f"[{index}] {quad}\n" for index, quad in enumerate(self.items))


@dataclass
class LocalMemoryContext:
    """Contadores de una sola función.

    Cada función reinicia su espacio local y temporal.
    """

    next_local_int: int = 3000
    next_local_float: int = 4000
    next_temp_int: int = 5000
    next_temp_float: int = 6000


@dataclass
class MemoryManager:
    """Asigna direcciones virtuales siguiendo el mismo modelo del compilador.

    Segmentos:
    - Variables globales enteras: 1000+
    - Variables globales flotantes: 2000+
    - Variables locales enteras: 3000+
    - Variables locales flotantes: 4000+
    - Temporales enteros: 5000+
    - Temporales flotantes: 6000+
    - Constantes enteras: 8000+
    - Constantes flotantes: 9000+
    - Constantes string: 10000+
    """

    const_int: dict[int, int] = field(default_factory=dict)
    const_float: dict[float, int] = field(default_factory=dict)
    const_str: dict[str, int] = field(default_factory=dict)
    next_global_int: int = 1000
    next_global_float: int = 2000
    next_const_int: int = 8000
    next_const_float: int = 9000
    next_const_str: int = 10000
    local_ctx: dict[str, LocalMemoryContext] = field(default_factory=dict)

    def alloc_global(self, tipo: TipoDato) -> int:
        # Reserva memoria para variables declaradas en el ámbito global.
        if tipo in (TipoDato.ENTERO, TipoDato.NULA):
            value = self.next_global_int
            self.next_global_int += 1
            return value
        value = self.next_global_float
        self.next_global_float += 1
        return value

    def crear_contexto_funcion(self, ambito: str) -> None:
        # Crea el contexto local/temporal de una función.
        self.local_ctx[ambito] = LocalMemoryContext()

    def _ctx(self, ambito: str) -> LocalMemoryContext:
        if ambito not in self.local_ctx:
            self.crear_contexto_funcion(ambito)
        return self.local_ctx[ambito]

    def alloc_local(self, ambito: str, tipo: TipoDato) -> int:
        # Reserva memoria para parámetros y variables locales.
        ctx = self._ctx(ambito)
        if tipo in (TipoDato.ENTERO, TipoDato.NULA):
            value = ctx.next_local_int
            ctx.next_local_int += 1
            return value
        value = ctx.next_local_float
        ctx.next_local_float += 1
        return value

    def alloc_temp(self, ambito: str, tipo: TipoDato) -> int:
        # Reserva temporales para resultados intermedios de expresiones.
        ctx = self._ctx(ambito)
        if tipo == TipoDato.FLOTANTE:
            value = ctx.next_temp_float
            ctx.next_temp_float += 1
            return value
        value = ctx.next_temp_int
        ctx.next_temp_int += 1
        return value

    def alloc_const_int(self, value: int) -> int:
        # Reutiliza o crea la dirección virtual de una constante entera.
        if value not in self.const_int:
            self.const_int[value] = self.next_const_int
            self.next_const_int += 1
        return self.const_int[value]

    def alloc_const_float(self, value: float) -> int:
        # Reutiliza o crea la dirección virtual de una constante flotante.
        if value not in self.const_float:
            self.const_float[value] = self.next_const_float
            self.next_const_float += 1
        return self.const_float[value]

    def alloc_const_str(self, value: str) -> int:
        # Reutiliza o crea la dirección virtual de strings y nombres de función.
        if value not in self.const_str:
            self.const_str[value] = self.next_const_str
            self.next_const_str += 1
        return self.const_str[value]


class QuadrupleGenerator:
    """Generador de cuádruplos a partir del CST ya validado."""

    def __init__(self, tree: SyntaxNode, directorio: DirectorioFunciones, cubo: CuboSemantico):
        self.tree = tree
        self.directorio = directorio
        self.cubo = cubo
        self.memory = MemoryManager()
        self.quadruples = QuadrupleQueue()
        self.operand_stack: list[int] = []
        self.type_stack: list[TipoDato] = []
        self.operator_stack: list[Operador] = []
        self.jump_stack: list[int] = []
        self.program_name = program_name(tree)

    def generate(self) -> None:
        self._preassign_memory()

        # PN-NL01: emitir GOTO inicial pendiente.
        goto_main = self.quadruples.push(Quadruple("GOTO"))

        for func_node in self._iter_funcs(program_funcs(self.tree)):
            nombre = func_name(func_node)
            # PN-NL02: guardar dir_inicio de la función.
            self.directorio.asignar_dir_inicio_funcion(nombre, len(self.quadruples.items))
            self._generate_cuerpo(func_body_node(func_node), nombre)
            # PN-NL05: cierre natural de función.
            self.quadruples.push(Quadruple("ENDFUNC"))

        # PN-NL03: rellenar salto al cuerpo principal.
        self.quadruples.items[goto_main].res = len(self.quadruples.items)
        self._generate_cuerpo(program_body(self.tree), self.program_name)

        # PN-NL04: emitir END.
        self.quadruples.push(Quadruple("END"))

    def _preassign_memory(self) -> None:
        """Reserva direcciones virtuales para variables globales y locales."""

        global_table = self.directorio.funciones[self.program_name].tabla_vars.variables
        for nombre, entrada in global_table.items():
            self.directorio.asignar_dir_variable(
                self.program_name, nombre, self.memory.alloc_global(entrada.tipo)
            )

        for func_name, entrada in self.directorio.funciones.items():
            if func_name == self.program_name:
                continue
            self.memory.crear_contexto_funcion(func_name)
            for nombre, var in entrada.tabla_vars.variables.items():
                self.directorio.asignar_dir_variable(
                    func_name, nombre, self.memory.alloc_local(func_name, var.tipo)
                )

    def _generate_cuerpo(self, cuerpo_node: SyntaxNode, ambito: str) -> None:
        for stmt in body_statements(cuerpo_node):
            self._generate_stmt(stmt, ambito)

    def _generate_stmt(self, stmt: SyntaxNode, ambito: str) -> None:
        if stmt.symbol == "Asigna":
            self._generate_assignment(stmt, ambito)
        elif stmt.symbol == "Imprime":
            self._generate_print(stmt, ambito)
        elif stmt.symbol == "LlamadaStmt":
            self._generate_call(stmt.children[0], ambito, as_expression=False)
        elif stmt.symbol == "Condicion":
            self._generate_if(stmt, ambito)
        elif stmt.symbol == "Ciclo":
            self._generate_while(stmt, ambito)
        elif stmt.symbol == "Regresa":
            self._generate_return(stmt, ambito)
        elif stmt.symbol == "Bloque":
            for nested in block_statements(stmt):
                self._generate_stmt(nested, ambito)

    def _generate_assignment(self, stmt: SyntaxNode, ambito: str) -> None:
        # PN-L08: generar cuadruplo de asignación.
        self._process_expression(assign_expression(stmt), ambito)
        expr_dir, expr_type = self._pop_operand()
        destino = assign_target_name(stmt)
        tipo_destino = self.directorio.resolver_variable(ambito, destino)
        self.cubo.consultar(tipo_destino, expr_type, Operador.ASIGNA)
        dir_destino = self.directorio.resolver_dir_variable(ambito, destino)
        self.quadruples.push(Quadruple("=", expr_dir, None, dir_destino))

    def _generate_print(self, stmt: SyntaxNode, ambito: str) -> None:
        for child in stmt.children[2:-2]:
            if isinstance(child, SyntaxNode) and child.symbol == "ImprimeExpr":
                # PN-L09: imprimir expresión.
                self._process_expression(child.children[0], ambito)
                expr_dir, _ = self._pop_operand()
                self.quadruples.push(Quadruple("PRINT", expr_dir, None, None))
            elif isinstance(child, SyntaxNode) and child.symbol == "ImprimeStr":
                # PN-L10: imprimir string.
                self.quadruples.push(
                    Quadruple("PRINTS", self.memory.alloc_const_str(leaf_value(child.children[0])), None, None)
                )

    def _generate_if(self, stmt: SyntaxNode, ambito: str) -> None:
        # PN-NL06: evaluar condición de `si`.
        self._process_expression(if_condition(stmt), ambito)
        dir_cond, tipo_cond = self._pop_operand()
        if tipo_cond != TipoDato.ENTERO:
            raise SemanticError("CondicionNoBooleana", contexto="si", tipo=str(tipo_cond))

        # PN-NL07: GOTOF pendiente.
        gotof = self.quadruples.push(Quadruple("GOTOF", dir_cond, None, None))
        self.jump_stack.append(gotof)

        self._generate_cuerpo(if_then_body(stmt), ambito)
        maybe_else_body = else_body(if_else_node(stmt))
        if maybe_else_body is not None:
            # PN-NL09: si con sino.
            goto_end = self.quadruples.push(Quadruple("GOTO", None, None, None))
            self.quadruples.items[self.jump_stack.pop()].res = len(self.quadruples.items)
            self.jump_stack.append(goto_end)
            self._generate_cuerpo(maybe_else_body, ambito)
            # PN-NL10: cerrar si/sino.
            self.quadruples.items[self.jump_stack.pop()].res = len(self.quadruples.items)
        else:
            # PN-NL08: cerrar si sin sino.
            self.quadruples.items[self.jump_stack.pop()].res = len(self.quadruples.items)

    def _generate_while(self, stmt: SyntaxNode, ambito: str) -> None:
        # PN-NL11: guardar retorno del ciclo.
        loop_start = len(self.quadruples.items)
        self.jump_stack.append(loop_start)

        # PN-NL12: evaluar condición.
        self._process_expression(while_condition(stmt), ambito)
        dir_cond, tipo_cond = self._pop_operand()
        if tipo_cond != TipoDato.ENTERO:
            raise SemanticError("CondicionNoBooleana", contexto="mientras", tipo=str(tipo_cond))

        # PN-NL13: GOTOF pendiente.
        gotof = self.quadruples.push(Quadruple("GOTOF", dir_cond, None, None))
        self.jump_stack.append(gotof)

        self._generate_cuerpo(while_body(stmt), ambito)

        # PN-NL14: cerrar ciclo y rellenar GOTOF.
        gotof_idx = self.jump_stack.pop()
        return_idx = self.jump_stack.pop()
        self.quadruples.push(Quadruple("GOTO", None, None, return_idx))
        self.quadruples.items[gotof_idx].res = len(self.quadruples.items)

    def _generate_return(self, stmt: SyntaxNode, ambito: str) -> None:
        # PN-NL22 / PN-NL23: validar retorno, copiar a la global homónima
        # y marcar explícitamente el retorno de control con RETURN.
        if ambito == self.program_name:
            raise SemanticError("RegresaFueraDeFuncion")
        entrada = self.directorio.buscar_funcion(ambito)
        if entrada.tipo_retorno == TipoDato.NULA:
            raise SemanticError("RegresaEnFuncionNula", funcion=ambito)
        self._process_expression(return_expression(stmt), ambito)
        dir_expr, tipo_expr = self._pop_operand()
        self.cubo.consultar(entrada.tipo_retorno, tipo_expr, Operador.ASIGNA)
        self.quadruples.push(Quadruple("RETURN", None, None, dir_expr))

    def _generate_call(self, llamada_node: SyntaxNode, ambito: str, as_expression: bool) -> TipoDato:
        nombre = call_name(llamada_node)
        entrada = self.directorio.buscar_funcion(nombre)
        args = self._extract_args(call_args_node(llamada_node))

        # PN-NL15 / PN-S11: validar existencia y aridad.
        if len(args) != entrada.num_params:
            raise SemanticError(
                "ArityMismatch", funcion=nombre, esperados=entrada.num_params, recibidos=len(args)
            )

        # PN-NL16: ERA con nombre de función como constante string.
        dir_nombre = self.memory.alloc_const_str(nombre)
        self.quadruples.push(Quadruple("ERA", dir_nombre, None, None))

        for index, (arg_node, tipo_param) in enumerate(zip(args, entrada.tipos_params)):
            # PN-NL17: validar argumento.
            self._process_expression(arg_node, ambito)
            dir_arg, tipo_arg = self._pop_operand()
            self.cubo.consultar(tipo_param, tipo_arg, Operador.ASIGNA)

            # PN-NL18: emitir PARAM.
            dir_param = entrada.tabla_vars.buscar(entrada.nombres_params[index]).dir_virtual
            self.quadruples.push(Quadruple("PARAM", dir_arg, None, dir_param))

        # PN-NL19: validar dir_inicio.
        if entrada.dir_inicio == 0:
            raise SemanticError("FuncionSinDirInicio", funcion=nombre)

        # PN-NL20: transferir control.
        self.quadruples.push(Quadruple("GOSUB", dir_nombre, None, entrada.dir_inicio))

        # PN-NL21: recuperar retorno a temporal.
        if entrada.tipo_retorno != TipoDato.NULA:
            dir_global = self.directorio.resolver_dir_variable(self.program_name, nombre)
            dir_temp = self.memory.alloc_temp(ambito, entrada.tipo_retorno)
            self.quadruples.push(Quadruple("=", dir_global, None, dir_temp))
            if as_expression:
                self.operand_stack.append(dir_temp)
                self.type_stack.append(entrada.tipo_retorno)
        return entrada.tipo_retorno

    def _process_expression(self, expr_node: SyntaxNode, ambito: str) -> None:
        # children[0]: parte aritmética izquierda; children[1]: operador relacional opcional.
        self._process_exp(expr_node.children[0], ambito)
        exp_op = expr_node.children[1]
        if exp_op.children:
            # PN-L06: operador relacional.
            # children[0]: operador; children[1]: expresión aritmética derecha.
            op = {
                ">": Operador.MAYOR,
                "<": Operador.MENOR,
                "==": Operador.IGUAL,
                "!=": Operador.DIFERENTE,
            }[leaf_value(exp_op.children[0])]
            self.operator_stack.append(op)
            self._process_exp(exp_op.children[1], ambito)
            self._reduce_expression(ambito)

    def _process_exp(self, exp_node: SyntaxNode, ambito: str) -> None:
        # children[0]: primer término; children[1]: lista de repeticiones (+/- término).
        self._process_term(exp_node.children[0], ambito)
        for item in exp_node.children[1].children:
            # PN-L04: + o -.
            # children[0]: operador; children[1]: término siguiente.
            op = Operador.SUMA if leaf_value(item.children[0]) == "+" else Operador.RESTA
            self.operator_stack.append(op)
            self._process_term(item.children[1], ambito)
            self._reduce_expression(ambito)

    def _process_term(self, term_node: SyntaxNode, ambito: str) -> None:
        # children[0]: primer factor; children[1]: lista de repeticiones (*// factor).
        self._process_factor(term_node.children[0], ambito)
        for item in term_node.children[1].children:
            # PN-L05: * o /.
            # children[0]: operador; children[1]: factor siguiente.
            op = Operador.MUL if leaf_value(item.children[0]) == "*" else Operador.DIV
            self.operator_stack.append(op)
            self._process_factor(item.children[1], ambito)
            self._reduce_expression(ambito)

    def _process_factor(self, factor_node: SyntaxNode, ambito: str) -> None:
        if factor_node.symbol == "FactorParen":
            # children[1]: expresión dentro de los paréntesis.
            self._process_expression(factor_node.children[1], ambito)
        elif factor_node.symbol == "FactorPosId":
            # PN-L02: identificador.
            # children[1]: nombre del identificador.
            self._push_variable(leaf_value(factor_node.children[1]), ambito)
        elif factor_node.symbol == "FactorNegId":
            # PN-L03: negación unaria como 0 - id.
            # children[1]: nombre del identificador negado.
            nombre = leaf_value(factor_node.children[1])
            tipo = self.directorio.resolver_variable(ambito, nombre)
            dir_id = self.directorio.resolver_dir_variable(ambito, nombre)
            dir_zero = self.memory.alloc_const_float(0.0) if tipo == TipoDato.FLOTANTE else self.memory.alloc_const_int(0)
            self.operand_stack.append(dir_zero)
            self.type_stack.append(tipo)
            self.operand_stack.append(dir_id)
            self.type_stack.append(tipo)
            self.operator_stack.append(Operador.RESTA)
            self._reduce_expression(ambito)
        elif factor_node.symbol == "FactorPosCte":
            # children[1]: nodo constante envuelto.
            cte = factor_node.children[1]
            if cte.symbol == "CteEnt":
                # children[0]: literal entero.
                value = leaf_value(cte.children[0])
                self.operand_stack.append(self.memory.alloc_const_int(value))
                self.type_stack.append(TipoDato.ENTERO)
            else:
                # children[0]: literal flotante.
                value = leaf_value(cte.children[0])
                self.operand_stack.append(self.memory.alloc_const_float(value))
                self.type_stack.append(TipoDato.FLOTANTE)
        elif factor_node.symbol == "FactorNegCte":
            # children[1]: nodo constante envuelto.
            cte = factor_node.children[1]
            if cte.symbol == "CteEnt":
                # children[0]: literal entero.
                value = leaf_value(cte.children[0])
                dir_zero = self.memory.alloc_const_int(0)
                dir_cte = self.memory.alloc_const_int(value)
                tipo = TipoDato.ENTERO
            else:
                # children[0]: literal flotante.
                value = leaf_value(cte.children[0])
                dir_zero = self.memory.alloc_const_float(0.0)
                dir_cte = self.memory.alloc_const_float(value)
                tipo = TipoDato.FLOTANTE
            self.operand_stack.append(dir_zero)
            self.type_stack.append(tipo)
            self.operand_stack.append(dir_cte)
            self.type_stack.append(tipo)
            self.operator_stack.append(Operador.RESTA)
            self._reduce_expression(ambito)
        elif factor_node.symbol == "FactorId":
            # PN-L02: identificador.
            # children[0]: nombre del identificador.
            self._push_variable(leaf_value(factor_node.children[0]), ambito)
        elif factor_node.symbol == "FactorLlamada":
            # children[0]: nodo de llamada a función.
            self._generate_call(factor_node.children[0], ambito, as_expression=True)
        elif factor_node.children[0].symbol == "CteEnt":
            # PN-L01: constante entera.
            # children[0]: nodo CteEnt; su children[0] es la hoja con el literal.
            value = leaf_value(factor_node.children[0].children[0])
            self.operand_stack.append(self.memory.alloc_const_int(value))
            self.type_stack.append(TipoDato.ENTERO)
        else:
            # PN-L01: constante flotante.
            # children[0]: nodo CteFlt; su children[0] es la hoja con el literal.
            value = leaf_value(factor_node.children[0].children[0])
            self.operand_stack.append(self.memory.alloc_const_float(value))
            self.type_stack.append(TipoDato.FLOTANTE)

    def _push_variable(self, nombre: str, ambito: str) -> None:
        dir_var = self.directorio.resolver_dir_variable(ambito, nombre)
        tipo = self.directorio.resolver_variable(ambito, nombre)
        self.operand_stack.append(dir_var)
        self.type_stack.append(tipo)

    def _reduce_expression(self, ambito: str) -> None:
        # PN-L07: reducción de expresión.
        op = self.operator_stack.pop()
        dir_der, tipo_der = self._pop_operand()
        dir_izq, tipo_izq = self._pop_operand()
        tipo_res = self.cubo.consultar(tipo_izq, tipo_der, op)
        dir_temp = self.memory.alloc_temp(ambito, tipo_res)
        quad_op = {
            Operador.SUMA: "+",
            Operador.RESTA: "-",
            Operador.MUL: "*",
            Operador.DIV: "/",
            Operador.MAYOR: ">",
            Operador.MENOR: "<",
            Operador.IGUAL: "==",
            Operador.DIFERENTE: "!=",
            Operador.ASIGNA: "=",
        }[op]
        self.quadruples.push(Quadruple(quad_op, dir_izq, dir_der, dir_temp))
        self.operand_stack.append(dir_temp)
        self.type_stack.append(tipo_res)

    def _pop_operand(self) -> tuple[int, TipoDato]:
        return self.operand_stack.pop(), self.type_stack.pop()

    def _iter_funcs(self, funcs_node: SyntaxNode) -> list[SyntaxNode]:
        return [child for child in funcs_node.children if isinstance(child, SyntaxNode)]

    def _extract_args(self, args_node: SyntaxNode) -> list[SyntaxNode]:
        args: list[SyntaxNode] = []
        for child in args_node.children:
            if isinstance(child, SyntaxNode) and child.symbol == "Expresion":
                args.append(child)
            elif isinstance(child, SyntaxNode) and child.symbol == "ArgsList":
                args.extend(self._extract_args(child))
        return args
