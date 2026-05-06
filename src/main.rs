// src/main.rs

use compilador::parse;

fn main() {
    let src = r#"
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

    match parse(src) {
        Ok(ast)  => println!("Parsed OK:\n{:#?}", ast),
        Err(err) => println!("Error: {}", err),
    }
}
