// src/main.rs

use compilador::analizador_semantico::AnalizadorSemantico;
use compilador::parse;

const SAMPLE_SRC: &str = r#"
programa calculadora;

vars resultado, temp : flotante;
vars opcion, i, n : entero;

entero esPar(num : entero) {
    vars resto : entero;
    {
        resto = num - num;
        si (resto == 0) {
            resto = 1;
        } sino {
            resto = 0;
        };
    }
};

flotante calcularPromedio(suma : flotante, cantidad : entero) {
    vars prom : flotante;
    {
        prom = suma / cantidad;
    }
};

nula imprimirLinea() {
    {
        escribe("----------------------------");
    }
};

nula contarHasta(limite : entero) {
    vars contador : entero;
    {
        contador = 1;
        mientras (contador < limite) haz {
            escribe("contando: ", contador);
            contador = contador + 1;
        };
    }
};

inicio {
    escribe("== Calculadora ==");
    imprimirLinea();

    resultado = 0.0;
    n = 5;
    i = 1;

    mientras (i < n) haz {
        temp = i + 0.5;
        resultado = resultado + temp;
        escribe("acumulado: ", resultado);
        i = i + 1;
    };

    imprimirLinea();
    escribe("suma final: ", resultado);

    resultado = calcularPromedio(resultado, n);
    escribe("promedio: ", resultado);

    imprimirLinea();
    contarHasta(4);

    si (resultado > 2.0) {
        escribe("promedio alto");
    } sino {
        escribe("promedio bajo");
    };

    imprimirLinea();
    escribe("fin del programa");
}
fin
"#;

fn compilar_y_analizar(src: &str, mostrar_ast: bool) -> Result<(), String> {
    let ast = parse(src)?;
    if mostrar_ast {
        println!("Parsed OK:\n{:#?}", ast);
    }

    let mut sem = AnalizadorSemantico::new();
    sem.analizar(&ast);
    if sem.tiene_errores() {
        return Err(sem.reporte());
    }

    Ok(())
}

fn main() {
    let mut args = std::env::args().skip(1);
    let path = args.next();
    let show_ast = args.next().as_deref() == Some("--ast");

    let (src, mostrar_ast) = match path {
        Some(path) => {
            let src = std::fs::read_to_string(&path)
                .unwrap_or_else(|_| panic!("No se pudo leer '{}'", path));
            (src, show_ast)
        }
        None => (SAMPLE_SRC.to_string(), true),
    };

    match compilar_y_analizar(&src, mostrar_ast) {
        Ok(()) => {}
        Err(msg) => eprintln!("{}", msg),
    }
}
