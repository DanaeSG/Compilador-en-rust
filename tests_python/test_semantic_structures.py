"""Pruebas unitarias de estructuras semánticas."""

import unittest

from compilador.semantics import (
    CuboSemantico,
    DirectorioFunciones,
    Operador,
    SemanticError,
    TablaVariables,
    TipoDato,
)


class SemanticCubeTests(unittest.TestCase):
    def test_cs_01_int_plus_int(self):
        cube = CuboSemantico()
        self.assertEqual(
            cube.consultar(TipoDato.ENTERO, TipoDato.ENTERO, Operador.SUMA),
            TipoDato.ENTERO,
        )

    def test_cs_02_int_plus_float(self):
        cube = CuboSemantico()
        self.assertEqual(
            cube.consultar(TipoDato.ENTERO, TipoDato.FLOTANTE, Operador.SUMA),
            TipoDato.FLOTANTE,
        )

    def test_cs_03_relational_returns_integer(self):
        cube = CuboSemantico()
        self.assertEqual(
            cube.consultar(TipoDato.FLOTANTE, TipoDato.FLOTANTE, Operador.MAYOR),
            TipoDato.ENTERO,
        )

    def test_cs_04_float_assign_to_float(self):
        cube = CuboSemantico()
        self.assertEqual(
            cube.consultar(TipoDato.FLOTANTE, TipoDato.FLOTANTE, Operador.ASIGNA),
            TipoDato.FLOTANTE,
        )

    def test_cs_05_float_assign_to_int_is_invalid(self):
        cube = CuboSemantico()
        with self.assertRaises(ValueError):
            cube.consultar(TipoDato.ENTERO, TipoDato.FLOTANTE, Operador.ASIGNA)


class VariableTableTests(unittest.TestCase):
    def test_tv_01_declare_variable(self):
        table = TablaVariables()
        table.declarar("x", TipoDato.ENTERO, False, -1)
        self.assertEqual(table.buscar("x").tipo, TipoDato.ENTERO)

    def test_tv_02_double_declaration(self):
        table = TablaVariables()
        table.declarar("x", TipoDato.ENTERO, False, -1)
        with self.assertRaises(SemanticError):
            table.declarar("x", TipoDato.FLOTANTE, False, -1)

    def test_tv_03_lookup_missing_variable(self):
        table = TablaVariables()
        with self.assertRaises(SemanticError):
            table.buscar("x")

    def test_tv_04_lookup_valid_variable(self):
        table = TablaVariables()
        table.declarar("y", TipoDato.FLOTANTE, False, -1)
        self.assertEqual(table.buscar("y").tipo, TipoDato.FLOTANTE)


class FunctionDirectoryTests(unittest.TestCase):
    def test_df_01_register_function(self):
        directory = DirectorioFunciones.nuevo("prog")
        directory.registrar_funcion("suma", TipoDato.ENTERO, [("a", TipoDato.ENTERO)])
        self.assertEqual(directory.buscar_funcion("suma").num_params, 1)

    def test_df_02_duplicate_function(self):
        directory = DirectorioFunciones.nuevo("prog")
        directory.registrar_funcion("f", TipoDato.NULA, [])
        with self.assertRaises(SemanticError):
            directory.registrar_funcion("f", TipoDato.NULA, [])

    def test_df_03_lookup_missing_function(self):
        directory = DirectorioFunciones.nuevo("prog")
        with self.assertRaises(SemanticError):
            directory.buscar_funcion("f")

    def test_df_04_resolve_global_variable(self):
        directory = DirectorioFunciones.nuevo("prog")
        directory.declarar_variable("prog", "x", TipoDato.ENTERO, -1)
        directory.registrar_funcion("f", TipoDato.NULA, [])
        self.assertEqual(directory.resolver_variable("f", "x"), TipoDato.ENTERO)

    def test_df_05_local_hides_global(self):
        directory = DirectorioFunciones.nuevo("prog")
        directory.declarar_variable("prog", "x", TipoDato.ENTERO, -1)
        directory.registrar_funcion("f", TipoDato.NULA, [])
        directory.declarar_variable("f", "x", TipoDato.FLOTANTE, -1)
        self.assertEqual(directory.resolver_variable("f", "x"), TipoDato.FLOTANTE)


if __name__ == "__main__":
    unittest.main()
