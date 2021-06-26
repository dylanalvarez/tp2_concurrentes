# Trabajo Práctico 2 - Buscador de Sinónimos Rústico
## Introduccion
El objetivo de este informe será presentar y detallar las soluciones implementadas por el grupo 1 para la construccion del trabajo practico 2, el cuál consistió en implementar un buscador de palabras utilizando distintos proveedores de informacion en la web, mediante el uso del lenguaje de Rust, sus distintas alternativas para manejar concurrencia y librerias.

## Solucion A - "Sin Actores"

## Solución B - "Con Actores"

En esta sección explicaremos los detalles mas importantes de esta solución B, donde como herramienta principal de manejo de concurrencia utilizamos el modelo de **Actores**, provistos por la libreria *actix*. A continuacion, un gráfico de soporte para entender la arquitectura básica y el modelo de actores implementados en esta sección:

![](esquema-actores.png)

En el esquema, podemos ver a los siguientes Actores:

|Actor|Descripcion|
|-----|-----------|
|**Main** |Actor principal en la solucion. Da comienzo a la ejecucion del proceso y, cuando lo determine, tambien lo finaliza|
|**Provider**| Actor que representa a un proveedor (por ejemplo, Merriam-Webster). Son "tipados" segun el proveedor que les toca y contienen la logica de parseo, segun el sitio|
|**Provider Coordinator**| Actor que contiene la logica de coordinacion y tiempo minimo entre requests de proveedores del mismo sitio.|
| **HttpRequester** | Tiene la logica basica de realizar el request Http al sitio y con la palabra que se le indique. Devuelve el body del request |
| **Global Result** | Actor que centraliza el resultado global de la ejecucion del programa. Dada una palabra y una lista de sinonimos, los agrupa y lleva el conteo de las repeticiones segun resultados. |
| **Logger** | Actor que tiene la tarea de logear sucesos indicados en el archivo de output |

<br>
<br>
<br>
Y por otra parte tambien podemos ver la lista de Mensajes implementados. A continuacion, el detalle:

|Mensaje|Descripcion|
|-------|-----------|
|*Init* | Mensaje de inicio del programa. Contiene la ruta del archivo a parsear con las palabras a buscar y el tiempo de espera minimo en milisegundos |
| *FindSynonyms* | Mensaje de petición a un provider para buscar los sinonimos de determinada palabra |
| *SendRequest* | Mensaje de peticion de realización de request Http contra un sitio en particular, para buscar los sinonimos de una palabra. Contiene ademas la direccion de la casilla del proveedor que espera recibir el resultado. |
| *RequestResult* | Una vez obtenido el resultado del request (producto del mensaje de arriba), se lo envia utilizando este mensaje|
| *Synonyms* | Luego de realizar la tarea de parseo que le corresponda a cada proveedor, mediante este mensaje se envian la palabra y la lista de sinonimos encontrados |
| *ProviderFinished* | Mensaje que envia un proveedor para indicar que su tarea ha finalizado |
| *Finish* | Mensaje final para indicar que todos los proveedores han finalizado y se espera obtener el resultado final |
| *Log* | Mensaje que contiene la cadena a escribir en el archivo de output del proceso |

<br>
<br>
Luego del detalle de los principales componentes, pasamos a describir las caracteristicas del flujo principal del programa para realizar la busqueda de los sinonimos y obtener el resultado final:

- En la funcion `start_actors` que se llama para comenzar el programa se inicializan los actores **Main**, **GlobalResult**, **Logger** y **HttpRequester**, y se envia el mensaje *Init* al **Main** para comenzar la ejecucion.
  - La caracteristica escencial en este paso es la forma en como se inicializa el actor **HttpRequester**, la cual es clave para la solucion de la cantidad maxima de requests concurrentes permitida. En lugar de inicializarlo utilizando la forma mas convencional (`Actor.start()`), se inicializa utilizando la función `SyncArbitrer::start` provista por la libreria de actix, que permite indicarle que cantidad de threads tiene que levantar para inicializar el actor que le pasamos por parámetro. De esta forma, por ejemplo, si quisieramos realizar como máximo 3 requests de forma concurrente, entonces pasandole el número indicado a esta funcion entonces podriamos "instanciar" de alguna manera tres instancias del **HttpRequester** que permitirán procesar tres mensajes *SendRequest* como máximo de forma concurrente.
- Luego de recibir el mensaje *Init*, el actor **Main** instancia un **Provider** de cada tipo, junto con el **ProviderCoordinator** que le corresponda. De esta forma, cada **Provider** de cada tipo tiene su propio **ProviderCoordinator**.
- Luego de instanciar los proveedores, parsea el archivo de palabras y envia un mensaje *FindSynonyms* a cada proveedor que tenga para indicarle que debe buscar los sinonimos de esa palabra.
- 