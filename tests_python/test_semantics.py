"""Pruebas end-to-end del análisis semántico."""

import unittest

from compilador.compiler import analyze_source


class SemanticAnalysisTests(unittest.TestCase):
    def test_sem_01_valid_program(self):
        result = analyze_source("programa t; vars x, y : entero; inicio { x = 1; y = x + 2; } fin")
        self.assertFalse(result.has_errors(), result.report())

    def test_sem_02_undeclared_variable(self):
        result = analyze_source("programa t; inicio { x = 1; } fin")
        self.assertTrue(result.has_errors())
        self.assertIn("no declarada", result.report())

    def test_sem_03_duplicate_variable(self):
        result = analyze_source("programa t; vars x:entero; vars x:flotante; inicio { } fin")
        self.assertTrue(result.has_errors())
        self.assertIn("doblemente declarada", result.report())

    def test_sem_04_invalid_float_to_int_assignment(self):
        result = analyze_source("programa t; vars x:entero; vars y:flotante; inicio { x = y; } fin")
        self.assertTrue(result.has_errors())

    def test_sem_05_valid_int_to_float_assignment(self):
        result = analyze_source("programa t; vars x:flotante; vars y:entero; inicio { x = y; } fin")
        self.assertFalse(result.has_errors(), result.report())

    def test_sem_05b_negative_integer_constant_assignment(self):
        result = analyze_source("programa t; vars x:entero; inicio { x = -1; } fin")
        self.assertFalse(result.has_errors(), result.report())

    def test_sem_06_call_to_missing_function(self):
        result = analyze_source("programa t; inicio { foo(); } fin")
        self.assertTrue(result.has_errors())
        self.assertIn("Función no declarada", result.report())

    def test_sem_07_wrong_arity(self):
        src = 'programa t; nula f(a:entero) { { escribe(a); } }; inicio { f(1, 2); } fin'
        result = analyze_source(src)
        self.assertTrue(result.has_errors())
        self.assertIn("args", result.report())

    def test_sem_08_duplicate_function(self):
        src = 'programa t; nula f() { { escribe("a"); } }; nula f() { { escribe("b"); } }; inicio { } fin'
        result = analyze_source(src)
        self.assertTrue(result.has_errors())
        self.assertIn("doblemente declarada", result.report())

    def test_sem_09_valid_function_call(self):
        src = "programa t; vars r : entero; entero doble(n:entero) { { regresa n + n; } }; inicio { r = doble(5); } fin"
        result = analyze_source(src)
        self.assertFalse(result.has_errors(), result.report())

    def test_sem_10_local_variable_out_of_scope(self):
        src = "programa t; nula f() { vars local:entero; { local = 1; } }; inicio { local = 2; } fin"
        result = analyze_source(src)
        self.assertTrue(result.has_errors())
        self.assertIn("no declarada", result.report())

    def test_sem_11_valid_return_type(self):
        src = "programa t; entero id(a:entero) { { regresa a; } }; inicio { } fin"
        result = analyze_source(src)
        self.assertFalse(result.has_errors(), result.report())

    def test_sem_12_invalid_return_type(self):
        src = "programa t; entero id(a:flotante) { { regresa a; } }; inicio { } fin"
        result = analyze_source(src)
        self.assertTrue(result.has_errors())
        self.assertIn("Retorno incompatible", result.report())

    def test_sem_13_return_in_void_function(self):
        src = "programa t; nula f() { { regresa 1; } }; inicio { } fin"
        result = analyze_source(src)
        self.assertTrue(result.has_errors())
        self.assertIn("no debe tener 'regresa'", result.report())

    def test_sem_14_missing_return(self):
        src = 'programa t; entero f() { { escribe("hola"); } }; inicio { } fin'
        result = analyze_source(src)
        self.assertTrue(result.has_errors())
        self.assertIn("debe tener al menos un 'regresa'", result.report())


if __name__ == "__main__":
    unittest.main()
