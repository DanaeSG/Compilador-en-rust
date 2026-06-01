// Estructuras semanticas compartidas.
// Incluye cubo semantico, tabla de variables y directorio de funciones.

use std::collections::HashMap;
use crate::ast::{Tipo, TipoFunc};
// Direccion aun no asignada.
pub const DIR_SIN_ASIGNAR: i32 = -1;

// Tipo de dato en compilacion.
// Nula se usa para funciones void.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TipoDato {
    Entero,
    Flotante,
    Nula,
}

impl TipoDato {
    pub fn from_tipo(t: &Tipo) -> Self {
        match t {
            Tipo::Entero   => TipoDato::Entero,
            Tipo::Flotante => TipoDato::Flotante,
        }
    }
    pub fn from_tipo_func(t: &TipoFunc) -> Self {
        match t {
            TipoFunc::Nula    => TipoDato::Nula,
            TipoFunc::Tipo(t) => TipoDato::from_tipo(t),
        }
    }
}

impl std::fmt::Display for TipoDato {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TipoDato::Entero   => write!(f, "entero"),
            TipoDato::Flotante => write!(f, "flotante"),
            TipoDato::Nula     => write!(f, "nula"),
        }
    }
}

// Operadores del lenguaje.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Operador {
    Suma, Resta, Mul, Div,          // aritméticos
    Mayor, Menor, Igual, Diferente, // relacionales
    Asigna,                         // asignación
}

// Cubo semantico en tabla dispersa.

#[derive(Debug, Clone)]
pub struct CuboSemantico {
    tabla: HashMap<(TipoDato, TipoDato, Operador), TipoDato>,
}

impl CuboSemantico {
    pub fn new() -> Self {
        use Operador::*;
        use TipoDato::*;

        let mut tabla = HashMap::new();

        //  Operadores aritméticos 
        for op in &[Suma, Resta, Mul, Div] {
            tabla.insert((Entero,   Entero,   op.clone()), Entero);
            tabla.insert((Entero,   Flotante, op.clone()), Flotante);
            tabla.insert((Flotante, Entero,   op.clone()), Flotante);
            tabla.insert((Flotante, Flotante, op.clone()), Flotante);
        }

        //  Operadores relacionales 
        for op in &[Mayor, Menor, Igual, Diferente] {
            tabla.insert((Entero,   Entero,   op.clone()), Entero);
            tabla.insert((Entero,   Flotante, op.clone()), Entero);
            tabla.insert((Flotante, Entero,   op.clone()), Entero);
            tabla.insert((Flotante, Flotante, op.clone()), Entero);
        }

        //  Asignación 
        tabla.insert((Entero,   Entero,   Asigna), Entero);
        tabla.insert((Flotante, Entero,   Asigna), Flotante); // promoción ok
        tabla.insert((Flotante, Flotante, Asigna), Flotante);
        // (Entero, Flotante, Asigna) -> ausente = error

        Self { tabla }
    }

    // Consulta el cubo.
    pub fn consultar(
        &self,
        op_izq: &TipoDato,
        op_der: &TipoDato,
        op: &Operador,
    ) -> Result<TipoDato, String> {
        let key = (op_izq.clone(), op_der.clone(), op.clone());
        self.tabla.get(&key).cloned().ok_or_else(|| {
            format!(
                "Operación semántica inválida: {:?} entre '{}' y '{}'",
                op, op_izq, op_der
            )
        })
    }
}

// Entrada de variable en tabla de simbolos.

#[derive(Debug, Clone)]
pub struct EntradaVariable {
    pub tipo:   TipoDato,
    // True si viene de parametros.
    pub es_param: bool,
    pub dir_virtual: i32,
}

// Tabla de variables por ambito.
#[derive(Debug, Clone)]
pub struct TablaVariables {
    pub variables: HashMap<String, EntradaVariable>, 
}

impl TablaVariables {
    pub fn new() -> Self {
        Self { variables: HashMap::new() }
    }

    // Declara variable.
    pub fn declarar(
        &mut self,
        nombre: &str,
        tipo: TipoDato,
        es_param: bool,
        dir_virtual: i32,
    ) -> Result<(), ErrorSemantico> {
        if self.variables.contains_key(nombre) {
            return Err(ErrorSemantico::VariableDoblementeDeclada(nombre.to_string()));
        }
        self.variables.insert(
            nombre.to_string(),
            EntradaVariable { tipo, es_param, dir_virtual },
        );
        Ok(())
    }

    // Busca variable.
    pub fn buscar(&self, nombre: &str) -> Result<&EntradaVariable, ErrorSemantico> {
        self.variables.get(nombre)
            .ok_or_else(|| ErrorSemantico::VariableNoDeclarada(nombre.to_string()))
    }

    pub fn asignar_direccion(&mut self, nombre: &str, dir_virtual: i32) -> Result<(), ErrorSemantico> {
        let entrada = self.variables.get_mut(nombre)
            .ok_or_else(|| ErrorSemantico::VariableNoDeclarada(nombre.to_string()))?;
        entrada.dir_virtual = dir_virtual;
        Ok(())
    }
}

// Registro de funcion en el directorio.

#[derive(Debug, Clone)]
pub struct EntradaFuncion {
    pub tipo_retorno:   TipoDato,
    pub num_params:     usize,
    // Primer cuadruplo ejecutable de la funcion.
    pub dir_inicio:     usize,
    pub tipos_params:   Vec<TipoDato>,   // orden de los parámetros
    pub nombres_params: Vec<String>,     // nombres en orden para PARAM
    pub tabla_vars:     TablaVariables,
}

#[derive(Debug)]
#[derive(Clone)]
pub struct DirectorioFunciones {
    pub funciones:  HashMap<String, EntradaFuncion>,
    // Nombre del programa y clave del ambito global.
    pub nombre_prog: String,
}

impl DirectorioFunciones {
    pub fn new(nombre_prog: &str) -> Self {
        let mut dir = Self {
            funciones: HashMap::new(),
            nombre_prog: nombre_prog.to_string(),
        };
        // Registra el ambito global desde el inicio.
        dir.funciones.insert(
            nombre_prog.to_string(),
            EntradaFuncion {
                tipo_retorno: TipoDato::Nula,
                num_params: 0,
                // En global no se usa GOSUB.
                dir_inicio: 0,
                tipos_params: vec![],
                nombres_params: vec![],
                tabla_vars: TablaVariables::new(),
            },
        );
        dir
    }

    // Registra funcion.
    pub fn registrar_funcion(
        &mut self,
        nombre: &str,
        tipo_retorno: TipoDato,
        params: Vec<(String, TipoDato)>,
    ) -> Result<(), ErrorSemantico> {
        if self.funciones.contains_key(nombre) {
            return Err(ErrorSemantico::FuncionDoblementeDeclada(nombre.to_string()));
        }
        let num_params = params.len();
        let tipos_params = params.iter().map(|(_, t)| t.clone()).collect();
        let nombres_params = params.iter().map(|(id, _)| id.clone()).collect();
        let mut tabla    = TablaVariables::new();
        // Los parámetros van a la tabla de variables local marcados como param
        for (id, tipo) in params {
            tabla.declarar(&id, tipo, true, DIR_SIN_ASIGNAR)?;
        }
        if tipo_retorno != TipoDato::Nula {
            // Guarda retorno tipado como variable global homonima.
            let global = self.funciones.get_mut(&self.nombre_prog).unwrap();
            global.tabla_vars.declarar(nombre, tipo_retorno.clone(), false, DIR_SIN_ASIGNAR)?;
        }
        self.funciones.insert(
            nombre.to_string(),
            EntradaFuncion { tipo_retorno, num_params, dir_inicio: 0, tipos_params, nombres_params, tabla_vars: tabla },
        );
        Ok(())
    }

    // Declara variable en ambito.
    pub fn declarar_variable(
        &mut self,
        ambito: &str,
        nombre: &str,
        tipo: TipoDato,
        dir_virtual: i32,
    ) -> Result<(), ErrorSemantico> {
        let entrada = self.funciones.get_mut(ambito)
            .ok_or_else(|| ErrorSemantico::AmbitoNoEncontrado(ambito.to_string()))?;
        entrada.tabla_vars.declarar(nombre, tipo, false, dir_virtual)
    }

    // Busca funcion.
    pub fn buscar_funcion(&self, nombre: &str) -> Result<&EntradaFuncion, ErrorSemantico> {
        self.funciones.get(nombre)
            .ok_or_else(|| ErrorSemantico::FuncionNoDeclarada(nombre.to_string()))
    }

    // Resuelve variable en local y luego global.
    pub fn resolver_variable(
        &self,
        ambito_local: &str,
        nombre: &str,
    ) -> Result<TipoDato, ErrorSemantico> {
        // 1. buscar en ámbito local
        if let Ok(ef) = self.buscar_funcion(ambito_local) {
            if let Ok(ev) = ef.tabla_vars.buscar(nombre) {
                return Ok(ev.tipo.clone());
            }
        }
        // 2. buscar en ámbito global
        if ambito_local != self.nombre_prog {
            let global = self.funciones.get(&self.nombre_prog).unwrap();
            if let Ok(ev) = global.tabla_vars.buscar(nombre) {
                return Ok(ev.tipo.clone());
            }
        }
        Err(ErrorSemantico::VariableNoDeclarada(nombre.to_string()))
    }

    pub fn resolver_dir_variable(
        &self,
        ambito_local: &str,
        nombre: &str,
    ) -> Result<i32, ErrorSemantico> {
        if let Ok(ef) = self.buscar_funcion(ambito_local) {
            if let Ok(ev) = ef.tabla_vars.buscar(nombre) {
                return Ok(ev.dir_virtual);
            }
        }
        if ambito_local != self.nombre_prog {
            let global = self.funciones.get(&self.nombre_prog).unwrap();
            if let Ok(ev) = global.tabla_vars.buscar(nombre) {
                return Ok(ev.dir_virtual);
            }
        }
        Err(ErrorSemantico::VariableNoDeclarada(nombre.to_string()))
    }

    pub fn asignar_dir_variable(
        &mut self,
        ambito: &str,
        nombre: &str,
        dir_virtual: i32,
    ) -> Result<(), ErrorSemantico> {
        let entrada = self.funciones.get_mut(ambito)
            .ok_or_else(|| ErrorSemantico::AmbitoNoEncontrado(ambito.to_string()))?;
        entrada.tabla_vars.asignar_direccion(nombre, dir_virtual)
    }

    pub fn asignar_dir_inicio_funcion(
        &mut self,
        nombre: &str,
        dir_inicio: usize,
    ) -> Result<(), ErrorSemantico> {
        let entrada = self.funciones.get_mut(nombre)
            .ok_or_else(|| ErrorSemantico::FuncionNoDeclarada(nombre.to_string()))?;
        entrada.dir_inicio = dir_inicio;
        Ok(())
    }
}

// ------------------------------------------------------------------------------
//  ERRORES SEMÁNTICOS
// ------------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub enum ErrorSemantico {
    VariableDoblementeDeclada(String),
    VariableNoDeclarada(String),
    FuncionDoblementeDeclada(String),
    FuncionNoDeclarada(String),
    AmbitoNoEncontrado(String),
    TipoIncompatible { op: String, izq: String, der: String },
    ArityMismatch { funcion: String, esperados: usize, recibidos: usize },
    AsignacionTipoIncompatible { var: String, var_tipo: String, expr_tipo: String },
    RegresaEnFuncionNula { funcion: String },
    RegresaFueraDeFuncion,
    RetornoTipoIncompatible { funcion: String, esperado: String, recibido: String },
    FaltaRegresa { funcion: String, esperado: String },
    UsoFuncionNulaEnExpresion { funcion: String },
    CondicionNoBooleana { contexto: String, tipo: String },
    FuncionSinDirInicio { funcion: String },
    EndDentroDeFuncion { funcion: String },
    EndfuncEnPrincipal,
}

impl std::fmt::Display for ErrorSemantico {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::VariableDoblementeDeclada(n) =>
                write!(f, "Variable doblemente declarada: '{}'", n),
            Self::VariableNoDeclarada(n) =>
                write!(f, "Variable no declarada: '{}'", n),
            Self::FuncionDoblementeDeclada(n) =>
                write!(f, "Función doblemente declarada: '{}'", n),
            Self::FuncionNoDeclarada(n) =>
                write!(f, "Función no declarada: '{}'", n),
            Self::AmbitoNoEncontrado(n) =>
                write!(f, "Ámbito no encontrado: '{}'", n),
            Self::TipoIncompatible { op, izq, der } =>
                write!(f, "Tipo incompatible: {} {} {}", izq, op, der),
            Self::ArityMismatch { funcion, esperados, recibidos } =>
                write!(f, "Función '{}': se esperaban {} args, se recibieron {}", funcion, esperados, recibidos),
            Self::AsignacionTipoIncompatible { var, var_tipo, expr_tipo } =>
                write!(f, "No se puede asignar '{}' a variable '{}' de tipo '{}'", expr_tipo, var, var_tipo),
            Self::RegresaEnFuncionNula { funcion } =>
                write!(f, "La función nula '{}' no debe tener 'regresa'", funcion),
            Self::RegresaFueraDeFuncion =>
                write!(f, "Uso de 'regresa' fuera de una función"),
            Self::RetornoTipoIncompatible { funcion, esperado, recibido } =>
                write!(f, "Retorno incompatible en '{}': se esperaba '{}' y se recibió '{}'", funcion, esperado, recibido),
            Self::FaltaRegresa { funcion, esperado } =>
                write!(f, "La función '{}' debe tener al menos un 'regresa' de tipo '{}'", funcion, esperado),
            Self::UsoFuncionNulaEnExpresion { funcion } =>
                write!(f, "La función nula '{}' no puede usarse dentro de una expresión", funcion),
            Self::CondicionNoBooleana { contexto, tipo } =>
                write!(f, "Condición no booleana en '{}': se recibió '{}'", contexto, tipo),
            Self::FuncionSinDirInicio { funcion } =>
                write!(f, "La función '{}' no tiene dir_inicio asignada", funcion),
            Self::EndDentroDeFuncion { funcion } =>
                write!(f, "END no puede generarse dentro de la función '{}'", funcion),
            Self::EndfuncEnPrincipal =>
                write!(f, "ENDFUNC no puede generarse en el programa principal"),
        }
    }
}
