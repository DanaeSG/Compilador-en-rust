"""Pruebas del lexer."""

import unittest

from compilador.lexer import LexicalError, PatitoLexer

from tests_python.test_support import token_types


class LexerTests(unittest.TestCase):
    def test_l_01_reserved_words(self):
        src = "programa inicio fin vars entero flotante si sino mientras haz nula escribe regresa"
        tokens = token_types(src)
        for expected in [
            "PROGRAMA",
            "INICIO",
            "FIN",
            "VARS",
            "ENTERO",
            "FLOTANTE",
            "SI",
            "SINO",
            "MIENTRAS",
            "HAZ",
            "NULA",
            "ESCRIBE",
            "REGRESA",
        ]:
            self.assertIn(expected, tokens)

    def test_l_02_valid_identifiers(self):
        for identifier in ["miVar", "_x", "a1", "Z"]:
            self.assertEqual(token_types(identifier), ["ID"])

    def test_l_03_reserved_is_not_identifier(self):
        self.assertEqual(token_types("si"), ["SI"])

    def test_l_04_integer_constants(self):
        for value in ["0", "42", "999"]:
            self.assertEqual(token_types(value), ["CTE_ENT"])

    def test_l_05_float_constants(self):
        for value in ["3.14", "0.0", "100.001"]:
            self.assertEqual(token_types(value), ["CTE_FLOT"])

    def test_l_06_string_literal(self):
        self.assertEqual(token_types('"hola mundo 123"'), ["LETRERO"])

    def test_l_07_operators(self):
        src = "+ - * / == != < > ="
        self.assertEqual(
            token_types(src),
            ["PLUS", "MINUS", "STAR", "SLASH", "EQEQ", "NEQ", "LT", "GT", "EQ"],
        )

    def test_l_08_punctuation(self):
        src = "; , : ( ) { } [ ]"
        self.assertEqual(
            token_types(src),
            ["SEMI", "COMMA", "COLON", "LPAREN", "RPAREN", "LBRACE", "RBRACE", "LBRACKET", "RBRACKET"],
        )

    def test_l_09_invalid_character(self):
        lexer = PatitoLexer()
        with self.assertRaises(LexicalError):
            list(lexer.tokenize("@"))


if __name__ == "__main__":
    unittest.main()
