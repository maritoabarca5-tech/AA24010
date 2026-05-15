// ============================================================================
// MOTOR DE TRÁFICO AÉREO — Aeropuerto Internacional de Santa Ana
// Estructura de Datos: Árbol AVL organizado por altitud (pies)
// ============================================================================
//
// ─── FASE 1: ANÁLISIS DE SEGURIDAD Y PROPIEDAD ─────────────────────────────
//
// 1) DOCUMENTACIÓN DE MEMORIA — Option::take() en rotaciones
//
//    En Rust, cada valor tiene exactamente UN dueño (ownership). Cuando un
//    nodo del árbol posee a sus hijos mediante Option<Box<Nodo>>, no podemos
//    simplemente "copiar" un hijo para moverlo a otro lugar, ya que eso
//    violaría la regla de un solo dueño.
//
//    Option::take() resuelve esto de forma segura:
//      - EXTRAE el valor del Option (transfiere el ownership al llamador).
//      - DEJA un None en la posición original, evitando referencias colgantes.
//
//    Ejemplo en rotar_derecha():
//      let mut x = y.izquierdo.take().unwrap();
//      //  → y.izquierdo ahora es None (ya no es dueño de x)
//      //  → x es el nuevo dueño del subárbol izquierdo
//      y.izquierdo = x.derecho.take();
//      //  → x.derecho pasa a ser None
//      //  → y.izquierdo toma ownership del antiguo hijo derecho de x
//      x.derecho = Some(y);
//      //  → x toma ownership de y como su hijo derecho
//
//    Sin take(), Rust no nos permitiría mover los nodos porque detectaría
//    que estamos intentando usar un valor que ya fue movido (error E0382).
//
// 2) CONCEPTO DE Box<Nodo>
//
//    En Rust, el tamaño de cada tipo debe ser conocido en tiempo de
//    compilación. Si definimos:
//      struct Nodo { izquierdo: Option<Nodo>, ... }
//    tendríamos un tipo recursivo de tamaño INFINITO: un Nodo contiene
//    otro Nodo, que contiene otro Nodo, ad infinitum.
//
//    Box<T> es un puntero inteligente que:
//      - Almacena el dato T en el HEAP (memoria dinámica).
//      - El Box en sí ocupa solo 8 bytes (un puntero) en el STACK.
//      - Esto da un tamaño FINITO y conocido en compilación.
//      - Cuando el Box se destruye (sale de ámbito), libera automáticamente
//        la memoria del heap (RAII), sin necesidad de free() manual.
//
//    Así, Option<Box<Nodo>> ocupa exactamente 8 bytes (puntero al heap)
//    o 0 lógico cuando es None, permitiendo la recursión segura.
//
// 3) PRUEBA DE ESCRITORIO — Ver archivo: prueba_de_escritorio.md
//    (Incluye dibujos paso a paso con rotaciones identificadas)
//
// ============================================================================

#[derive(Debug, Clone)]
struct Vuelo {
    id: String,
    altitud: u32, // Clave del árbol AVL (pies de altitud)
}

struct Nodo {
    vuelo: Vuelo,
    izquierdo: Option<Box<Nodo>>, // Subárbol izquierdo (altitudes menores)
    derecho: Option<Box<Nodo>>,   // Subárbol derecho (altitudes mayores)
    altura: i32,                  // Altura del nodo para cálculo de balance
}

impl Nodo {
    /// Crea un nodo hoja con altura 1 y sin hijos.
    /// Recibe ownership del Vuelo (se mueve, no se copia).
    fn nuevo(vuelo: Vuelo) -> Self {
        Nodo {
            vuelo,
            izquierdo: None,
            derecho: None,
            altura: 1,
        }
    }
}

// =================== UTILIDADES DE BALANCEO (NO MODIFICAR) ===================

/// Retorna la altura de un nodo. Si es None, retorna 0.
/// Usa as_ref() para obtener una referencia sin tomar ownership.
fn obtener_altura(nodo: &Option<Box<Nodo>>) -> i32 {
    // as_ref() convierte &Option<Box<Nodo>> en Option<&Box<Nodo>>
    // permitiendo leer el valor sin consumir el Option.
    nodo.as_ref().map_or(0, |n| n.altura)
}

/// Recalcula la altura de un nodo basándose en sus hijos.
/// Recibe &mut Nodo: referencia mutable exclusiva (Rust garantiza
/// que nadie más puede leer/escribir este nodo simultáneamente).
fn actualizar_altura(nodo: &mut Nodo) {
    nodo.altura = 1
        + std::cmp::max(
            obtener_altura(&nodo.izquierdo),
            obtener_altura(&nodo.derecho),
        );
}

/// Factor de balance = altura_izquierda - altura_derecha.
/// Si > 1: desbalanceado a la izquierda.
/// Si < -1: desbalanceado a la derecha.
fn obtener_balance(nodo: &Nodo) -> i32 {
    obtener_altura(&nodo.izquierdo) - obtener_altura(&nodo.derecho)
}

/// Rotación simple a la derecha (caso Left-Left).
/// take() transfiere ownership de los hijos para reconectar punteros.
///
///       y                x
///      / \             /   \
///     x   T3   →     T1    y
///    / \                  / \
///   T1  T2              T2  T3
fn rotar_derecha(mut y: Box<Nodo>) -> Box<Nodo> {
    // take() mueve ownership del hijo izquierdo de y → variable x.
    // y.izquierdo queda como None (seguridad: sin referencia colgante).
    let mut x = y.izquierdo.take().unwrap();

    // El hijo derecho de x pasa a ser el hijo izquierdo de y.
    // take() mueve ownership de x.derecho → y.izquierdo.
    y.izquierdo = x.derecho.take();

    // Primero actualizamos y (ahora es un nodo más bajo).
    actualizar_altura(&mut y);

    // y pasa a ser hijo derecho de x (x toma ownership de y).
    x.derecho = Some(y);

    // Actualizamos x (nueva raíz del subárbol).
    actualizar_altura(&mut x);

    x // Retornamos x como nueva raíz (transferimos ownership al llamador)
}

/// Rotación simple a la izquierda (caso Right-Right).
/// Misma lógica de take() pero en dirección opuesta.
///
///     x                  y
///    / \               /   \
///   T1  y      →      x    T3
///      / \           / \
///     T2  T3        T1  T2
fn rotar_izquierda(mut x: Box<Nodo>) -> Box<Nodo> {
    // take() transfiere ownership del hijo derecho de x → variable y.
    let mut y = x.derecho.take().unwrap();

    // El hijo izquierdo de y pasa a ser hijo derecho de x.
    x.derecho = y.izquierdo.take();

    actualizar_altura(&mut x);

    // x pasa a ser hijo izquierdo de y.
    y.izquierdo = Some(x);

    actualizar_altura(&mut y);

    y // Nueva raíz del subárbol
}

// ========================= FASE 2: INSERTAR ==================================

/// Inserta un vuelo en el árbol AVL manteniendo el balance.
///
/// Recibe ownership del subárbol (Option<Box<Nodo>>) y del vuelo.
/// Retorna el subárbol modificado (transferencia de ownership de vuelta).
///
/// Nota: Se guarda `vuelo.altitud` en `altitud_vuelo` ANTES de mover
/// `vuelo` a la llamada recursiva, porque una vez que `vuelo` se mueve
/// (ownership transferido), ya no podemos acceder a sus campos.
/// Esto evita usar .clone() innecesario.
fn insertar(nodo_opt: Option<Box<Nodo>>, vuelo: Vuelo) -> Box<Nodo> {

    let mut nodo = match nodo_opt {
        // Si el subárbol está vacío, creamos un nodo hoja.
        // El ownership de `vuelo` se mueve a Nodo::nuevo().
        None => return Box::new(Nodo::nuevo(vuelo)),
        Some(n) => n,
    };

    // Guardamos la altitud ANTES de mover vuelo (u32 implementa Copy,
    // así que se copia el valor numérico sin problemas).
    let altitud_vuelo = vuelo.altitud;

    if altitud_vuelo < nodo.vuelo.altitud {
        // take() extrae el subárbol izquierdo, lo pasa a la recursión,
        // y el resultado se reasigna como nuevo hijo izquierdo.
        nodo.izquierdo =
            Some(insertar(nodo.izquierdo.take(), vuelo));

    } else if altitud_vuelo > nodo.vuelo.altitud {
        nodo.derecho =
            Some(insertar(nodo.derecho.take(), vuelo));

    } else {
        // Altitud duplicada: no insertamos (cada altitud es única en el radar).
        return nodo;
    }

    // Recalcular altura y balance tras la inserción
    actualizar_altura(&mut nodo);
    let balance = obtener_balance(&nodo);

    // Caso Izquierda-Izquierda: Rotación Simple a la Derecha
    if balance > 1
        && altitud_vuelo
            < nodo.izquierdo.as_ref().unwrap().vuelo.altitud
    {
        return rotar_derecha(nodo);
    }

    // Caso Derecha-Derecha: Rotación Simple a la Izquierda
    if balance < -1
        && altitud_vuelo
            > nodo.derecho.as_ref().unwrap().vuelo.altitud
    {
        return rotar_izquierda(nodo);
    }

    // Caso Izquierda-Derecha: Rotación Doble (izq luego der)
    if balance > 1
        && altitud_vuelo
            > nodo.izquierdo.as_ref().unwrap().vuelo.altitud
    {
        let hijo_izq = nodo.izquierdo.take().unwrap();
        nodo.izquierdo = Some(rotar_izquierda(hijo_izq));
        return rotar_derecha(nodo);
    }

    // Caso Derecha-Izquierda: Rotación Doble (der luego izq)
    if balance < -1
        && altitud_vuelo
            < nodo.derecho.as_ref().unwrap().vuelo.altitud
    {
        let hijo_der = nodo.derecho.take().unwrap();
        nodo.derecho = Some(rotar_derecha(hijo_der));
        return rotar_izquierda(nodo);
    }

    nodo
}

// ======================== FASE 2: BUSCAR VUELO ===============================

/// Busca un vuelo por altitud en O(log n) usando SOLO referencias.
///
/// Firma requerida: fn buscar_vuelo(nodo: &Option<Box<Nodo>>, altitud: u32) -> Option<&Vuelo>
///
/// - Recibe &Option<Box<Nodo>>: referencia inmutable (solo lectura).
///   El radar NO puede modificar accidentalmente los datos del vuelo.
/// - Retorna Option<&Vuelo>: referencia al vuelo encontrado, sin copiar datos.
/// - El lifetime 'a garantiza que la referencia retornada vive tanto
///   como el árbol original, evitando referencias colgantes.
/// - Complejidad O(log n): en cada paso descartamos la mitad del árbol.
fn buscar_vuelo<'a>(
    nodo: &'a Option<Box<Nodo>>,
    altitud: u32,
) -> Option<&'a Vuelo> {

    match nodo {
        // Árbol vacío o llegamos a una hoja sin encontrar
        None => None,

        Some(n) => {
            if altitud == n.vuelo.altitud {
                // Encontrado: retornamos referencia al vuelo (sin copiar)
                Some(&n.vuelo)

            } else if altitud < n.vuelo.altitud {
                // Buscar en subárbol izquierdo (altitudes menores)
                buscar_vuelo(&n.izquierdo, altitud)

            } else {
                // Buscar en subárbol derecho (altitudes mayores)
                buscar_vuelo(&n.derecho, altitud)
            }
        }
    }
}

// ================= AUXILIAR PARA ELIMINACIÓN =================================

/// Encuentra el nodo con la MAYOR altitud en un subárbol (predecesor in-order).
/// Recorre hacia la derecha hasta encontrar el nodo más a la derecha.
///
/// Retorna un Vuelo clonado. Este es el ÚNICO clone necesario en todo el
/// programa: necesitamos copiar los datos del predecesor para reemplazar
/// al nodo eliminado, ya que no podemos mover el vuelo de un nodo que
/// aún está dentro del árbol.
fn obtener_maximo(nodo: &Box<Nodo>) -> Vuelo {

    let mut actual = nodo;

    // Avanzar al hijo derecho mientras exista
    while let Some(ref der) = actual.derecho {
        actual = der;
    }

    // Clonamos el vuelo del nodo más a la derecha (predecesor in-order)
    actual.vuelo.clone()
}

// ===================== FASE 3: ELIMINAR VUELO ================================

/// Elimina un vuelo del árbol AVL por su altitud y rebalancea.
///
/// Maneja los 3 casos clásicos de eliminación en BST:
///   Caso 1: Nodo sin hijos (hoja) → simplemente se elimina (return None).
///   Caso 2: Nodo con un hijo → se reemplaza por su único hijo.
///   Caso 3: Nodo con dos hijos → se sustituye por el predecesor in-order
///           (el mayor del subárbol izquierdo), luego se elimina el predecesor.
///
/// Tras la eliminación, se recalculan alturas y se ejecutan las rotaciones
/// necesarias para mantener la propiedad AVL (|balance| <= 1).
///
/// A diferencia de insertar(), aquí usamos obtener_balance() del hijo
/// para decidir el tipo de rotación, ya que no tenemos la altitud del
/// vuelo insertado como referencia.
fn eliminar_vuelo(
    nodo_opt: Option<Box<Nodo>>,
    altitud: u32,
) -> Option<Box<Nodo>> {

    // Si el subárbol está vacío, el vuelo no existe en el radar
    let mut nodo = match nodo_opt {
        None => return None,
        Some(n) => n,
    };

    // ── PASO 1: Buscar el nodo a eliminar ──

    if altitud < nodo.vuelo.altitud {
        // Buscar en subárbol izquierdo
        nodo.izquierdo =
            eliminar_vuelo(nodo.izquierdo.take(), altitud);

    } else if altitud > nodo.vuelo.altitud {
        // Buscar en subárbol derecho
        nodo.derecho =
            eliminar_vuelo(nodo.derecho.take(), altitud);

    } else {
        // ── PASO 2: Nodo encontrado — proceder a eliminar ──

        // CASO 1: Nodo hoja (sin hijos) → eliminarlo retornando None
        // El Box<Nodo> se destruye aquí (Rust libera la memoria del heap).
        if nodo.izquierdo.is_none() && nodo.derecho.is_none() {
            return None;
        }

        // CASO 2: Un solo hijo → reemplazar nodo por su hijo
        // take() transfiere ownership del hijo, el nodo actual se destruye.
        if nodo.izquierdo.is_none() {
            return nodo.derecho; // Solo tiene hijo derecho
        }
        if nodo.derecho.is_none() {
            return nodo.izquierdo; // Solo tiene hijo izquierdo
        }

        // CASO 3: Dos hijos → sustituir por predecesor in-order
        // El predecesor es el valor MÁS ALTO del subárbol izquierdo.
        let predecesor =
            obtener_maximo(nodo.izquierdo.as_ref().unwrap());

        // Guardamos la altitud del predecesor ANTES de mover el Vuelo,
        // ya que después de la asignación `nodo.vuelo = predecesor`,
        // la variable `predecesor` ya no es accesible (ownership movido).
        // Esto evita un .clone() innecesario.
        let alt_predecesor = predecesor.altitud;

        // Reemplazamos los datos del nodo actual con los del predecesor.
        // Move directo (no clone): predecesor ya es un Vuelo owned.
        nodo.vuelo = predecesor;

        // Eliminamos el predecesor original del subárbol izquierdo.
        nodo.izquierdo =
            eliminar_vuelo(nodo.izquierdo.take(), alt_predecesor);
    }

    // ── PASO 3: Recalcular altura y rebalancear ──

    actualizar_altura(&mut nodo);
    let balance = obtener_balance(&nodo);

    // Caso Izquierda-Izquierda: Rotación Simple a la Derecha
    if balance > 1
        && obtener_balance(
            nodo.izquierdo.as_ref().unwrap(),
        ) >= 0
    {
        return Some(rotar_derecha(nodo));
    }

    // Caso Izquierda-Derecha: Rotación Doble
    if balance > 1
        && obtener_balance(
            nodo.izquierdo.as_ref().unwrap(),
        ) < 0
    {
        let hijo_izq = nodo.izquierdo.take().unwrap();
        nodo.izquierdo = Some(rotar_izquierda(hijo_izq));
        return Some(rotar_derecha(nodo));
    }

    // Caso Derecha-Derecha: Rotación Simple a la Izquierda
    if balance < -1
        && obtener_balance(
            nodo.derecho.as_ref().unwrap(),
        ) <= 0
    {
        return Some(rotar_izquierda(nodo));
    }

    // Caso Derecha-Izquierda: Rotación Doble
    if balance < -1
        && obtener_balance(
            nodo.derecho.as_ref().unwrap(),
        ) > 0
    {
        let hijo_der = nodo.derecho.take().unwrap();
        nodo.derecho = Some(rotar_derecha(hijo_der));
        return Some(rotar_izquierda(nodo));
    }

    Some(nodo)
}

// =============== FASE 4: ALERTA DE COLISIÓN (Opción A) =======================

/// Cuenta cuántos vuelos están en un rango de altitud [min, max].
/// Detecta vuelos volando peligrosamente cerca en el espacio aéreo.
///
/// Eficiencia O(log n + k) donde k es el número de resultados:
/// - Si la altitud del nodo actual es MENOR que min, solo busca a la derecha
///   (descarta todo el subárbol izquierdo que tiene valores aún menores).
/// - Si es MAYOR que max, solo busca a la izquierda.
/// - Si está dentro del rango, cuenta 1 y busca en ambos subárboles.
///
/// Usa solo referencias inmutables (&): no modifica el árbol.
fn vuelos_en_rango(
    nodo: &Option<Box<Nodo>>,
    min: u32,
    max: u32,
) -> usize {

    match nodo {
        None => 0,

        Some(n) => {
            if n.vuelo.altitud < min {
                // Nodo fuera de rango por abajo: solo buscar a la derecha
                vuelos_en_rango(&n.derecho, min, max)

            } else if n.vuelo.altitud > max {
                // Nodo fuera de rango por arriba: solo buscar a la izquierda
                vuelos_en_rango(&n.izquierdo, min, max)

            } else {
                // Nodo DENTRO del rango: contar + buscar ambos lados
                1 + vuelos_en_rango(&n.izquierdo, min, max)
                  + vuelos_en_rango(&n.derecho, min, max)
            }
        }
    }
}

// ==================== UTILIDADES DE IMPRESIÓN =================================

/// Recorrido in-order (izquierda → raíz → derecha).
/// Imprime los vuelos en orden ascendente de altitud.
/// Usa referencias inmutables: solo lectura del árbol.
fn inorder(nodo: &Option<Box<Nodo>>) {
    if let Some(n) = nodo {
        inorder(&n.izquierdo);
        println!(
            "  ✈  {} — {} pies (altura nodo: {})",
            n.vuelo.id, n.vuelo.altitud, n.altura
        );
        inorder(&n.derecho);
    }
}

/// Imprime el árbol de forma visual con indentación para mostrar la estructura.
/// Útil para verificar que las rotaciones mantienen el balance correcto.
fn imprimir_arbol(nodo: &Option<Box<Nodo>>, prefijo: &str, es_izq: bool) {
    if let Some(n) = nodo {
        let conector = if es_izq { "├── " } else { "└── " };
        let extension = if es_izq { "│   " } else { "    " };

        // Primero imprimir subárbol derecho (arriba visualmente)
        imprimir_arbol(
            &n.derecho,
            &format!("{}{}", prefijo, extension),
            true,
        );

        // Imprimir nodo actual
        println!(
            "{}{}[{}] {} pies (h={}, bal={})",
            prefijo,
            conector,
            n.vuelo.id,
            n.vuelo.altitud,
            n.altura,
            obtener_balance(n),
        );

        // Luego imprimir subárbol izquierdo (abajo visualmente)
        imprimir_arbol(
            &n.izquierdo,
            &format!("{}{}", prefijo, extension),
            false,
        );
    }
}

// =========================== FUNCIÓN PRINCIPAL ===============================

fn main() {
    println!("╔══════════════════════════════════════════════════════════╗");
    println!("║     MOTOR DE TRÁFICO AÉREO — Radar AVL                  ║");
    println!("║     Aeropuerto Internacional de Santa Ana                ║");
    println!("╚══════════════════════════════════════════════════════════╝");

    // =====================================================================
    // FASE 1: ANÁLISIS DE SEGURIDAD Y PROPIEDAD
    // =====================================================================
    println!("\n{}", "=".repeat(60));
    println!("  FASE 1: ANÁLISIS DE SEGURIDAD Y PROPIEDAD");
    println!("{}", "=".repeat(60));

    // ── Pregunta 1: Option::take() y Ownership ──
    println!("\n─── ¿Qué ocurre con el Ownership en Option::take()? ───\n");
    println!("  En Rust cada valor tiene UN solo dueño. No podemos tener");
    println!("  dos variables apuntando al mismo dato en memoria.");
    println!("  Option::take() resuelve esto en las rotaciones:");
    println!("    1. EXTRAE el valor del Option (transfiere el ownership).");
    println!("    2. DEJA None en la posición original.");
    println!("  Así podemos mover un hijo de un nodo a otro sin que Rust");
    println!("  detecte referencias colgantes o doble ownership.");
    println!();
    println!("  Ejemplo en rotar_derecha():");
    println!("    let mut x = y.izquierdo.take().unwrap();");
    println!("    // → y.izquierdo ahora es None (ya no es dueño de x)");
    println!("    // → x es el nuevo dueño del subárbol");

    // ── Pregunta 2: ¿Por qué Box<Nodo>? ──
    println!("\n─── ¿Por qué usamos Box<Nodo> en vez de Nodo? ───\n");
    println!("  En Rust, el tamaño de cada tipo debe conocerse en tiempo");
    println!("  de compilación. Si pusiéramos:");
    println!("    struct Nodo {{ izquierdo: Option<Nodo>, ... }}");
    println!("  tendríamos un tipo recursivo de tamaño INFINITO.");
    println!();
    println!("  Box<T> almacena el dato en el HEAP y guarda solo un");
    println!("  puntero (8 bytes) en el STACK. Así el compilador conoce");
    println!("  el tamaño exacto de la estructura.");
    println!("  Cuando el Box sale de ámbito, libera la memoria (RAII).");

    // =====================================================================
    // FASE 1: PRUEBA DE ESCRITORIO — Inserción paso a paso
    // =====================================================================
    println!("\n{}", "=".repeat(60));
    println!("  FASE 1: PRUEBA DE ESCRITORIO");
    println!("  Inserciones: [5000, 3000, 2000, 4000, 3500, 6000]");
    println!("{}", "=".repeat(60));

    let mut radar: Option<Box<Nodo>> = None;

    // ── Paso 1: Insertar 5000 ──
    println!("\n── Paso 1: Insertar 5000 (AV123) ──\n");
    println!("  Árbol vacío → se crea el nodo raíz.\n");
    radar = Some(insertar(radar.take(), Vuelo {
        id: "AV123".to_string(), altitud: 5000,
    }));
    println!("      [5000]");
    println!("\n  Balance: 0 → Sin rotación ✓");

    // ── Paso 2: Insertar 3000 ──
    println!("\n── Paso 2: Insertar 3000 (UA456) ──\n");
    println!("  3000 < 5000 → va a la izquierda.\n");
    radar = Some(insertar(radar.take(), Vuelo {
        id: "UA456".to_string(), altitud: 3000,
    }));
    println!("        [5000]");
    println!("        /");
    println!("     [3000]");
    println!("\n  Balance de 5000: +1 → Sin rotación ✓");

    // ── Paso 3: Insertar 2000 (¡ROTACIÓN!) ──
    println!("\n── Paso 3: Insertar 2000 (IB101) ⚠ ROTACIÓN ──\n");
    println!("  2000 < 5000 → izq. 2000 < 3000 → izq.\n");
    println!("  ANTES de rotar:");
    println!("          [5000]  ← balance = +2 ¡desbalanceado!");
    println!("          /");
    println!("       [3000]");
    println!("       /");
    println!("    [2000]");
    println!();
    println!("  Caso: Izquierda-Izquierda");
    println!("  → ROTACIÓN SIMPLE A LA DERECHA sobre 5000");
    println!();
    radar = Some(insertar(radar.take(), Vuelo {
        id: "IB101".to_string(), altitud: 2000,
    }));
    println!("  DESPUÉS de rotar:");
    println!("       [3000]   ← nueva raíz");
    println!("       /    \\");
    println!("   [2000]  [5000]");
    println!("\n  Todos los balances = 0 ✓");

    // ── Paso 4: Insertar 4000 ──
    println!("\n── Paso 4: Insertar 4000 (AF999) ──\n");
    println!("  4000 > 3000 → der. 4000 < 5000 → izq.\n");
    radar = Some(insertar(radar.take(), Vuelo {
        id: "AF999".to_string(), altitud: 4000,
    }));
    println!("         [3000]");
    println!("         /    \\");
    println!("     [2000]  [5000]");
    println!("             /");
    println!("          [4000]");
    println!("\n  Balance de 3000: -1 → Sin rotación ✓");

    // ── Paso 5: Insertar 3500 (¡ROTACIÓN!) ──
    println!("\n── Paso 5: Insertar 3500 (TA222) ⚠ ROTACIÓN ──\n");
    println!("  3500 > 3000 → der. 3500 < 5000 → izq. 3500 < 4000 → izq.\n");
    println!("  ANTES de rotar:");
    println!("         [3000]");
    println!("         /    \\");
    println!("     [2000]  [5000]  ← balance = +2 ¡desbalanceado!");
    println!("             /");
    println!("          [4000]");
    println!("          /");
    println!("       [3500]");
    println!();
    println!("  Caso: Izquierda-Izquierda (en subárbol de 5000)");
    println!("  → ROTACIÓN SIMPLE A LA DERECHA sobre 5000");
    println!();
    radar = Some(insertar(radar.take(), Vuelo {
        id: "TA222".to_string(), altitud: 3500,
    }));
    println!("  DESPUÉS de rotar:");
    println!("         [3000]");
    println!("         /    \\");
    println!("     [2000]  [4000]");
    println!("             /    \\");
    println!("          [3500] [5000]");
    println!("\n  Balance de 3000: -1 → OK ✓");

    // ── Paso 6: Insertar 6000 (¡ROTACIÓN!) ──
    println!("\n── Paso 6: Insertar 6000 (AM777) ⚠ ROTACIÓN ──\n");
    println!("  6000 > 3000 → der. 6000 > 4000 → der. 6000 > 5000 → der.\n");
    println!("  ANTES de rotar:");
    println!("         [3000]    ← balance = -2 ¡desbalanceado!");
    println!("         /    \\");
    println!("     [2000]  [4000]");
    println!("             /    \\");
    println!("          [3500] [5000]");
    println!("                     \\");
    println!("                   [6000]");
    println!();
    println!("  Caso: Derecha-Derecha");
    println!("  → ROTACIÓN SIMPLE A LA IZQUIERDA sobre 3000");
    println!();
    radar = Some(insertar(radar.take(), Vuelo {
        id: "AM777".to_string(), altitud: 6000,
    }));
    println!("  DESPUÉS de rotar:");
    println!("            [4000]       ← nueva raíz final");
    println!("           /      \\");
    println!("       [3000]    [5000]");
    println!("       /    \\        \\");
    println!("   [2000] [3500]   [6000]");
    println!("\n  Todos los balances: 0, 0, -1 ✓ Balanceado");

    // ── Resumen de rotaciones ──
    println!("\n{}", "=".repeat(60));
    println!("  RESUMEN DE ROTACIONES");
    println!("{}", "=".repeat(60));
    println!();
    println!("  Paso | Altitud | Nodo afectado | Tipo de rotación");
    println!("  ─────┼─────────┼───────────────┼──────────────────────────");
    println!("    3  |  2000   |  5000 (bal +2) | Simple Derecha (L-L)");
    println!("    5  |  3500   |  5000 (bal +2) | Simple Derecha (L-L)");
    println!("    6  |  6000   |  3000 (bal -2) | Simple Izquierda (R-R)");
    println!();
    println!("  → Solo ocurrieron Rotaciones Simples (3 en total).");
    println!("  → No se presentaron Rotaciones Dobles.");

    // ── Verificación: árbol real ──
    println!("\n{}", "=".repeat(60));
    println!("  VERIFICACIÓN: ÁRBOL AVL REAL (generado por el código)");
    println!("{}", "=".repeat(60));
    println!();
    println!("  Recorrido in-order (debe salir ordenado):\n");
    inorder(&radar);
    println!("\n  Estructura del árbol:\n");
    imprimir_arbol(&radar, "  ", false);

    // =====================================================================
    // FASE 2: LOCALIZACIÓN DE VUELOS (búsqueda O(log n))
    // =====================================================================
    println!("\n{}", "=".repeat(60));
    println!("  FASE 2: LOCALIZACIÓN DE VUELOS");
    println!("{}", "=".repeat(60));
    println!();
    println!("  buscar_vuelo() usa solo referencias (&) → solo lectura.");
    println!("  Complejidad O(log n): descarta mitad del árbol en cada paso.");

    println!("\n── Buscar altitud 3500 ──\n");
    match buscar_vuelo(&radar, 3500) {
        Some(v) => println!("  ✓ Encontrado: {} a {} pies", v.id, v.altitud),
        None => println!("  ✗ No encontrado"),
    }

    println!("\n── Buscar altitud 9999 (no existe) ──\n");
    match buscar_vuelo(&radar, 9999) {
        Some(v) => println!("  ✓ Encontrado: {} a {} pies", v.id, v.altitud),
        None => println!("  ✗ No encontrado en el radar"),
    }

    // =====================================================================
    // FASE 4: ALERTA DE COLISIÓN (Opción A — rango)
    // =====================================================================
    println!("\n{}", "=".repeat(60));
    println!("  FASE 4: ALERTA DE PROXIMIDAD");
    println!("{}", "=".repeat(60));
    println!();
    println!("  vuelos_en_rango() cuenta aviones en [min, max].");
    println!("  Poda ramas fuera de rango → eficiente.\n");

    let cantidad = vuelos_en_rango(&radar, 3000, 5000);
    println!("  Rango 3000–5000 pies: {} vuelos en zona de riesgo ⚠", cantidad);

    let cantidad2 = vuelos_en_rango(&radar, 2000, 3500);
    println!("  Rango 2000–3500 pies: {} vuelos en zona de riesgo ⚠", cantidad2);

    // =====================================================================
    // FASE 3: DESCENSO Y ATERRIZAJE (eliminación)
    // =====================================================================
    println!("\n{}", "=".repeat(60));
    println!("  FASE 3: DESCENSO Y ATERRIZAJE (Eliminación)");
    println!("{}", "=".repeat(60));

    // ── Eliminar 4000 (raíz, tiene 2 hijos → Caso 3) ──
    println!("\n── Eliminar 4000 (AF999) — Caso 3: dos hijos ──\n");
    println!("  El nodo 4000 es la raíz y tiene dos hijos.");
    println!("  Se busca el predecesor in-order: el MAYOR del subárbol izq.");
    println!("  Predecesor = 3500. Se reemplaza 4000 por 3500");
    println!("  y se elimina el 3500 original del subárbol izquierdo.\n");

    println!("  ANTES:");
    println!("            [4000]");
    println!("           /      \\");
    println!("       [3000]    [5000]");
    println!("       /    \\        \\");
    println!("   [2000] [3500]   [6000]");

    radar = eliminar_vuelo(radar.take(), 4000);

    println!("\n  DESPUÉS:");
    println!("            [3500]    ← 3500 reemplazó a 4000");
    println!("           /      \\");
    println!("       [3000]    [5000]");
    println!("       /              \\");
    println!("   [2000]           [6000]");

    println!("\n  Árbol real tras eliminación:\n");
    imprimir_arbol(&radar, "  ", false);

    // ── Eliminar 6000 (hoja → Caso 1) ──
    println!("\n── Eliminar 6000 (AM777) — Caso 1: sin hijos (hoja) ──\n");
    println!("  El nodo 6000 no tiene hijos → se elimina directamente.\n");
    radar = eliminar_vuelo(radar.take(), 6000);
    println!("  Árbol real tras eliminación:\n");
    imprimir_arbol(&radar, "  ", false);

    // ── Eliminar 5000 (un hijo → Caso 2) ──
    println!("\n── Eliminar 5000 (AV123) — Caso 2: un solo hijo ──\n");
    println!("  El nodo 5000 no tiene hijos (6000 ya fue eliminado).");
    println!("  → Se elimina directamente (es hoja ahora).\n");
    radar = eliminar_vuelo(radar.take(), 5000);
    println!("  Árbol real tras eliminación:\n");
    imprimir_arbol(&radar, "  ", false);

    // ── Verificación final ──
    println!("\n{}", "=".repeat(60));
    println!("  VERIFICACIÓN FINAL");
    println!("{}", "=".repeat(60));

    println!("\n  Recorrido in-order (vuelos restantes):\n");
    inorder(&radar);

    let cantidad = vuelos_en_rango(&radar, 0, 99999);
    println!("\n  Total de vuelos en radar: {}", cantidad);

    println!("\n  Búsqueda de vuelo eliminado (4000):");
    match buscar_vuelo(&radar, 4000) {
        Some(v) => println!("  ✓ Encontrado: {:?}", v),
        None => println!("  ✗ 4000 ya no existe en el radar (eliminado correctamente)"),
    }

    println!();
}