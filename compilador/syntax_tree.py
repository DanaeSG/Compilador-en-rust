"""Árbol sintáctico concreto (CST).
"""

from __future__ import annotations

from dataclasses import dataclass, field


@dataclass
class SyntaxLeaf:
    """Hoja terminal del árbol sintáctico."""

    symbol: str
    value: object

    def pretty(self, indent: int = 0) -> str:
        return f"{'  ' * indent}{self.symbol}: {self.value!r}"
    
    def __str__(self) -> str:
        return self.pretty()


@dataclass
class SyntaxNode:
    """Nodo no terminal del árbol sintáctico concreto."""

    symbol: str
    children: list[SyntaxNode | SyntaxLeaf] = field(default_factory=list)

    def pretty(self, indent: int = 0) -> str:
        lines = [f"{'  ' * indent}{self.symbol}"]
        for child in self.children:
            lines.append(child.pretty(indent + 1))
        return "\n".join(lines)

    def __str__(self) -> str:
        return self.pretty()

def leaf(symbol: str, value: object) -> SyntaxLeaf:
    """Crea una hoja terminal con nombre legible."""

    return SyntaxLeaf(symbol=symbol, value=value)


def node(symbol: str, *children: SyntaxNode | SyntaxLeaf) -> SyntaxNode:
    """Crea un nodo no terminal con sus hijos."""

    return SyntaxNode(symbol=symbol, children=list(children))


def leaf_value(child: SyntaxLeaf | SyntaxNode):
    """Regresa el valor de una hoja terminal."""

    if isinstance(child, SyntaxLeaf):
        return child.value
    raise TypeError("Se esperaba una hoja terminal")


def program_name(tree: SyntaxNode) -> str:
    return leaf_value(tree.children[1])


def program_vars(tree: SyntaxNode) -> SyntaxNode:
    return tree.children[3]


def program_funcs(tree: SyntaxNode) -> SyntaxNode:
    return tree.children[4]


def program_body(tree: SyntaxNode) -> SyntaxNode:
    return tree.children[6]


def func_name(func_node: SyntaxNode) -> str:
    return leaf_value(func_node.children[1])


def func_return_type_node(func_node: SyntaxNode) -> SyntaxNode:
    return func_node.children[0]


def func_params_node(func_node: SyntaxNode) -> SyntaxNode:
    return func_node.children[3]


def func_vars_node(func_node: SyntaxNode) -> SyntaxNode:
    return func_node.children[6]


def func_body_node(func_node: SyntaxNode) -> SyntaxNode:
    return func_node.children[7]


def body_statements(cuerpo_node: SyntaxNode) -> list[SyntaxNode]:
    return [child for child in cuerpo_node.children[1].children if isinstance(child, SyntaxNode)]


def block_statements(stmt: SyntaxNode) -> list[SyntaxNode]:
    return [child for child in stmt.children[1].children if isinstance(child, SyntaxNode)]


def assign_target_name(stmt: SyntaxNode) -> str:
    return leaf_value(stmt.children[0])


def assign_expression(stmt: SyntaxNode) -> SyntaxNode:
    return stmt.children[2]


def if_condition(stmt: SyntaxNode) -> SyntaxNode:
    return stmt.children[2]


def if_then_body(stmt: SyntaxNode) -> SyntaxNode:
    return stmt.children[4]


def if_else_node(stmt: SyntaxNode) -> SyntaxNode:
    return stmt.children[5]


def else_body(sino_node: SyntaxNode) -> SyntaxNode | None:
    return sino_node.children[1] if sino_node.children else None


def while_condition(stmt: SyntaxNode) -> SyntaxNode:
    return stmt.children[2]


def while_body(stmt: SyntaxNode) -> SyntaxNode:
    return stmt.children[5]


def return_expression(stmt: SyntaxNode) -> SyntaxNode:
    return stmt.children[1]


def call_name(llamada_node: SyntaxNode) -> str:
    return leaf_value(llamada_node.children[0])


def call_args_node(llamada_node: SyntaxNode) -> SyntaxNode:
    return llamada_node.children[2]
