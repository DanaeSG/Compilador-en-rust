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


if __name__ == "__main__":
    unittest.main()
