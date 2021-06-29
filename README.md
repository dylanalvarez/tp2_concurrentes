# Buscador de Sinónimos Rústico
## Integrantes
* Donato, Juan Pablo
* Botalla, Tomas
* Alvarez, Dylan

## Ejecución
El programa recibe 4 parámetros de entrada:
- **option**: con cual de las soluciones hacer la búsqueda. Valores posibles: *actors* / *without_actors* 
- **max_concurrent_requests**: cantidad máxima de requests HTTP a procesar en forma concurrente para todos los sitios
- **min_seconds_between_requests**: tiempo mínimo de espera entre dos requests HTTP sucesivos para un sitio dado
- **filename**: nombre del archivo que contiene las palabras a buscar

Por ejemplo para ejecutar la solución con el modelo de actores, con a lo sumo 2 requests en forma paralela, un tiempo mínimo de 1 segundo entre requests para un mismo proveedor, y dado un archivo de entrada words.txt:
> cargo run actors 2 1 words.txt

## Enunciado

#### Fecha de entrega: 29 de junio antes de las 19 hs.

### Objetivo

El presente trabajo práctico tiene como objetivo implementar un software buscador de sinónimos de palabras.

Para ello se realizarán peticiones de varios sitios de Internet que ofrecen este servicio.

### Requerimientos Funcionales

Se debe implementar un software para ser ejecutado en la consola de comandos que busque sinónimos de palabras en distintos sitios de Internet.

Se usarán los siguientes sitios que permiten buscar sinóminos de palabras en inglés:

* https://thesaurus.yourdictionary.com/
* https://www.thesaurus.com/browse/
* https://www.merriam-webster.com/thesaurus/

Por ejemplo, para buscar sinónimos de "car", se debe invocar:

* https://thesaurus.yourdictionary.com/car
* https://www.thesaurus.com/browse/car
* https://www.merriam-webster.com/thesaurus/car

_Nota_: Estos tres proveedores de sinónimos son los que disponemos hoy en día. En un futuro se debería poder incorporar fácilmente nuevos proveedores, el único trabajo que se debería hacer es el parser específico de la respuesta, para extraer las palabras.

El programa debe recibir por parámetro la ruta a un archivo que contiene una lista de palabras sobre las que se quieren buscar sinónimos (una palabra por línea del archivo) y los parámetros de configuración de la ejecución, como se explica más adelante.

Para realizar las consultas de los sinónimos, se debe realizar un pedido (request) HTTP a cada una de las direcciones. Para eso, se debe utilizar el crate reqwest. Este crate debe ser usado en forma bloqueante (ver blocking).

Debe procesarse el texto de respuesta de cada una de las invocaciones (string) para extraer las palabras que son sinónimos. Para esto, debe utilizarse los métodos de String de la std del lenguaje.

Los resultados deben mostrarse de forma consolidada, sin repetir los resultados (en caso de que se encuentren en más de un sitio). Al lado de cada resultado se debe indicar entre paréntesis, en cuántos sitios apareció ese valor.

Las parámetros que se debe poder configurar son:

* Cantidad máxima de pedidos a los sitios web (requests) a procesar de forma concurrente.
* Tiempo de espera mínimo entre dos invocaciones al mismo sitio de sinónimos.

Se debe divir el programa en hilos de ejecución (threads) que sean lo más chico posibles y con una responsabilidad acotada, que permitan realizar las diversas tareas de forma concurrente.

Se debe escribir un archivo de log con las operaciones que se realizan y sus resultados.

#### Parte A

Implementar el programa utilizando las herramientas de concurrencia de la biblioteca standard de Rust vistas en clase: Mutex, RwLock, Semáforos (del crate std-semaphore), Channels, Barriers y Condvars.

#### Parte B

Implementar el programa basado en el modelo de Actores, utilizando el framework Actix.

### Requerimientos no funcionales

Los siguientes son los requerimientos no funcionales para la resolución de los ejercicios:

* El proyecto deberá ser desarrollado en lenguaje Rust, usando las herramientas de la biblioteca estándar.
* No se permite utilizar crates externos, salvo los explícitamente mencionados.
* El código fuente debe compilarse en la última versión stable del compilador y no se permite utilizar bloques unsafe.
* El código deberá funcionar en ambiente Unix / Linux.
* El programa deberá ejecutarse en la línea de comandos.
* La compilación no debe arrojar warnings del compilador, ni del linter clippy.
* Las funciones y los tipos de datos (struct) deben estar documentadas siguiendo el estándar de cargo doc.
* El código debe formatearse utilizando cargo fmt.
* Cada tipo de dato implementado debe ser colocado en una unidad de compilación (archivo fuente) independiente.

### Tareas a Realizar

A continuación se listan las tareas a realizar para completar el desarrollo del proyecto:

* Dividir el proyecto en threads. El objetivo es lograr que la simulación esté conformada por un conjunto de hilos de ejecución que sean lo más sencillos posible.
* Una vez obtenida la división en threads, establecer un esquema de comunicación entre ellos teniendo en cuenta los requerimientos de la aplicación. ¿Qué threads se comunican entre sı́? ¿Qué datos necesitan compartir para poder trabajar?
* Realizar la codificación de la aplicación. El código fuente debe estar documentado.
* Implementar tests unitarios de las funciones que considere relevantes.

### Entrega

La resolución del presente proyecto es en grupos de tres integrantes.

La entrega del proyecto comprende lo siguiente:

* Informe, se deberá presentar en forma digital (PDF) enviado por correo electrónico a la dirección: pdeymon@fi.uba.ar
* El código fuente de la aplicación, que se entregará únicamente por e-mail. El código fuente debe estar estructurado en un proyecto de cargo, y se debe omitir el directorio target/ en la entrega. El informe a entregar debe contener los siguientes items:
  * Una explicación del diseño y de las decisiones tomadas para la implementación de la solución.
  * Detalle de resolución de la lista de tareas anterior.
  * Diagrama que refleje los threads, el flujo de comunicación entre ellos y los datos que intercambian.
  * Diagramas de entidades realizados (structs y demás).
