"""Pruebas del parser y del CST."""

import unittest

from compilador.syntax_tree import program_funcs, program_vars

from tests_python.test_support import parse_ok


class ParserTests(unittest.TestCase):
    def test_p_01_minimal_program(self):
        tree = parse_ok("programa p; inicio { } fin")
        self.assertEqual(tree.symbol, "Programa")

    def test_p_02_var_declarations(self):
        tree = parse_ok("programa v; vars a, b : entero; inicio { } fin")
        self.assertEqual(len(program_vars(tree).children), 1)

    def test_p_03_assignment(self):
        parse_ok("programa p; inicio { x = 10; } fin")

    def test_p_04_arithmetic_expression(self):
        parse_ok("programa p; inicio { x = 1 + 2 * 3; } fin")

    def test_p_05_relational_expression(self):
        parse_ok("programa rel; inicio { r = x > y; } fin")

    def test_p_06_if_without_else(self):
        tree = parse_ok("programa p; inicio { si (x > 0) { x = 1; }; } fin")
        stmt = tree.children[6].children[1].children[0]
        self.assertEqual(stmt.symbol, "Condicion")
        self.assertEqual(stmt.children[5].symbol, "SinoOp")
        self.assertFalse(stmt.children[5].children)

    def test_p_07_if_with_else(self):
        tree = parse_ok("programa p; inicio { si (x > 0) { x = 1; } sino { x = 2; }; } fin")
        stmt = tree.children[6].children[1].children[0]
        self.assertEqual(stmt.symbol, "Condicion")
        self.assertTrue(stmt.children[5].children)

    def test_p_08_while_loop(self):
        parse_ok("programa p; inicio { mientras (x < 10) haz { x = x + 1; }; } fin")

    def test_p_09_escribe(self):
        parse_ok('programa imp; inicio { escribe("res: ", x); } fin')

    def test_p_10_void_function_definition(self):
        tree = parse_ok(
            "programa fn1; nula suma(a : entero, b : entero) { { escribe(a); } }; inicio { } fin"
        )
        funcs = program_funcs(tree)
        self.assertEqual(len(funcs.children), 1)
        params_node = funcs.children[0].children[3]
        self.assertEqual(len(params_node.children), 2)

    def test_p_11_function_call_in_expression(self):
        parse_ok(
            "programa fn2; entero doble(n : entero) { { r = n + n; } }; inicio { resultado = doble(5); } fin"
        )

    def test_p_12_parenthesized_expression(self):
        parse_ok("programa par; inicio { z = (a + b) * c; } fin")

    def test_p_13_unary_negation(self):
        parse_ok("programa neg; inicio { z = -x; } fin")

    def test_p_13b_negative_integer_constant(self):
        parse_ok("programa neg; inicio { z = -1; } fin")

    def test_p_14_block(self):
        parse_ok("programa blq; inicio { [ x = 1; y = 2; ] } fin")

    def test_p_15_function_call_statement(self):
        parse_ok('programa call; nula greet() { { escribe("hi"); } }; inicio { greet(); } fin')

    def test_p_16_return_statement(self):
        parse_ok("programa ret; entero id(a:entero) { { regresa a; } }; inicio { } fin")


if __name__ == "__main__":
    unittest.main()
