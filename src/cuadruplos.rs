// Generador de cuadruplos.
// Usa AST, directorio y cubo semantico.
//
// Distribucion de direcciones virtuales
// +--------------------+--------+
// | Segmento           | Rango  |
// +--------------------+--------+
// | Global entero      | 1000+  |
// | Global flotante    | 2000+  |
// | Local entero       | 3000+  |
// | Local flotante     | 4000+  |
// | Temporal entero    | 5000+  |
// | Temporal flotante  | 6000+  |
// | Constante entero   | 8000+  |
// | Constante flotante | 9000+  |
// | Constante string   | 10000+ |
// +--------------------+--------+

use std::collections::{HashMap, VecDeque};

use crate::ast::*;
use crate::semantica::{
    CuboSemantico, DirectorioFunciones, EntradaFuncion, ErrorSemantico, Operador, TipoDato,
};

#[derive(Debug, Clone)]
pub enum OperadorCuadruplo {
    Suma,
    Resta,
    Mul,
    Div,
    Mayor,
    Menor,
    Igual,
    Diferente,
    Asigna,
    Print,
    PrintStr,
    Era,
    Param,
    Gosub,
    Endfunc,
    Goto,
    Gotov,
    Gotof,
    End,
}

impl std::fmt::Display for OperadorCuadruplo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            OperadorCuadruplo::Suma => "+",
            OperadorCuadruplo::Resta => "-",
            OperadorCuadruplo::Mul => "*",
            OperadorCuadruplo::Div => "/",
            OperadorCuadruplo::Mayor => ">",
            OperadorCuadruplo::Menor => "<",
            OperadorCuadruplo::Igual => "==",
            OperadorCuadruplo::Diferente => "!=",
            OperadorCuadruplo::Asigna => "=",
            OperadorCuadruplo::Print => "PRINT",
            OperadorCuadruplo::PrintStr => "PRINTS",
            OperadorCuadruplo::Era => "ERA",
            OperadorCuadruplo::Param => "PARAM",
            OperadorCuadruplo::Gosub => "GOSUB",
            OperadorCuadruplo::Endfunc => "ENDFUNC",
            OperadorCuadruplo::Goto => "GOTO",
            OperadorCuadruplo::Gotov => "GOTOV",
            OperadorCuadruplo::Gotof => "GOTOF",
            OperadorCuadruplo::End => "END",
        };
        write!(f, "{}", s)
    }
}

#[derive(Debug, Clone)]
pub struct Cuadruplo {
    pub op: OperadorCuadruplo,
    pub arg1: Option<i32>,
    pub arg2: Option<i32>,
    pub res: Option<i32>,
}

impl std::fmt::Display for Cuadruplo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let a1 = self.arg1.map_or("_".to_string(), |v| v.to_string());
        let a2 = self.arg2.map_or("_".to_string(), |v| v.to_string());
        let r = self.res.map_or("_".to_string(), |v| v.to_string());
        write!(f, "({}, {}, {}, {})", self.op, a1, a2, r)
    }
}

// Cola de salida de cuadruplos.
#[derive(Debug, Clone)]
pub struct FilaCuadruplos {
    pub items: VecDeque<Cuadruplo>,
}

impl FilaCuadruplos {
    pub fn new() -> Self {
        Self { items: VecDeque::new() }
    }

    pub fn push(&mut self, q: Cuadruplo) {
        self.items.push_back(q);
    }

    pub fn dump(&self) -> String {
        let mut out = String::new();
        for (i, q) in self.items.iter().enumerate() {
            out.push_str(&format!("[{}] {}\n", i, q));
        }
        out
    }
}

// Administra direcciones virtuales.
#[derive(Debug, Clone)]
struct Memoria {
    const_int: HashMap<i64, i32>,
    const_float: HashMap<u64, i32>,
    const_str: HashMap<String, i32>,
    next_global_int: i32,
    next_global_float: i32,
    local_ctx: HashMap<String, MemoriaLocalFunc>,
    next_const_int: i32,
    next_const_float: i32,
    next_const_str: i32,
}

#[derive(Debug, Clone)]
struct MemoriaLocalFunc {
    next_local_int: i32,
    next_local_float: i32,
    next_temp_int: i32,
    next_temp_float: i32,
}

impl Memoria {
    fn new() -> Self {
        Self {
            const_int: HashMap::new(),
            const_float: HashMap::new(),
            const_str: HashMap::new(),
            next_global_int: 1000,
            next_global_float: 2000,
            local_ctx: HashMap::new(),
            next_const_int: 8000,
            next_const_float: 9000,
            next_const_str: 10000,
        }
    }

    fn alloc_global(&mut self, tipo: &TipoDato) -> i32 {
        match tipo {
            TipoDato::Entero | TipoDato::Nula => {
                let d = self.next_global_int;
                self.next_global_int += 1;
                d
            }
            TipoDato::Flotante => {
                let d = self.next_global_float;
                self.next_global_float += 1;
                d
            }
        }
    }

    fn crear_contexto_funcion(&mut self, ambito: &str) {
        // Reinicia contadores por funcion.
        self.local_ctx.insert(
            ambito.to_string(),
            MemoriaLocalFunc {
                next_local_int: 3000,
                next_local_float: 4000,
                next_temp_int: 5000,
                next_temp_float: 6000,
            },
        );
    }

    fn alloc_local(&mut self, ambito: &str, tipo: &TipoDato) -> i32 {
        let ctx = self.local_ctx.entry(ambito.to_string()).or_insert(MemoriaLocalFunc {
            next_local_int: 3000,
            next_local_float: 4000,
            next_temp_int: 5000,
            next_temp_float: 6000,
        });
        match tipo {
            TipoDato::Entero | TipoDato::Nula => {
                let d = ctx.next_local_int;
                ctx.next_local_int += 1;
                d
            }
            TipoDato::Flotante => {
                let d = ctx.next_local_float;
                ctx.next_local_float += 1;
                d
            }
        }
    }

    fn alloc_temp(&mut self, ambito: &str, tipo: &TipoDato) -> i32 {
        let ctx = self.local_ctx.entry(ambito.to_string()).or_insert(MemoriaLocalFunc {
            next_local_int: 3000,
            next_local_float: 4000,
            next_temp_int: 5000,
            next_temp_float: 6000,
        });
        match tipo {
            TipoDato::Entero => {
                let d = ctx.next_temp_int;
                ctx.next_temp_int += 1;
                d
            }
            TipoDato::Flotante => {
                let d = ctx.next_temp_float;
                ctx.next_temp_float += 1;
                d
            }
            TipoDato::Nula => {
                let d = ctx.next_temp_int;
                ctx.next_temp_int += 1;
                d
            }
        }
    }

    fn alloc_const_int(&mut self, v: i64) -> i32 {
        if let Some(dir) = self.const_int.get(&v) {
            return *dir;
        }
        let d = self.next_const_int;
        self.next_const_int += 1;
        self.const_int.insert(v, d);
        d
    }

    fn alloc_const_float(&mut self, v: f64) -> i32 {
        let key = v.to_bits();
        if let Some(dir) = self.const_float.get(&key) {
            return *dir;
        }
        let d = self.next_const_float;
        self.next_const_float += 1;
        self.const_float.insert(key, d);
        d
    }

    fn alloc_const_str(&mut self, v: &str) -> i32 {
        if let Some(dir) = self.const_str.get(v) {
            return *dir;
        }
        let d = self.next_const_str;
        self.next_const_str += 1;
        self.const_str.insert(v.to_string(), d);
        d
    }
}

#[derive(Debug)]
pub struct GeneradorCuadruplos {
    directorio: DirectorioFunciones,
    cubo: CuboSemantico,
    pila_operadores: Vec<Operador>,
    pila_operandos: Vec<i32>,
    pila_tipos: Vec<TipoDato>,
    pila_saltos: Vec<usize>,
    pub fila: FilaCuadruplos,
    memoria: Memoria,
    nombre_global: String,
    end_generado: bool,
}

impl GeneradorCuadruplos {
    pub fn new(prog: &Programa, directorio: &DirectorioFunciones, cubo: &CuboSemantico) -> Self {
        let mut gen = Self {
            directorio: directorio.clone(),
            cubo: cubo.clone(),
            pila_operadores: Vec::new(),
            pila_operandos: Vec::new(),
            pila_tipos: Vec::new(),
            pila_saltos: Vec::new(),
            fila: FilaCuadruplos::new(),
            memoria: Memoria::new(),
            nombre_global: prog.nombre.clone(),
            end_generado: false,
        };
        gen.preasignar_variables(prog);
        gen
    }

    fn preasignar_variables(&mut self, prog: &Programa) {
        // Asigna direcciones antes de emitir cuadruplos.
        let global_scope = self.nombre_global.clone();
        let globals: Vec<(String, TipoDato)> = self
            .directorio
            .funciones
            .get(&global_scope)
            .map(|ef| {
                ef.tabla_vars
                    .variables
                    .iter()
                    .map(|(id, ev)| (id.clone(), ev.tipo.clone()))
                    .collect()
            })
            .unwrap_or_default();
        for (id, tipo) in globals {
            let dir = self.memoria.alloc_global(&tipo);
            let _ = self.directorio.asignar_dir_variable(&global_scope, &id, dir);
        }

        for func in &prog.funcs {
            self.preasignar_funcion(func);
        }
    }

    fn preasignar_funcion(&mut self, func: &Funcion) {
        // Cada función recibe su propio contexto local/temporal.
        let ambito = &func.nombre;
        self.memoria.crear_contexto_funcion(ambito);
        let locales: Vec<(String, TipoDato)> = self
            .directorio
            .funciones
            .get(ambito)
            .map(|ef| {
                ef.tabla_vars
                    .variables
                    .iter()
                    .map(|(id, ev)| (id.clone(), ev.tipo.clone()))
                    .collect()
            })
            .unwrap_or_default();
        for (id, tipo) in locales {
            let dir = self.memoria.alloc_local(ambito, &tipo);
            let _ = self.directorio.asignar_dir_variable(ambito, &id, dir);
        }
    }

    pub fn generar(&mut self, prog: &Programa) -> Result<(), String> {
        // PN-NL01: PROGRAMA_GENERAR_GOTO_MAIN_PENDIENTE
        self.fila.push(Cuadruplo {
            op: OperadorCuadruplo::Goto,
            arg1: None,
            arg2: None,
            res: None,
        });
        self.pila_saltos.push(0);

        for func in &prog.funcs {
            // PN-NL02: FUNC_GUARDAR_DIR_INICIO
            // Guarda indice inicial para GOSUB.
            self.directorio
                .asignar_dir_inicio_funcion(&func.nombre, self.fila.items.len())
                .map_err(|e| e.to_string())?;
            self.generar_cuerpo(&func.cuerpo, &func.nombre)?;
            // PN-NL05: FUNC_EMITIR_ENDFUNC
            self.fila.push(Cuadruplo {
                op: OperadorCuadruplo::Endfunc,
                arg1: None,
                arg2: None,
                res: None,
            });
        }

        // PN-NL03: PROGRAMA_RELLENAR_GOTO_MAIN
        // El GOTO inicial salta al main.
        let main_inicio = self.fila.items.len() as i32;
        let idx_goto_main = self.pila_saltos.pop().ok_or_else(|| "Falta GOTO main pendiente".to_string())?;
        self.rellenar_salto(idx_goto_main, main_inicio)?;

        self.generar_cuerpo(&prog.cuerpo, &prog.nombre)?;
        // PN-NL04: PROGRAMA_EMITIR_END
        self.fila.push(Cuadruplo {
            op: OperadorCuadruplo::End,
            arg1: None,
            arg2: None,
            res: None,
        });
        self.end_generado = true;
        Ok(())
    }

    fn generar_cuerpo(&mut self, stmts: &[Estatuto], ambito: &str) -> Result<(), String> {
        for stmt in stmts {
            self.generar_estatuto(stmt, ambito)?;
        }
        Ok(())
    }

    fn es_main(&self, ambito: &str) -> bool {
        ambito == self.nombre_global
    }

    fn generar_estatuto(&mut self, stmt: &Estatuto, ambito: &str) -> Result<(), String> {
        match stmt {
            Estatuto::Asigna(id, expr) => self.generar_asignacion(id, expr, ambito),
            Estatuto::Imprime(alts) => self.generar_imprime(alts, ambito),
            Estatuto::Bloque(stmts) => self.generar_cuerpo(stmts, ambito),
            Estatuto::Llamada(ll) => {
                let _ = self.generar_llamada(ll, ambito, false)?;
                Ok(())
            }
            Estatuto::Condicion { cond, entonces, sino } => self.generar_condicion(cond, entonces, sino, ambito),
            Estatuto::Ciclo { cond, cuerpo } => self.generar_ciclo(cond, cuerpo, ambito),
            Estatuto::Regresa { valor } => self.generar_regresa(valor, ambito),
        }
    }

    fn generar_condicion(
        &mut self,
        cond: &Expresion,
        entonces: &[Estatuto],
        sino: &Option<Vec<Estatuto>>,
        ambito: &str,
    ) -> Result<(), String> {
        // PN-NL06: IF_VALIDAR_Y_OBTENER_COND
        self.procesar_expresion(cond, ambito)?;
        let (dir_cond, tipo_cond) = self.pop_operando()?;
        if tipo_cond != TipoDato::Entero {
            return Err(ErrorSemantico::CondicionNoBooleana {
                contexto: "si".to_string(),
                tipo: tipo_cond.to_string(),
            }
            .to_string());
        }

        // PN-NL07: IF_GENERAR_GOTOF_PENDIENTE
        // El destino se rellena al cerrar bloques.
        let idx_gotof = self.fila.items.len();
        self.fila.push(Cuadruplo {
            op: OperadorCuadruplo::Gotof,
            arg1: Some(dir_cond),
            arg2: None,
            res: None,
        });
        self.pila_saltos.push(idx_gotof);

        self.generar_cuerpo(entonces, ambito)?;

        if let Some(bloque_sino) = sino {
            // PN-NL09: IF_CON_SINO_ABRIR_BLOQUE_SINO
            let idx_goto_fin = self.fila.items.len();
            self.fila.push(Cuadruplo {
                op: OperadorCuadruplo::Goto,
                arg1: None,
                arg2: None,
                res: None,
            });
            self.pila_saltos.push(idx_goto_fin);

            let pendiente_gotof = self.pila_saltos.remove(self.pila_saltos.len() - 2);
            self.rellenar_salto(pendiente_gotof, self.fila.items.len() as i32)?;

            self.generar_cuerpo(bloque_sino, ambito)?;

            // PN-NL10: IF_CON_SINO_CERRAR
            let pendiente_goto = self.pila_saltos.pop().ok_or_else(|| "Falta GOTO pendiente en sino".to_string())?;
            self.rellenar_salto(pendiente_goto, self.fila.items.len() as i32)?;
        } else {
            // PN-NL08: IF_SIN_SINO_CERRAR
            let pendiente_gotof = self.pila_saltos.pop().ok_or_else(|| "Falta GOTOF pendiente en si".to_string())?;
            self.rellenar_salto(pendiente_gotof, self.fila.items.len() as i32)?;
        }

        Ok(())
    }

    fn generar_ciclo(&mut self, cond: &Expresion, cuerpo: &[Estatuto], ambito: &str) -> Result<(), String> {
        // PN-NL11: WHILE_GUARDAR_RETORNO
        let inicio_ciclo = self.fila.items.len();
        self.pila_saltos.push(inicio_ciclo);

        // PN-NL12: WHILE_VALIDAR_Y_OBTENER_COND
        self.procesar_expresion(cond, ambito)?;
        let (dir_cond, tipo_cond) = self.pop_operando()?;
        if tipo_cond != TipoDato::Entero {
            return Err(ErrorSemantico::CondicionNoBooleana {
                contexto: "mientras".to_string(),
                tipo: tipo_cond.to_string(),
            }
            .to_string());
        }

        // PN-NL13: WHILE_GENERAR_GOTOF_PENDIENTE
        let idx_gotof = self.fila.items.len();
        self.fila.push(Cuadruplo {
            op: OperadorCuadruplo::Gotof,
            arg1: Some(dir_cond),
            arg2: None,
            res: None,
        });
        self.pila_saltos.push(idx_gotof);

        self.generar_cuerpo(cuerpo, ambito)?;

        // PN-NL14: WHILE_CERRAR_Y_RELLENAR
        // Cierra primero GOTOF y luego regreso.
        let gotof_pendiente = self.pila_saltos.pop().ok_or_else(|| "Falta GOTOF pendiente en mientras".to_string())?;
        let regreso = self.pila_saltos.pop().ok_or_else(|| "Falta inicio de ciclo".to_string())?;

        self.fila.push(Cuadruplo {
            op: OperadorCuadruplo::Goto,
            arg1: None,
            arg2: None,
            res: Some(regreso as i32),
        });

        self.rellenar_salto(gotof_pendiente, self.fila.items.len() as i32)?;
        Ok(())
    }

    fn generar_regresa(&mut self, valor: &Expresion, ambito: &str) -> Result<(), String> {
        // PN-NL22: RETURN_VALIDAR_CONTEXTO_Y_TIPO
        if self.es_main(ambito) {
            return Err(ErrorSemantico::RegresaFueraDeFuncion.to_string());
        }

        let info = self
            .directorio
            .buscar_funcion(ambito)
            .map_err(|e| e.to_string())?
            .clone();

        if info.tipo_retorno == TipoDato::Nula {
            return Err(ErrorSemantico::RegresaEnFuncionNula {
                funcion: ambito.to_string(),
            }
            .to_string());
        }

        self.procesar_expresion(valor, ambito)?;
        let (dir_expr, tipo_expr) = self.pop_operando()?;
        self.cubo
            .consultar(&info.tipo_retorno, &tipo_expr, &Operador::Asigna)
            .map_err(|e| e.to_string())?;

        let dir_global_retorno = self
            .directorio
            .resolver_dir_variable(&self.nombre_global, ambito)
            .map_err(|e| e.to_string())?;

        // PN-NL23: RETURN_COPIAR_A_GLOBAL_Y_ENDFUNC
        self.fila.push(Cuadruplo {
            op: OperadorCuadruplo::Asigna,
            arg1: Some(dir_expr),
            arg2: None,
            res: Some(dir_global_retorno),
        });

        self.fila.push(Cuadruplo {
            op: OperadorCuadruplo::Endfunc,
            arg1: None,
            arg2: None,
            res: None,
        });

        Ok(())
    }

    fn rellenar_salto(&mut self, idx: usize, destino: i32) -> Result<(), String> {
        let q = self
            .fila
            .items
            .get_mut(idx)
            .ok_or_else(|| format!("Índice de salto inválido: {}", idx))?;
        q.res = Some(destino);
        Ok(())
    }

    fn generar_asignacion(&mut self, id: &str, expr: &Expresion, ambito: &str) -> Result<(), String> {
        // PN-L08: ASIGNA_EMITIR
        self.procesar_expresion(expr, ambito)?;
        self.generar_asignacion_cuadruplo(id, ambito)
    }

    fn generar_asignacion_cuadruplo(&mut self, id: &str, ambito: &str) -> Result<(), String> {
        let (dir_expr, tipo_expr) = self.pop_operando()?;
        let tipo_id = self.directorio.resolver_variable(ambito, id).map_err(|e| e.to_string())?;
        self.cubo
            .consultar(&tipo_id, &tipo_expr, &Operador::Asigna)
            .map_err(|e| e.to_string())?;
        let dir_id = self
            .directorio
            .resolver_dir_variable(ambito, id)
            .map_err(|e| e.to_string())?;

        self.fila.push(Cuadruplo {
            op: OperadorCuadruplo::Asigna,
            arg1: Some(dir_expr),
            arg2: None,
            res: Some(dir_id),
        });
        Ok(())
    }

    fn generar_imprime(&mut self, alts: &[ImprimeAlt], ambito: &str) -> Result<(), String> {
        for alt in alts {
            match alt {
                ImprimeAlt::Expr(expr) => {
                    // PN-L09: PRINT_EXPRESION_EMITIR
                    self.procesar_expresion(expr, ambito)?;
                    self.generar_print()?;
                }
                ImprimeAlt::Letrero(s) => {
                    // PN-L10: PRINT_STRING_EMITIR
                    self.generar_prints(s)
                }
            }
        }
        Ok(())
    }

    fn generar_print(&mut self) -> Result<(), String> {
        let (dir_expr, _tipo) = self.pop_operando()?;
        self.fila.push(Cuadruplo {
            op: OperadorCuadruplo::Print,
            arg1: Some(dir_expr),
            arg2: None,
            res: None,
        });
        Ok(())
    }

    fn generar_prints(&mut self, s: &str) {
        let dir = self.memoria.alloc_const_str(s);
        self.fila.push(Cuadruplo {
            op: OperadorCuadruplo::PrintStr,
            arg1: Some(dir),
            arg2: None,
            res: None,
        });
    }

    fn procesar_expresion(&mut self, expr: &Expresion, ambito: &str) -> Result<(), String> {
        self.procesar_exp(&expr.izq, ambito)?;
        if let Some((op_rel, exp_der)) = &expr.op {
            // PN-L06: EXPRESION_PUSH_OP_RELACIONAL
            self.push_operador_relacional(op_rel);
            self.procesar_exp(exp_der, ambito)?;
            // PN-L07: REDUCIR_TOPE_Y_GENERAR_TEMP
            self.reducir_expresion(ambito)?;
        }
        Ok(())
    }

    fn procesar_exp(&mut self, exp: &Exp, ambito: &str) -> Result<(), String> {
        self.procesar_termino(&exp.termino, ambito)?;
        for (op_arit, term) in &exp.cont {
            // PN-L04: EXP_PUSH_OP_SUMA_RESTA
            self.push_operador_suma_resta(op_arit);
            self.procesar_termino(term, ambito)?;
            // PN-L07: REDUCIR_TOPE_Y_GENERAR_TEMP
            self.reducir_expresion(ambito)?;
        }
        Ok(())
    }

    fn procesar_termino(&mut self, term: &Termino, ambito: &str) -> Result<(), String> {
        self.procesar_factor(&term.factor, ambito)?;
        for (op_mul, fac) in &term.cont {
            // PN-L05: TERMINO_PUSH_OP_MUL_DIV
            self.push_operador_mul_div(op_mul);
            self.procesar_factor(fac, ambito)?;
            // PN-L07: REDUCIR_TOPE_Y_GENERAR_TEMP
            self.reducir_expresion(ambito)?;
        }
        Ok(())
    }

    fn procesar_factor(&mut self, factor: &Factor, ambito: &str) -> Result<(), String> {
        match factor {
            Factor::Cte(Constante::Entero(v)) => {
                // PN-L01: FACTOR_PUSH_CONSTANTE
                self.push_constante_entero(*v);
                Ok(())
            }
            Factor::Cte(Constante::Flotante(v)) => {
                // PN-L01: FACTOR_PUSH_CONSTANTE
                self.push_constante_flotante(*v);
                Ok(())
            }
            Factor::Id(id) | Factor::PosId(id) => {
                // PN-L02: FACTOR_PUSH_IDENTIFICADOR
                self.push_variable(id, ambito)
            }
            Factor::NegId(id) => {
                // PN-L03: FACTOR_NEGACION_UNARIA
                self.generar_negacion_unaria(id, ambito)
            }
            Factor::Paren(expr) => self.procesar_expresion(expr, ambito),
            Factor::Llamada(ll) => {
                let _tipo = self.generar_llamada(ll, ambito, true)?;
                Ok(())
            }
        }
    }

    fn generar_llamada(&mut self, ll: &Llamada, ambito_llamador: &str, como_expresion: bool) -> Result<TipoDato, String> {
        // PN-NL15: CALL_VALIDAR_EXISTENCIA_Y_ARIDAD
        let entrada = self.validar_existencia_y_aridad_llamada(ll)?;

        // PN-NL16: CALL_GENERAR_ERA
        let dir_nombre = self.generar_era_llamada(&ll.nombre);
        self.procesar_argumentos_llamada(ll, ambito_llamador, &entrada)?;
        self.validar_fin_parametros(ll, &entrada)?;

        // PN-NL20: CALL_GENERAR_GOSUB
        self.generar_gosub(dir_nombre, entrada.dir_inicio as i32);
        self.recuperar_retorno_llamada(ll, ambito_llamador, &entrada, como_expresion)?;

        Ok(entrada.tipo_retorno)
    }

    // PN-L04: EXP_PUSH_OP_SUMA_RESTA
    fn push_operador_suma_resta(&mut self, op_arit: &OpArit) {
        let op = match op_arit {
            OpArit::Plus => Operador::Suma,
            OpArit::Minus => Operador::Resta,
        };
        self.push_operador(op);
    }

    // PN-L05: TERMINO_PUSH_OP_MUL_DIV
    fn push_operador_mul_div(&mut self, op_mul: &OpMul) {
        let op = match op_mul {
            OpMul::Star => Operador::Mul,
            OpMul::Slash => Operador::Div,
        };
        self.push_operador(op);
    }

    // PN-L06: EXPRESION_PUSH_OP_RELACIONAL
    fn push_operador_relacional(&mut self, op_rel: &OpRel) {
        let op = match op_rel {
            OpRel::Gt => Operador::Mayor,
            OpRel::Lt => Operador::Menor,
            OpRel::EqEq => Operador::Igual,
            OpRel::Neq => Operador::Diferente,
        };
        self.push_operador(op);
    }

    // PN-NL15: CALL_VALIDAR_EXISTENCIA_Y_ARIDAD
    fn validar_existencia_y_aridad_llamada(&self, ll: &Llamada) -> Result<EntradaFuncion, String> {
        let entrada = self
            .directorio
            .buscar_funcion(&ll.nombre)
            .map_err(|e| e.to_string())?
            .clone();
        if ll.args.len() != entrada.num_params {
            return Err(ErrorSemantico::ArityMismatch {
                funcion: ll.nombre.clone(),
                esperados: entrada.num_params,
                recibidos: ll.args.len(),
            }
            .to_string());
        }
        Ok(entrada)
    }

    // PN-NL16: CALL_GENERAR_ERA
    fn generar_era_llamada(&mut self, nombre_funcion: &str) -> i32 {
        let dir_nombre = self.memoria.alloc_const_str(nombre_funcion);
        self.fila.push(Cuadruplo {
            op: OperadorCuadruplo::Era,
            arg1: Some(dir_nombre),
            arg2: None,
            res: None,
        });
        dir_nombre
    }

    fn procesar_argumentos_llamada(
        &mut self,
        ll: &Llamada,
        ambito_llamador: &str,
        entrada: &EntradaFuncion,
    ) -> Result<(), String> {
        for (i, arg) in ll.args.iter().enumerate() {
            let (dir_arg, _) =
                self.validar_argumento_llamada(arg, ambito_llamador, &entrada.tipos_params[i])?;
            let dir_param = self.obtener_direccion_parametro(entrada, i)?;
            self.generar_param(dir_arg, dir_param);
        }
        Ok(())
    }

    // PN-NL17: CALL_ARGUMENTO_VALIDAR_TIPO
    fn validar_argumento_llamada(
        &mut self,
        arg: &Expresion,
        ambito_llamador: &str,
        tipo_param: &TipoDato,
    ) -> Result<(i32, TipoDato), String> {
        self.procesar_expresion(arg, ambito_llamador)?;
        let (dir_arg, tipo_arg) = self.pop_operando()?;
        self.cubo
            .consultar(tipo_param, &tipo_arg, &Operador::Asigna)
            .map_err(|e| e.to_string())?;
        Ok((dir_arg, tipo_arg))
    }

    fn obtener_direccion_parametro(&self, entrada: &EntradaFuncion, idx: usize) -> Result<i32, String> {
        let nombre_param = &entrada.nombres_params[idx];
        entrada
            .tabla_vars
            .buscar(nombre_param)
            .map_err(|e| e.to_string())
            .map(|v| v.dir_virtual)
    }

    // PN-NL18: CALL_GENERAR_PARAM
    fn generar_param(&mut self, dir_arg: i32, dir_param: i32) {
        self.fila.push(Cuadruplo {
            op: OperadorCuadruplo::Param,
            arg1: Some(dir_arg),
            arg2: None,
            res: Some(dir_param),
        });
    }

    // PN-NL19: CALL_VALIDAR_FIN_PARAMETROS
    fn validar_fin_parametros(&self, ll: &Llamada, entrada: &EntradaFuncion) -> Result<(), String> {
        // Valida que la funcion tenga inicio.
        if entrada.dir_inicio == 0 {
            return Err(ErrorSemantico::FuncionSinDirInicio {
                funcion: ll.nombre.clone(),
            }
            .to_string());
        }
        Ok(())
    }

    // PN-NL20: CALL_GENERAR_GOSUB
    fn generar_gosub(&mut self, dir_nombre: i32, dir_inicio: i32) {
        self.fila.push(Cuadruplo {
            op: OperadorCuadruplo::Gosub,
            arg1: Some(dir_nombre),
            arg2: None,
            res: Some(dir_inicio),
        });
    }

    // PN-NL21: CALL_RECUPERAR_RETORNO_A_TEMP
    fn recuperar_retorno_llamada(
        &mut self,
        ll: &Llamada,
        ambito_llamador: &str,
        entrada: &EntradaFuncion,
        como_expresion: bool,
    ) -> Result<(), String> {
        if entrada.tipo_retorno == TipoDato::Nula {
            return Ok(());
        }
        // Copia retorno global a temporal.
        let dir_global_retorno = self
            .directorio
            .resolver_dir_variable(&self.nombre_global, &ll.nombre)
            .map_err(|e| e.to_string())?;
        let dir_temp = self.memoria.alloc_temp(ambito_llamador, &entrada.tipo_retorno);
        self.fila.push(Cuadruplo {
            op: OperadorCuadruplo::Asigna,
            arg1: Some(dir_global_retorno),
            arg2: None,
            res: Some(dir_temp),
        });
        if como_expresion {
            self.pila_operandos.push(dir_temp);
            self.pila_tipos.push(entrada.tipo_retorno.clone());
        }
        Ok(())
    }

    fn push_constante_entero(&mut self, v: i64) {
        let dir = self.memoria.alloc_const_int(v);
        self.pila_operandos.push(dir);
        self.pila_tipos.push(TipoDato::Entero);
    }

    fn push_constante_flotante(&mut self, v: f64) {
        let dir = self.memoria.alloc_const_float(v);
        self.pila_operandos.push(dir);
        self.pila_tipos.push(TipoDato::Flotante);
    }

    fn push_variable(&mut self, id: &str, ambito: &str) -> Result<(), String> {
        let tipo = self.directorio.resolver_variable(ambito, id).map_err(|e| e.to_string())?;
        let dir = self
            .directorio
            .resolver_dir_variable(ambito, id)
            .map_err(|e| e.to_string())?;
        self.pila_operandos.push(dir);
        self.pila_tipos.push(tipo);
        Ok(())
    }

    fn push_operador(&mut self, op: Operador) {
        self.pila_operadores.push(op);
    }

    fn generar_negacion_unaria(&mut self, id: &str, ambito: &str) -> Result<(), String> {
        let tipo = self.directorio.resolver_variable(ambito, id).map_err(|e| e.to_string())?;
        let dir_id = self
            .directorio
            .resolver_dir_variable(ambito, id)
            .map_err(|e| e.to_string())?;

        let dir_cero = match tipo {
            TipoDato::Entero | TipoDato::Nula => self.memoria.alloc_const_int(0),
            TipoDato::Flotante => self.memoria.alloc_const_float(0.0),
        };

        self.pila_operandos.push(dir_cero);
        self.pila_tipos.push(tipo.clone());
        self.pila_operandos.push(dir_id);
        self.pila_tipos.push(tipo);
        self.push_operador(Operador::Resta);
        self.reducir_expresion(ambito)
    }

    fn reducir_expresion(&mut self, ambito: &str) -> Result<(), String> {
        let op = self
            .pila_operadores
            .pop()
            .ok_or_else(|| "Pila de operadores vacia".to_string())?;
        let (dir_der, tipo_der) = self.pop_operando()?;
        let (dir_izq, tipo_izq) = self.pop_operando()?;

        let tipo_res = self
            .cubo
            .consultar(&tipo_izq, &tipo_der, &op)
            .map_err(|e| e.to_string())?;
        // Cada reduccion crea un temporal.
        let dir_temp = self.memoria.alloc_temp(ambito, &tipo_res);

        let op_cuad = match op {
            Operador::Suma => OperadorCuadruplo::Suma,
            Operador::Resta => OperadorCuadruplo::Resta,
            Operador::Mul => OperadorCuadruplo::Mul,
            Operador::Div => OperadorCuadruplo::Div,
            Operador::Mayor => OperadorCuadruplo::Mayor,
            Operador::Menor => OperadorCuadruplo::Menor,
            Operador::Igual => OperadorCuadruplo::Igual,
            Operador::Diferente => OperadorCuadruplo::Diferente,
            Operador::Asigna => OperadorCuadruplo::Asigna,
        };

        self.fila.push(Cuadruplo {
            op: op_cuad,
            arg1: Some(dir_izq),
            arg2: Some(dir_der),
            res: Some(dir_temp),
        });

        self.pila_operandos.push(dir_temp);
        self.pila_tipos.push(tipo_res);
        Ok(())
    }

    fn pop_operando(&mut self) -> Result<(i32, TipoDato), String> {
        let dir = self
            .pila_operandos
            .pop()
            .ok_or_else(|| "Pila de operandos vacia".to_string())?;
        let tipo = self
            .pila_tipos
            .pop()
            .ok_or_else(|| "Pila de tipos vacia".to_string())?;
        Ok((dir, tipo))
    }
}
