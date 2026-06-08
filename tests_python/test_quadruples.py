"""Pruebas smoke de generación de cuádruplos."""

import unittest

from compilador.compiler import compile_source


class QuadrupleSmokeTests(unittest.TestCase):
    def test_quad_generation_contains_return(self):
        src = (
            "programa t; vars r:entero; "
            "entero doble(n:entero) { { regresa n + n; } }; "
            "inicio { r = doble(5); } fin"
        )
        result = compile_source(src)
        self.assertIsNotNone(result.quadruple_generator)
        dump = result.quadruple_generator.quadruples.dump()
        self.assertIn("GOSUB", dump)
        self.assertIn("RETURN", dump)
        self.assertIn("(=, 5000, _, 1001)", dump)


if __name__ == "__main__":
    unittest.main()
