// src/analizador_semantico.rs
// ------------------------------------------------------------------------------
//  PUNTOS NEURÁLGICOS IMPLEMENTADOS
//
//  PN-1  Al entrar a <Programa>
//        - Crear DirectorioFunciones con el nombre del programa.
//        - Registrar variables globales en el ámbito global.
//
//  PN-2  Al entrar a cada <FUNC>
//        - Verificar que la función no esté doblemente declarada.
//        - Registrar la función (tipo retorno + params) en el Directorio.
//        - Registrar variables locales de la función.
//
//  PN-3  En <ASIGNA>: id = <EXPRESION>
//        - Verificar que 'id' esté declarada en el ámbito visible.
//        - Inferir el tipo de la expresión derecha.
//        - Consultar el cubo semántico con (op_izq, op_der, op).
//
//  PN-4  En <EXPRESION> con operador relacional
//        - Inferir tipos de ambos <EXP>.
//        - Consultar cubo con (op_izq, op_der, op).
//
//  PN-5  En <EXP> / <TERMINO> con operadores aritméticos
//        - Inferir tipos de operandos.
//        - Consultar cubo con (op_izq, op_der, op).
//
//  PN-6  En <FACTOR> con Id
//        - Verificar que el identificador esté declarado en ámbito visible.
//        - Devolver su tipo para la inferencia ascendente.
//
//  PN-7  En <LLAMADA> (dentro de expresión o estatuto)
//        - Verificar que la función esté declarada.
//        - Verificar aridad (número de argumentos).
//        - Verificar tipo de cada argumento contra tipo del parámetro.
//        - Devolver tipo de retorno de la función.
//
//  PN-8  En <CONDICION> y <CICLO>
//        - Verificar que la expresión condicional produce un tipo válido
//          (entero o flotante — no nula).
// ------------------------------------------------------------------------------

use crate::ast::*;
use crate::semantica::*;

pub struct AnalizadorSemantico {
    pub directorio: DirectorioFunciones,
    pub cubo:       CuboSemantico,
    pub errores:    Vec<ErrorSemantico>,
}

impl AnalizadorSemantico {
    pub fn new() -> Self {
        Self {
            directorio: DirectorioFunciones::new("__global__"),
            cubo:       CuboSemantico::new(),
            errores:    Vec::new(),
        }
    }

    //  Punto de entrada 
    pub fn analizar(&mut self, prog: &Programa) {
        // PN-1: Inicializar directorio con el nombre del programa
        self.directorio = DirectorioFunciones::new(&prog.nombre);

        // PN-1: Registrar variables globales
        self.registrar_vars_en_ambito(&prog.vars, &prog.nombre);

        // PN-2: Registrar todas las funciones antes de analizar los cuerpos
        //       (permite llamadas hacia adelante)
        for func in &prog.funcs {
            self.registrar_funcion(func);
        }

        // PN-2 (cont): Analizar cuerpos de funciones
        for func in &prog.funcs {
            self.analizar_cuerpo(&func.cuerpo, &func.nombre);
        }

        // PN-1 (cont): Analizar cuerpo principal
        self.analizar_cuerpo(&prog.cuerpo, &prog.nombre);
    }

    //  PN-1 / PN-2: Registrar variables en un ámbito 
    fn registrar_vars_en_ambito(&mut self, decls: &[DeclVars], ambito: &str) {
        for decl in decls {
            let tipo = TipoDato::from_tipo(&decl.tipo);
            for id in &decl.ids {
                if let Err(e) = self.directorio.declarar_variable(ambito, id, tipo.clone()) {
                    self.errores.push(e);
                }
            }
        }
    }

    //  PN-2: Registrar una función en el directorio 
    fn registrar_funcion(&mut self, func: &Funcion) {
        let tipo_ret = TipoDato::from_tipo_func(&func.tipo_retorno);
        let params: Vec<(String, TipoDato)> = func.params.iter()
            .map(|p| (p.nombre.clone(), TipoDato::from_tipo(&p.tipo)))
            .collect();

        match self.directorio.registrar_funcion(&func.nombre, tipo_ret, params) {
            Ok(()) => {
                // Registrar vars locales dentro del ámbito de la función
                self.registrar_vars_en_ambito(&func.vars, &func.nombre);
            }
            Err(e) => self.errores.push(e),
        }
    }

    //  Analizar lista de estatutos 
    fn analizar_cuerpo(&mut self, stmts: &[Estatuto], ambito: &str) {
        for stmt in stmts {
            self.analizar_estatuto(stmt, ambito);
        }
    }

    //  Analizar un estatuto 
    fn analizar_estatuto(&mut self, stmt: &Estatuto, ambito: &str) {
        match stmt {

            // PN-3: Asignación
            Estatuto::Asigna(id, expr) => {
                // tipo del lado izquierdo
                let tipo_id = match self.directorio.resolver_variable(ambito, id) {
                    Ok(t) => t,
                    Err(e) => { self.errores.push(e); return; }
                };
                // tipo de la expresión derecha
                let tipo_expr = match self.inferir_expresion(expr, ambito) {
                    Ok(t) => t,
                    Err(e) => { self.errores.push(e); return; }
                };
                // consultar cubo: (op_izq, op_der, op)
                if let Err(_) = self.cubo.consultar(&tipo_id, &tipo_expr, &Operador::Asigna) {
                    self.errores.push(ErrorSemantico::AsignacionTipoIncompatible {
                        var:       id.clone(),
                        var_tipo:  tipo_id.to_string(),
                        expr_tipo: tipo_expr.to_string(),
                    });
                }
            }

            // PN-8: Condición
            Estatuto::Condicion { cond, entonces, sino } => {
                self.verificar_cond_tipo(cond, ambito);
                self.analizar_cuerpo(entonces, ambito);
                if let Some(sino_body) = sino {
                    self.analizar_cuerpo(sino_body, ambito);
                }
            }

            // PN-8: Ciclo
            Estatuto::Ciclo { cond, cuerpo } => {
                self.verificar_cond_tipo(cond, ambito);
                self.analizar_cuerpo(cuerpo, ambito);
            }

            // PN-7: Llamada como estatuto
            Estatuto::Llamada(ll) => {
                if let Err(e) = self.inferir_llamada(ll, ambito) {
                    self.errores.push(e);
                }
            }

            // Imprime: verificar tipos de expresiones
            Estatuto::Imprime(alts) => {
                for alt in alts {
                    if let ImprimeAlt::Expr(e) = alt {
                        if let Err(err) = self.inferir_expresion(e, ambito) {
                            self.errores.push(err);
                        }
                    }
                }
            }

            Estatuto::Bloque(stmts) => self.analizar_cuerpo(stmts, ambito),
        }
    }

    //  PN-4: Inferir tipo de una Expresion 
    fn inferir_expresion(
        &mut self,
        expr: &Expresion,
        ambito: &str,
    ) -> Result<TipoDato, ErrorSemantico> {
        let tipo_izq = self.inferir_exp(&expr.izq, ambito)?;
        match &expr.op {
            None => Ok(tipo_izq),
            Some((op_rel, exp_der)) => {
                let tipo_der = self.inferir_exp(exp_der, ambito)?;
                let op = match op_rel {
                    OpRel::Gt   => Operador::Mayor,
                    OpRel::Lt   => Operador::Menor,
                    OpRel::EqEq => Operador::Igual,
                    OpRel::Neq  => Operador::Diferente,
                };
                self.cubo.consultar(&tipo_izq, &tipo_der, &op)
                    .map_err(|_msg| ErrorSemantico::TipoIncompatible {
                        op:  format!("{:?}", op_rel),
                        izq: tipo_izq.to_string(),
                        der: tipo_der.to_string(),
                    })
            }
        }
    }

    //  PN-5: Inferir tipo de un Exp (suma/resta) 
    fn inferir_exp(&mut self, exp: &Exp, ambito: &str) -> Result<TipoDato, ErrorSemantico> {
        let mut tipo_acc = self.inferir_termino(&exp.termino, ambito)?;
        for (op_arit, term) in &exp.cont {
            let tipo_der = self.inferir_termino(term, ambito)?;
            let op = match op_arit {
                OpArit::Plus  => Operador::Suma,
                OpArit::Minus => Operador::Resta,
            };
            tipo_acc = self.cubo.consultar(&tipo_acc, &tipo_der, &op)
                .map_err(|_| ErrorSemantico::TipoIncompatible {
                    op:  format!("{:?}", op_arit),
                    izq: tipo_acc.to_string(),
                    der: tipo_der.to_string(),
                })?;
        }
        Ok(tipo_acc)
    }

    //  PN-5: Inferir tipo de un Termino (mul/div) 
    fn inferir_termino(
        &mut self,
        term: &Termino,
        ambito: &str,
    ) -> Result<TipoDato, ErrorSemantico> {
        let mut tipo_acc = self.inferir_factor(&term.factor, ambito)?;
        for (op_mul, fac) in &term.cont {
            let tipo_der = self.inferir_factor(fac, ambito)?;
            let op = match op_mul {
                OpMul::Star  => Operador::Mul,
                OpMul::Slash => Operador::Div,
            };
            tipo_acc = self.cubo.consultar(&tipo_acc, &tipo_der, &op)
                .map_err(|_| ErrorSemantico::TipoIncompatible {
                    op:  format!("{:?}", op_mul),
                    izq: tipo_acc.to_string(),
                    der: tipo_der.to_string(),
                })?;
        }
        Ok(tipo_acc)
    }

    //  PN-6 / PN-7: Inferir tipo de un Factor 
    fn inferir_factor(
        &mut self,
        factor: &Factor,
        ambito: &str,
    ) -> Result<TipoDato, ErrorSemantico> {
        match factor {
            Factor::Cte(c) => Ok(match c {
                Constante::Entero(_)   => TipoDato::Entero,
                Constante::Flotante(_) => TipoDato::Flotante,
            }),

            // PN-6: Id simple
            Factor::Id(id) | Factor::PosId(id) | Factor::NegId(id) => {
                self.directorio.resolver_variable(ambito, id)
            }

            Factor::Paren(expr) => self.inferir_expresion(expr, ambito),

            // PN-7: Llamada dentro de expresión
            Factor::Llamada(ll) => self.inferir_llamada(ll, ambito),
        }
    }

    //  PN-7: Validar llamada y devolver tipo de retorno 
    fn inferir_llamada(
        &mut self,
        ll: &Llamada,
        ambito: &str,
    ) -> Result<TipoDato, ErrorSemantico> {
        // Built-in: escribe acepta cualquier número/tipo de args
        if ll.nombre == "escribe" {
            for arg in &ll.args {
                let _ = self.inferir_expresion(arg, ambito);
            }
            return Ok(TipoDato::Nula);
        }

        let entrada = self.directorio.buscar_funcion(&ll.nombre)?.clone();

        // Verificar aridad
        if ll.args.len() != entrada.num_params {
            self.errores.push(ErrorSemantico::ArityMismatch {
                funcion:   ll.nombre.clone(),
                esperados: entrada.num_params,
                recibidos: ll.args.len(),
            });
        }

        // Verificar tipo de cada argumento
        for (i, (arg, tipo_param)) in
            ll.args.iter().zip(entrada.tipos_params.iter()).enumerate()
        {
            match self.inferir_expresion(arg, ambito) {
                Ok(tipo_arg) => {
                    // usamos el cubo de asignación para verificar compatibilidad
                    if let Err(_) = self.cubo.consultar(
                        tipo_param, &tipo_arg, &Operador::Asigna
                    ) {
                        self.errores.push(ErrorSemantico::TipoIncompatible {
                            op:  format!("arg {} de '{}'", i + 1, ll.nombre),
                            izq: tipo_param.to_string(),
                            der: tipo_arg.to_string(),
                        });
                    }
                }
                Err(e) => self.errores.push(e),
            }
        }

        Ok(entrada.tipo_retorno.clone())
    }

    //  PN-8: Verificar que condición no sea nula 
    fn verificar_cond_tipo(&mut self, cond: &Expresion, ambito: &str) {
        match self.inferir_expresion(cond, ambito) {
            Ok(TipoDato::Nula) => self.errores.push(ErrorSemantico::TipoIncompatible {
                op:  "condición".to_string(),
                izq: "nula".to_string(),
                der: "nula".to_string(),
            }),
            Ok(_) => {}
            Err(e) => self.errores.push(e),
        }
    }

    //  Resultado final 
    pub fn tiene_errores(&self) -> bool { !self.errores.is_empty() }

    pub fn reporte(&self) -> String {
        if self.errores.is_empty() {
            return "Análisis semántico: OK — sin errores.".to_string();
        }
        let mut out = format!("Análisis semántico: {} error(es)\n", self.errores.len());
        for (i, e) in self.errores.iter().enumerate() {
            out.push_str(&format!("  [{}] {}\n", i + 1, e));
        }
        out
    }
}
