"""Lexer del lenguaje usando SLY."""

from sly import Lexer


class LexicalError(Exception):
    """Error léxico con el mismo formato básico del compilador."""

    def __init__(self, start: int, end: int, slice_: str):
        self.span = (start, end)
        self.slice = slice_
        super().__init__(str(self))

    def __str__(self) -> str:
        return f"Token no reconocido: {self.slice!r} en bytes {self.span[0]}..{self.span[1]}"


class PatitoLexer(Lexer):
    """Tokeniza el código fuente y conserva offsets para depuración."""

    tokens = {
        PROGRAMA,
        INICIO,
        FIN,
        NULA,
        ESCRIBE,
        MIENTRAS,
        HAZ,
        SI,
        SINO,
        VARS,
        ENTERO,
        FLOTANTE,
        REGRESA,
        ID,
        CTE_FLOT,
        CTE_ENT,
        LETRERO,
        EQEQ,
        NEQ,
        PLUS,
        MINUS,
        STAR,
        SLASH,
        LT,
        GT,
        EQ,
        SEMI,
        COMMA,
        COLON,
        LPAREN,
        RPAREN,
        LBRACE,
        RBRACE,
        LBRACKET,
        RBRACKET,
    }

    ignore = " \t\r\n"

    ID = r"[a-zA-Z_][a-zA-Z_0-9]*"
    ID["programa"] = PROGRAMA
    ID["inicio"] = INICIO
    ID["fin"] = FIN
    ID["nula"] = NULA
    ID["escribe"] = ESCRIBE
    ID["mientras"] = MIENTRAS
    ID["haz"] = HAZ
    ID["si"] = SI
    ID["sino"] = SINO
    ID["vars"] = VARS
    ID["entero"] = ENTERO
    ID["flotante"] = FLOTANTE
    ID["regresa"] = REGRESA

    CTE_FLOT = r"[0-9]+\.[0-9]+"
    CTE_ENT = r"[0-9]+"
    LETRERO = r'"[^"]*"'

    EQEQ = r"=="
    NEQ = r"!="
    PLUS = r"\+"
    MINUS = r"-"
    STAR = r"\*"
    SLASH = r"/"
    LT = r"<"
    GT = r">"
    EQ = r"="
    SEMI = r";"
    COMMA = r","
    COLON = r":"
    LPAREN = r"\("
    RPAREN = r"\)"
    LBRACE = r"\{"
    RBRACE = r"\}"
    LBRACKET = r"\["
    RBRACKET = r"\]"

    def CTE_FLOT(self, token):
        token.value = float(token.value)
        return token

    def CTE_ENT(self, token):
        token.value = int(token.value)
        return token

    def LETRERO(self, token):
        token.value = token.value[1:-1]
        return token

    def error(self, token):
        raise LexicalError(token.index, token.index + 1, token.value[0])


def iter_tokens(src: str):
    """Produce tokens con rangos para el modo `--lex`."""

    lexer = PatitoLexer()
    for token in lexer.tokenize(src):
        start = token.index
        end = start + len(str(token.value if token.type not in {"CTE_ENT", "CTE_FLOT"} else token.value))
        if token.type in {"PROGRAMA", "INICIO", "FIN", "NULA", "ESCRIBE", "MIENTRAS", "HAZ", "SI", "SINO", "VARS", "ENTERO", "FLOTANTE", "REGRESA"}:
            end = start + len(token.value)
        elif token.type == "LETRERO":
            end = start + len(token.value) + 2
        yield start, token, end
