"""Pruebas de errores sintácticos."""

import unittest

from compilador.compiler import build_syntax_tree


class ErrorTests(unittest.TestCase):
    def test_e_01_missing_fin(self):
        with self.assertRaises(SyntaxError):
            build_syntax_tree("programa p; inicio { x = 1; }")

    def test_e_02_missing_semicolon(self):
        with self.assertRaises(SyntaxError):
            build_syntax_tree("programa p; inicio { x = 1 } fin")

    def test_e_03_invalid_parentheses(self):
        with self.assertRaises(SyntaxError):
            build_syntax_tree("programa p; inicio { x = (1 + 2; } fin")

    def test_e_04_invalid_type(self):
        with self.assertRaises(SyntaxError):
            build_syntax_tree("programa p; vars x : real; inicio { } fin")

    def test_e_05_else_without_if(self):
        with self.assertRaises(SyntaxError):
            build_syntax_tree("programa p; inicio { sino { x = 1; } } fin")

    def test_e_06_invalid_call(self):
        with self.assertRaises(SyntaxError):
            build_syntax_tree("programa p; inicio { f(,1); } fin")

    def test_e_07_empty_program(self):
        with self.assertRaises(SyntaxError):
            build_syntax_tree("")


if __name__ == "__main__":
    unittest.main()
