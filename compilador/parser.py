"""Parser SLY puro.

Reconoce la gramática y construye un árbol sintáctico concreto (CST). 
"""

from sly import Parser

from .lexer import PatitoLexer
from .syntax_tree import SyntaxNode, leaf, node


class PatitoParser(Parser):
    """Parser del lenguaje que construye un CST."""

    tokens = PatitoLexer.tokens
    start = "programa"

    @_('PROGRAMA ID SEMI vars funcs INICIO cuerpo FIN')
    def programa(self, p):
        return node(
            "Programa",
            leaf("programa", p.PROGRAMA),
            leaf("id", p.ID),
            leaf(";", p.SEMI),
            p.vars,
            p.funcs,
            leaf("inicio", p.INICIO),
            p.cuerpo,
            leaf("fin", p.FIN),
        )

    @_('vars_decl vars')
    def vars(self, p):
        return node("Vars", p.vars_decl, *p.vars.children)

    @_('empty')
    def vars(self, _p):
        return node("Vars")

    @_('VARS id_list COLON tipo SEMI')
    def vars_decl(self, p):
        return node(
            "VarDecl",
            leaf("vars", p.VARS),
            p.id_list,
            leaf(":", p.COLON),
            p.tipo,
            leaf(";", p.SEMI),
        )

    @_('ID vars_list')
    def id_list(self, p):
        return node("IdList", leaf("id", p.ID), *p.vars_list.children)

    @_('COMMA ID vars_list')
    def vars_list(self, p):
        return node("VarsList", leaf(",", p.COMMA), leaf("id", p.ID), *p.vars_list.children)

    @_('empty')
    def vars_list(self, _p):
        return node("VarsList")

    @_('ENTERO')
    def tipo(self, p):
        return node("Tipo", leaf("entero", p.ENTERO))

    @_('FLOTANTE')
    def tipo(self, p):
        return node("Tipo", leaf("flotante", p.FLOTANTE))

    @_('func funcs')
    def funcs(self, p):
        return node("Funcs", p.func, *p.funcs.children)

    @_('empty')
    def funcs(self, _p):
        return node("Funcs")

    @_('tipo_func ID LPAREN params RPAREN LBRACE vars cuerpo RBRACE SEMI')
    def func(self, p):
        return node(
            "Func",
            p.tipo_func,
            leaf("id", p.ID),
            leaf("(", p.LPAREN),
            p.params,
            leaf(")", p.RPAREN),
            leaf("{", p.LBRACE),
            p.vars,
            p.cuerpo,
            leaf("}", p.RBRACE),
            leaf(";", p.SEMI),
        )

    @_('NULA')
    def tipo_func(self, p):
        return node("TipoFunc", leaf("nula", p.NULA))

    @_('tipo')
    def tipo_func(self, p):
        return node("TipoFunc", p.tipo)

    @_('ID COLON tipo params_list')
    def params(self, p):
        first = node("Param", leaf("id", p.ID), leaf(":", p.COLON), p.tipo)
        return node("Params", first, *p.params_list.children)

    @_('empty')
    def params(self, _p):
        return node("Params")

    @_('COMMA ID COLON tipo params_list')
    def params_list(self, p):
        current = node(
            "ParamListItem",
            leaf(",", p.COMMA),
            leaf("id", p.ID),
            leaf(":", p.COLON),
            p.tipo,
        )
        return node("ParamsList", current, *p.params_list.children)

    @_('empty')
    def params_list(self, _p):
        return node("ParamsList")

    @_('LBRACE estatuto_list RBRACE')
    def cuerpo(self, p):
        return node("Cuerpo", leaf("{", p.LBRACE), p.estatuto_list, leaf("}", p.RBRACE))

    @_('estatuto estatuto_list')
    def estatuto_list(self, p):
        return node("EstatutoList", p.estatuto, *p.estatuto_list.children)

    @_('empty')
    def estatuto_list(self, _p):
        return node("EstatutoList")

    @_('asigna')
    def estatuto(self, p):
        return p.asigna

    @_('condicion')
    def estatuto(self, p):
        return p.condicion

    @_('ciclo')
    def estatuto(self, p):
        return p.ciclo

    @_('llamada SEMI')
    def estatuto(self, p):
        return node("LlamadaStmt", p.llamada, leaf(";", p.SEMI))

    @_('regresa')
    def estatuto(self, p):
        return p.regresa

    @_('imprime')
    def estatuto(self, p):
        return p.imprime

    @_('LBRACKET estatuto_list RBRACKET')
    def estatuto(self, p):
        return node("Bloque", leaf("[", p.LBRACKET), p.estatuto_list, leaf("]", p.RBRACKET))

    @_('ID EQ expresion SEMI')
    def asigna(self, p):
        return node("Asigna", leaf("id", p.ID), leaf("=", p.EQ), p.expresion, leaf(";", p.SEMI))

    @_('SI LPAREN expresion RPAREN cuerpo sino_op SEMI')
    def condicion(self, p):
        return node(
            "Condicion",
            leaf("si", p.SI),
            leaf("(", p.LPAREN),
            p.expresion,
            leaf(")", p.RPAREN),
            p.cuerpo,
            p.sino_op,
            leaf(";", p.SEMI),
        )

    @_('SINO cuerpo')
    def sino_op(self, p):
        return node("SinoOp", leaf("sino", p.SINO), p.cuerpo)

    @_('empty')
    def sino_op(self, _p):
        return node("SinoOp")

    @_('MIENTRAS LPAREN expresion RPAREN HAZ cuerpo SEMI')
    def ciclo(self, p):
        return node(
            "Ciclo",
            leaf("mientras", p.MIENTRAS),
            leaf("(", p.LPAREN),
            p.expresion,
            leaf(")", p.RPAREN),
            leaf("haz", p.HAZ),
            p.cuerpo,
            leaf(";", p.SEMI),
        )

    @_('ESCRIBE LPAREN imprime_alt imprime_list RPAREN SEMI')
    def imprime(self, p):
        return node(
            "Imprime",
            leaf("escribe", p.ESCRIBE),
            leaf("(", p.LPAREN),
            p.imprime_alt,
            *p.imprime_list.children,
            leaf(")", p.RPAREN),
            leaf(";", p.SEMI),
        )

    @_('expresion')
    def imprime_alt(self, p):
        return node("ImprimeExpr", p.expresion)

    @_('LETRERO')
    def imprime_alt(self, p):
        return node("ImprimeStr", leaf("letrero", p.LETRERO))

    @_('COMMA imprime_alt imprime_list')
    def imprime_list(self, p):
        return node("ImprimeList", leaf(",", p.COMMA), p.imprime_alt, *p.imprime_list.children)

    @_('empty')
    def imprime_list(self, _p):
        return node("ImprimeList")

    @_('ID LPAREN args RPAREN')
    def llamada(self, p):
        return node(
            "Llamada",
            leaf("id", p.ID),
            leaf("(", p.LPAREN),
            p.args,
            leaf(")", p.RPAREN),
        )

    @_('expresion args_list')
    def args(self, p):
        return node("Args", p.expresion, *p.args_list.children)

    @_('empty')
    def args(self, _p):
        return node("Args")

    @_('COMMA expresion args_list')
    def args_list(self, p):
        return node("ArgsList", leaf(",", p.COMMA), p.expresion, *p.args_list.children)

    @_('empty')
    def args_list(self, _p):
        return node("ArgsList")

    @_('exp exp_op')
    def expresion(self, p):
        return node("Expresion", p.exp, p.exp_op)

    @_('GT exp')
    def exp_op(self, p):
        return node("ExpOp", leaf(">", p.GT), p.exp)

    @_('LT exp')
    def exp_op(self, p):
        return node("ExpOp", leaf("<", p.LT), p.exp)

    @_('NEQ exp')
    def exp_op(self, p):
        return node("ExpOp", leaf("!=", p.NEQ), p.exp)

    @_('EQEQ exp')
    def exp_op(self, p):
        return node("ExpOp", leaf("==", p.EQEQ), p.exp)

    @_('empty')
    def exp_op(self, _p):
        return node("ExpOp")

    @_('termino exp_cont')
    def exp(self, p):
        return node("Exp", p.termino, p.exp_cont)

    @_('PLUS termino exp_cont')
    def exp_cont(self, p):
        item = node("ExpContItem", leaf("+", p.PLUS), p.termino)
        return node("ExpCont", item, *p.exp_cont.children)

    @_('MINUS termino exp_cont')
    def exp_cont(self, p):
        item = node("ExpContItem", leaf("-", p.MINUS), p.termino)
        return node("ExpCont", item, *p.exp_cont.children)

    @_('empty')
    def exp_cont(self, _p):
        return node("ExpCont")

    @_('factor termino_cont')
    def termino(self, p):
        return node("Termino", p.factor, p.termino_cont)

    @_('STAR factor termino_cont')
    def termino_cont(self, p):
        item = node("TermContItem", leaf("*", p.STAR), p.factor)
        return node("TerminoCont", item, *p.termino_cont.children)

    @_('SLASH factor termino_cont')
    def termino_cont(self, p):
        item = node("TermContItem", leaf("/", p.SLASH), p.factor)
        return node("TerminoCont", item, *p.termino_cont.children)

    @_('empty')
    def termino_cont(self, _p):
        return node("TerminoCont")

    @_('LPAREN expresion RPAREN')
    def factor(self, p):
        return node("FactorParen", leaf("(", p.LPAREN), p.expresion, leaf(")", p.RPAREN))

    @_('PLUS ID')
    def factor(self, p):
        return node("FactorPosId", leaf("+", p.PLUS), leaf("id", p.ID))

    @_('MINUS ID')
    def factor(self, p):
        return node("FactorNegId", leaf("-", p.MINUS), leaf("id", p.ID))

    @_('PLUS cte')
    def factor(self, p):
        return node("FactorPosCte", leaf("+", p.PLUS), p.cte)

    @_('MINUS cte')
    def factor(self, p):
        return node("FactorNegCte", leaf("-", p.MINUS), p.cte)

    @_('llamada')
    def factor(self, p):
        return node("FactorLlamada", p.llamada)

    @_('ID')
    def factor(self, p):
        return node("FactorId", leaf("id", p.ID))

    @_('cte')
    def factor(self, p):
        return node("FactorCte", p.cte)

    @_('CTE_ENT')
    def cte(self, p):
        return node("CteEnt", leaf("cte_ent", p.CTE_ENT))

    @_('CTE_FLOT')
    def cte(self, p):
        return node("CteFlot", leaf("cte_flot", p.CTE_FLOT))

    @_('REGRESA expresion SEMI')
    def regresa(self, p):
        return node("Regresa", leaf("regresa", p.REGRESA), p.expresion, leaf(";", p.SEMI))

    @_('')
    def empty(self, _p):
        return node("Empty")

    def error(self, token):
        if token is None:
            raise SyntaxError("Error de sintaxis: fin de archivo inesperado")
        raise SyntaxError(f"Error de sintaxis cerca de '{token.value}'")


def parse_source(src: str) -> SyntaxNode:
    """Parsea el código fuente y regresa el CST completo."""

    parser = PatitoParser()
    lexer = PatitoLexer()
    return parser.parse(lexer.tokenize(src))
