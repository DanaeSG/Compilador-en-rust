"""Pruebas de ejecución para la máquina virtual."""

import tempfile
import unittest
from pathlib import Path

from compilador.compiler import run_object_file, run_source, save_object_program
from compilador.vm import VirtualMachineError


class VirtualMachineTests(unittest.TestCase):
    def test_vm_01_runs_compiled_source(self):
        src = (
            "programa t; vars x, y, z : entero; "
            "entero suma(a:entero, b:entero) { { regresa a + b; } }; "
            "inicio { x = suma(2, 3); y = suma(x, 4); z = y + 1; escribe(\"z=\", z); } fin"
        )
        self.assertEqual(run_source(src), ["z=10"])

    def test_vm_02_runs_object_file(self):
        src = (
            "programa t; vars n, acc : entero; "
            "inicio { n = 0; acc = 0; mientras (n < 3) haz { acc = acc + n; n = n + 1; }; "
            "escribe(\"acc=\", acc); } fin"
        )
        with tempfile.TemporaryDirectory() as tmp_dir:
            path = Path(tmp_dir) / "programa.obj"
            save_object_program(src, path)
            self.assertEqual(run_object_file(path), ["acc=3"])

    def test_vm_03_detects_division_by_zero(self):
        src = "programa t; vars x : entero; inicio { x = 1 / 0; } fin"
        with self.assertRaises(VirtualMachineError):
            run_source(src)

    def test_vm_04_runs_factorial_in_main(self):
        src = Path("programas_prueba/p13_factorial_main.patito").read_text(encoding="utf-8")
        self.assertEqual(run_source(src), ["factorial=120"])

    def test_vm_05_runs_factorial_in_function(self):
        src = Path("programas_prueba/p14_factorial_funcion.patito").read_text(encoding="utf-8")
        self.assertEqual(run_source(src), ["factorial=120"])

    def test_vm_06_runs_fibonacci_in_main(self):
        src = Path("programas_prueba/p15_fibonacci_main.patito").read_text(encoding="utf-8")
        self.assertEqual(
            run_source(src),
            ["fib=0", "fib=1", "fib=1", "fib=2", "fib=3", "fib=5", "fib=8"],
        )

    def test_vm_07_runs_fibonacci_in_function(self):
        src = Path("programas_prueba/p16_fibonacci_funcion.patito").read_text(encoding="utf-8")
        self.assertEqual(
            run_source(src),
            ["fib=0", "fib=1", "fib=1", "fib=2", "fib=3", "fib=5", "fib=8"],
        )


if __name__ == "__main__":
    unittest.main()
