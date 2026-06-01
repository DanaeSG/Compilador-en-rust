// Analizador semantico del AST.
// Valida tipos, ambitos, llamadas y retornos.

use crate::ast::*;
use crate::semantica::*;

// Estado del analisis semantico.
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

    // Ejecuta analisis semantico completo.
    pub fn analizar(&mut self, prog: &Programa) {
        // PN-S01: PROGRAMA_INICIALIZAR_DIRECTORIO
        self.inicializar_directorio_programa(&prog.nombre);
        println!("Analizando programa '{}'", prog.nombre);

        // PN-S03: GLOBALES_REGISTRAR
        self.registrar_globales(&prog.vars, &prog.nombre);

        // PN-S04: FUNC_PREDECLARAR_FIRMA
        self.predeclarar_funciones(&prog.funcs);

        // Analiza cuerpos despues de registrar firmas.
        self.analizar_cuerpos_de_funciones(&prog.funcs);

        // Analizar cuerpo principal
        self.analizar_cuerpo(&prog.cuerpo, &prog.nombre);
    }

    // PN-S01: PROGRAMA_INICIALIZAR_DIRECTORIO
    fn inicializar_directorio_programa(&mut self, nombre_prog: &str) {
        self.directorio = DirectorioFunciones::new(nombre_prog);
    }

    // PN-S04: FUNC_PREDECLARAR_FIRMA
    fn predeclarar_funciones(&mut self, funcs: &[Funcion]) {
        for func in funcs {
            self.registrar_funcion(func);
        }
    }

    // PN-S16: FUNC_VALIDAR_RETORNO_MINIMO
    fn analizar_cuerpos_de_funciones(&mut self, funcs: &[Funcion]) {
        for func in funcs {
            self.analizar_cuerpo(&func.cuerpo, &func.nombre);
            self.validar_retorno_funcion(func);
            self.cerrar_funcion(&func.nombre);
        }
    }

    // PN-S03: GLOBALES_REGISTRAR
    fn registrar_globales(&mut self, decls: &[DeclVars], ambito: &str) {
        self.registrar_vars_en_ambito(decls, ambito);
    }

    // PN-S03 / PN-S05: Registrar variables por ámbito
    fn registrar_vars_en_ambito(&mut self, decls: &[DeclVars], ambito: &str) {
        for decl in decls {
            // PN-S02: TIPO_CONVERTIR_SINTACTICO
            let tipo = self.convertir_tipo(&decl.tipo);
            for id in &decl.ids {
                if let Err(e) = self.directorio.declarar_variable(ambito, id, tipo.clone(), DIR_SIN_ASIGNAR) {
                    self.errores.push(e);
                }
            }
        }
    }

    // PN-S04 + PN-S05: predeclarar firma y registrar locales.
    fn registrar_funcion(&mut self, func: &Funcion) {
        // PN-S02: TIPO_CONVERTIR_SINTACTICO
        let tipo_ret = self.convertir_tipo_func(&func.tipo_retorno);
        // PN-S04: firma de parámetros
        let params = self.registrar_parametros(func);

        match self.directorio.registrar_funcion(&func.nombre, tipo_ret, params) {
            Ok(()) => {
                // PN-S05: registrar locales
                self.registrar_locales(&func.vars, &func.nombre);
            }
            Err(e) => self.errores.push(e),
        }
    }

    // PN-S04: FUNC_PREDECLARAR_FIRMA (params)
    fn registrar_parametros(&self, func: &Funcion) -> Vec<(String, TipoDato)> {
        func.params
            .iter()
            .map(|p| (p.nombre.clone(), self.convertir_tipo(&p.tipo)))
            .collect()
    }

    // PN-S05: registrar variables locales.
    fn registrar_locales(&mut self, decls: &[DeclVars], ambito: &str) {
        self.registrar_vars_en_ambito(decls, ambito);
    }

    // Punto de extension para cierre de ambito.
    fn cerrar_funcion(&mut self, _ambito: &str) {}

    // Analiza una secuencia de estatutos dentro de un ámbito.
    fn analizar_cuerpo(&mut self, stmts: &[Estatuto], ambito: &str) {
        for stmt in stmts {
            self.analizar_estatuto(stmt, ambito);
        }
    }

    // Despacha validaciones semánticas por tipo de estatuto.
    fn analizar_estatuto(&mut self, stmt: &Estatuto, ambito: &str) {
        match stmt {

            // PN-S09: ASIGNA_VALIDAR_COMPATIBILIDAD
            Estatuto::Asigna(id, expr) => {
                self.validar_asignacion(id, expr, ambito);
            }

            // PN-S15: COND_VALIDAR_TIPO_ENTERO
            Estatuto::Condicion { cond, entonces, sino } => {
                self.verificar_cond_tipo(cond, ambito);
                self.analizar_cuerpo(entonces, ambito);
                if let Some(sino_body) = sino {
                    self.analizar_cuerpo(sino_body, ambito);
                }
            }

            // PN-S15: COND_VALIDAR_TIPO_ENTERO
            Estatuto::Ciclo { cond, cuerpo } => {
                self.verificar_cond_tipo(cond, ambito);
                self.analizar_cuerpo(cuerpo, ambito);
            }

            // PN-S10 / PN-S11 / PN-S12: validación de llamada
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

            // PN-S13 + PN-S14: validaciones de regresa
            Estatuto::Regresa { valor } => {
                self.procesar_regresa(valor, ambito);
            }

            Estatuto::Bloque(stmts) => self.analizar_cuerpo(stmts, ambito),
        }
    }

    // Valida regresa por contexto y tipo.
    fn procesar_regresa(&mut self, valor: &Expresion, ambito: &str) {
        // PN-S13: RETURN_VALIDAR_CONTEXTO
        let Some(tipo_retorno_func) = self.validar_contexto_regresa(ambito) else {
            return;
        };
        // PN-S14: RETURN_VALIDAR_TIPO
        self.validar_tipo_regresa(valor, ambito, &tipo_retorno_func);
    }

    // PN-S13: RETURN_VALIDAR_CONTEXTO
    fn validar_contexto_regresa(&mut self, ambito: &str) -> Option<TipoDato> {
        if ambito == self.directorio.nombre_prog {
            self.errores.push(ErrorSemantico::RegresaFueraDeFuncion);
            return None;
        }
        match self.directorio.buscar_funcion(ambito) {
            Ok(entrada) => Some(entrada.tipo_retorno.clone()),
            Err(e) => {
                self.errores.push(e);
                None
            }
        }
    }

    // PN-S14: RETURN_VALIDAR_TIPO
    fn validar_tipo_regresa(&mut self, valor: &Expresion, ambito: &str, tipo_retorno_func: &TipoDato) {
        self.validar_regresa(valor, ambito, tipo_retorno_func);
    }

    fn validar_regresa(&mut self, valor: &Expresion, ambito: &str, tipo_retorno_func: &TipoDato) {
        if *tipo_retorno_func == TipoDato::Nula {
            self.errores.push(ErrorSemantico::RegresaEnFuncionNula {
                funcion: ambito.to_string(),
            });
            return;
        }

        match self.inferir_expresion(valor, ambito) {
            Ok(tipo_expr) => {
                if self
                    .cubo
                    .consultar(tipo_retorno_func, &tipo_expr, &Operador::Asigna)
                    .is_err()
                {
                    self.errores.push(ErrorSemantico::RetornoTipoIncompatible {
                        funcion: ambito.to_string(),
                        esperado: tipo_retorno_func.to_string(),
                        recibido: tipo_expr.to_string(),
                    });
                }
            }
            Err(e) => self.errores.push(e),
        }
    }

    fn validar_retorno_funcion(&mut self, func: &Funcion) {
        if self.convertir_tipo_func(&func.tipo_retorno) == TipoDato::Nula {
            return;
        }

        // Esta regla exige al menos un `regresa` en cualquier punto del cuerpo.
        // No hace análisis de alcanzabilidad; busca claridad antes que complejidad.
        if !self.contiene_regresa(&func.cuerpo) {
            self.errores.push(ErrorSemantico::FaltaRegresa {
                funcion: func.nombre.clone(),
                esperado: self.convertir_tipo_func(&func.tipo_retorno).to_string(),
            });
        }
    }

    fn contiene_regresa(&self, stmts: &[Estatuto]) -> bool {
        for stmt in stmts {
            match stmt {
                Estatuto::Regresa { .. } => return true,
                Estatuto::Condicion { entonces, sino, .. } => {
                    if self.contiene_regresa(entonces) {
                        return true;
                    }
                    if let Some(sino_body) = sino {
                        if self.contiene_regresa(sino_body) {
                            return true;
                        }
                    }
                }
                Estatuto::Ciclo { cuerpo, .. } | Estatuto::Bloque(cuerpo) => {
                    if self.contiene_regresa(cuerpo) {
                        return true;
                    }
                }
                _ => {}
            }
        }
        false
    }

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
                // PN-S08: Validar operación relacional
                self.validar_operacion_relacional(op_rel, &tipo_izq, &tipo_der)
            }
        }
    }

    fn inferir_exp(&mut self, exp: &Exp, ambito: &str) -> Result<TipoDato, ErrorSemantico> {
        let mut tipo_acc = self.inferir_termino(&exp.termino, ambito)?;
        for (op_arit, term) in &exp.cont {
            let tipo_der = self.inferir_termino(term, ambito)?;
            // PN-S07: Validar operación aritmética
            tipo_acc = self.validar_operacion_aritmetica(op_arit, &tipo_acc, &tipo_der)?;
        }
        Ok(tipo_acc)
    }

    fn inferir_termino(
        &mut self,
        term: &Termino,
        ambito: &str,
    ) -> Result<TipoDato, ErrorSemantico> {
        let mut tipo_acc = self.inferir_factor(&term.factor, ambito)?;
        for (op_mul, fac) in &term.cont {
            let tipo_der = self.inferir_factor(fac, ambito)?;
            // PN-S07: Validar operación aritmética
            tipo_acc = self.validar_operacion_mul_div(op_mul, &tipo_acc, &tipo_der)?;
        }
        Ok(tipo_acc)
    }

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

            // PN-S06: IDENT_RESOLVER_TIPO
            Factor::Id(id) | Factor::PosId(id) | Factor::NegId(id) => {
                self.resolver_identificador(ambito, id)
            }

            Factor::Paren(expr) => self.inferir_expresion(expr, ambito),

            // PN-S10 / PN-S11 / PN-S12: llamada en expresión
            Factor::Llamada(ll) => {
                let tipo = self.inferir_llamada(ll, ambito)?;
                if tipo == TipoDato::Nula {
                    return Err(ErrorSemantico::UsoFuncionNulaEnExpresion {
                        funcion: ll.nombre.clone(),
                    });
                }
                Ok(tipo)
            }
        }
    }

    fn inferir_llamada(
        &mut self,
        ll: &Llamada,
        ambito: &str,
    ) -> Result<TipoDato, ErrorSemantico> {
        // escribe acepta cualquier lista de argumentos.
        if ll.nombre == "escribe" {
            for arg in &ll.args {
                let _ = self.inferir_expresion(arg, ambito);
            }
            return Ok(TipoDato::Nula);
        }

        // PN-S10: CALL_VALIDAR_EXISTENCIA
        let entrada = self.validar_existencia_llamada(&ll.nombre)?.clone();
        // PN-S11: CALL_VALIDAR_ARIDAD
        self.validar_aridad_llamada(ll, &entrada);
        // PN-S12: CALL_VALIDAR_TIPOS_ARGUMENTOS
        self.validar_tipos_argumentos(ll, ambito, &entrada);

        Ok(entrada.tipo_retorno.clone())
    }

    // PN-S10: CALL_VALIDAR_EXISTENCIA
    fn validar_existencia_llamada(&self, nombre: &str) -> Result<&EntradaFuncion, ErrorSemantico> {
        self.directorio.buscar_funcion(nombre)
    }

    // PN-S11: CALL_VALIDAR_ARIDAD
    fn validar_aridad_llamada(&mut self, ll: &Llamada, entrada: &EntradaFuncion) {
        if ll.args.len() != entrada.num_params {
            self.errores.push(ErrorSemantico::ArityMismatch {
                funcion: ll.nombre.clone(),
                esperados: entrada.num_params,
                recibidos: ll.args.len(),
            });
        }
    }

    // PN-S12: CALL_VALIDAR_TIPOS_ARGUMENTOS
    fn validar_tipos_argumentos(
        &mut self,
        ll: &Llamada,
        ambito: &str,
        entrada: &EntradaFuncion,
    ) {
        for (i, (arg, tipo_param)) in ll.args.iter().zip(entrada.tipos_params.iter()).enumerate() {
            match self.inferir_expresion(arg, ambito) {
                Ok(tipo_arg) => {
                    if self
                        .cubo
                        .consultar(tipo_param, &tipo_arg, &Operador::Asigna)
                        .is_err()
                    {
                        self.errores.push(ErrorSemantico::TipoIncompatible {
                            op: format!("arg {} de '{}'", i + 1, ll.nombre),
                            izq: tipo_param.to_string(),
                            der: tipo_arg.to_string(),
                        });
                    }
                }
                Err(e) => self.errores.push(e),
            }
        }
    }

    // PN-S15: COND_VALIDAR_TIPO_ENTERO
    fn verificar_cond_tipo(&mut self, cond: &Expresion, ambito: &str) {
        match self.inferir_expresion(cond, ambito) {
            Ok(TipoDato::Entero) => {}
            Ok(t) => self.errores.push(ErrorSemantico::CondicionNoBooleana {
                contexto: ambito.to_string(),
                tipo: t.to_string(),
            }),
            Err(e) => self.errores.push(e),
        }
    }

    // PN-S06: IDENT_RESOLVER_TIPO
    fn resolver_identificador(
        &self,
        ambito: &str,
        id: &str,
    ) -> Result<TipoDato, ErrorSemantico> {
        self.directorio.resolver_variable(ambito, id)
    }

    // PN-S07: EXPR_VALIDAR_ARITMETICA (+/-)
    fn validar_operacion_aritmetica(
        &self,
        op_arit: &OpArit,
        tipo_izq: &TipoDato,
        tipo_der: &TipoDato,
    ) -> Result<TipoDato, ErrorSemantico> {
        let op = match op_arit {
            OpArit::Plus => Operador::Suma,
            OpArit::Minus => Operador::Resta,
        };
        self.cubo.consultar(tipo_izq, tipo_der, &op)
            .map_err(|_| ErrorSemantico::TipoIncompatible {
                op:  format!("{:?}", op_arit),
                izq: tipo_izq.to_string(),
                der: tipo_der.to_string(),
            })
    }

    // PN-S07: EXPR_VALIDAR_ARITMETICA (*,/)
    fn validar_operacion_mul_div(
        &self,
        op_mul: &OpMul,
        tipo_izq: &TipoDato,
        tipo_der: &TipoDato,
    ) -> Result<TipoDato, ErrorSemantico> {
        let op = match op_mul {
            OpMul::Star => Operador::Mul,
            OpMul::Slash => Operador::Div,
        };
        self.cubo.consultar(tipo_izq, tipo_der, &op)
            .map_err(|_| ErrorSemantico::TipoIncompatible {
                op:  format!("{:?}", op_mul),
                izq: tipo_izq.to_string(),
                der: tipo_der.to_string(),
            })
    }

    // PN-S08: EXPR_VALIDAR_RELACIONAL
    fn validar_operacion_relacional(
        &self,
        op_rel: &OpRel,
        tipo_izq: &TipoDato,
        tipo_der: &TipoDato,
    ) -> Result<TipoDato, ErrorSemantico> {
        let op = match op_rel {
            OpRel::Gt => Operador::Mayor,
            OpRel::Lt => Operador::Menor,
            OpRel::EqEq => Operador::Igual,
            OpRel::Neq => Operador::Diferente,
        };
        self.cubo.consultar(tipo_izq, tipo_der, &op)
            .map_err(|_msg| ErrorSemantico::TipoIncompatible {
                op:  format!("{:?}", op_rel),
                izq: tipo_izq.to_string(),
                der: tipo_der.to_string(),
            })
    }

    // PN-S09: ASIGNA_VALIDAR_COMPATIBILIDAD
    fn validar_asignacion(&mut self, id: &str, expr: &Expresion, ambito: &str) {
        let tipo_id = match self.directorio.resolver_variable(ambito, id) {
            Ok(t) => t,
            Err(e) => {
                self.errores.push(e);
                return;
            }
        };

        let tipo_expr = match self.inferir_expresion(expr, ambito) {
            Ok(t) => t,
            Err(e) => {
                self.errores.push(e);
                return;
            }
        };

        if let Err(_) = self.cubo.consultar(&tipo_id, &tipo_expr, &Operador::Asigna) {
            self.errores.push(ErrorSemantico::AsignacionTipoIncompatible {
                var:       id.to_string(),
                var_tipo:  tipo_id.to_string(),
                expr_tipo: tipo_expr.to_string(),
            });
        }
    }

    // PN-S02: TIPO_CONVERTIR_SINTACTICO
    fn convertir_tipo(&self, tipo: &Tipo) -> TipoDato {
        TipoDato::from_tipo(tipo)
    }

    // PN-S02: TIPO_CONVERTIR_SINTACTICO (retorno)
    fn convertir_tipo_func(&self, tipo: &TipoFunc) -> TipoDato {
        TipoDato::from_tipo_func(tipo)
    }

    // True si hubo errores semanticos.
    pub fn tiene_errores(&self) -> bool { !self.errores.is_empty() }

    // Reporte para CLI y pruebas.
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
