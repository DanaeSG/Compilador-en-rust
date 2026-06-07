"""Pruebas de estrés del parser."""

import unittest

from tests_python.test_support import parse_ok


class StressTests(unittest.TestCase):
    def test_s_01_deep_nesting(self):
        src = "programa p; inicio {"
        src += " si (x > 0) {" * 25
        src += " };" * 25
        src += " } fin"
        parse_ok(src)

    def test_s_02_large_var_list(self):
        vars_decl = ", ".join(["a"] + [f"v{i}" for i in range(1, 100)])
        parse_ok(f"programa p; vars {vars_decl} : entero; inicio {{ }} fin")

    def test_s_03_complex_expression(self):
        parse_ok("programa p; inicio { x = 1 + 2 * 3 - 4 / (2 + 1) == 5; } fin")


if __name__ == "__main__":
    unittest.main()
