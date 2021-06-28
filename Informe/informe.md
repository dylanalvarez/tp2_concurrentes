# Trabajo Práctico 2 - Buscador de Sinónimos Rústico
## Integrantes
|Integrante|Padrón|Mail|
|----------|------|----|
|Botalla, Tomás| 96356 | tbotalla@fi.uba.ar |
|Alvarez, Dylan| 98225 | dylanalvarez1995@gmail.com |
|Donato, Juan Pablo| 100839 | judonato@fi.uba.ar |
## Introducción
El objetivo de este informe será presentar y detallar las soluciones implementadas por el grupo 1 para la construcción del trabajo práctico 2, el cuál consistió en implementar un buscador de sinónimos de palabras utilizando distintos proveedores de información en la web, mediante el uso del lenguaje de Rust, sus distintas alternativas para manejar concurrencia y librerias.

## Solución A - "Sin Actores"
Para la solucion A (a la que decidimos llamar "sin actores") lo que intentamos hacer fue aplicar la mayor cantidad y variedad de las herramientas de manejo de concurrencia vistas en clase, como *Semaforos, Canales, Condvars y Mutex*. A continuacion, el detalle del programa.

- Para lo que son los "proveedores" de sinonimos (Thesaurus, Thesaurus2 y Merriam-Webster), para cada uno de ellos creamos un thread que realiza las siguientes tareas:
  - Lee el archivo de input de palabras
  - Por cada palabra a buscarle los sinonimos, se crea un thread que hace el trabajo de hacer el request HTTP, parsear esa respuesta, obtener los sinonimos y luego enviarlos a traves de un *canal* al thread que recolecta los resultados (lo veremos en los siguientes puntos). Cada uno de estos threads lanzados se almacenan en una lista.
- Para la recolección de los resultados finales, lanzamos un thread al que denominamos `result_builder` el cual recibe a traves del canal que mencionamos antes los resultados de los distintos proveedores ejecutados, y los almacena en una estructura de tipo hash llevando el conteo de las repeticiones de cada sinonimo. Además, cuando ya no queden threads de tipo "providers" ejecutandose, el `result_builder` recibira un mensaje indicando que ya no hay mas sinonimos, devolviendo el resultado final en el join.
- Para lo que es el manejo de la *cantidad maxima de requests concurrentes* que se pueden realizar a los distintos sitios, utilizamos un **semaforo** inicializado con el parametro enviado por linea de comandos en donde, previo a hacer el request, los providers hacen el `semaphore.acquire()` necesario, y posteriormente a obtener la respuesta de parte del sitio que corresponda realizan el `semaphore.release()` que corresponde.
- Para lo que es el manejo del *tiempo minimo entre requests del mismo provider*, nuestra solución propuesta incluye la utilizacion de una **Condvar** (con su **Mutex**) correspondiente, en donde cada provider, previo a parsear el archivo de sinonimos, lanza un thread que denominaremos `sleeper` cuya funcion es recibir una alerta de algun provider que quiera buscar un sinonimo (a traves de un **canal**), y el `sleeper` efectuará un `sleep` del *tiempo minimo entre requests* y notificara a los proveedores que estan esperando que se ha cumplido este tiempo, utilizando la variable condicional mencionada anteriormente. Por si parte, los providers no avanzaran en el proceso de efectuar el request HTTP al sitio que le corresponda hasta tanto no obtener el mutex de la variable condicional, y que el valor de dicha variable sea el indicado (simplemente, un booleano en `true` en nuestro caso).
- Para lo que se trata del registro de sucesos a traves de un archivo de output, lo que implementamos fue un thread denominado `logger` que recibe mensajes a traves de un **canal** (cuya referencia 'sender' esta compartida por todos los demas threads) y los deposita en el archivo de output.
- Para finalizar, el thread "main" de la aplicacion, luego de lanzar todos los threads indicados procede a:
  - Hacer un `join` a todos los providers lanzados hasta que finalicen con su tarea
  - Enviar un mensaje al `result_builder` para indicarle que ya finalizaron todos los providers (mensaje del tipo `NoMoreSynonyms`)
  - Hacer un `join` al `result_builder` para obtener el resultado e imprimirlo.
  - En el proceso, utilizar al `logger` para dejar registros del flujo de la aplicacion.
## Solución B - "Con Actores"

En esta sección explicaremos los detalles mas importantes de esta solución B, donde como herramienta principal de manejo de concurrencia utilizamos el modelo de **Actores**, provistos por la libreria *actix*. A continuación, un gráfico de soporte para entender la arquitectura básica y el modelo de actores implementados en esta sección:

![](esquema-actores.png)

En el esquema, podemos ver a los siguientes Actores:

|Actor|Descripción|
|-----|-----------|
|**Main** |Actor principal en la solución. Da comienzo a la ejecución del proceso y, cuando lo determine, tambien lo finaliza|
|**Provider**| Actor que representa a un proveedor (por ejemplo, Merriam-Webster). Son "tipados" segun el proveedor que les toca y contienen la logica de parseo, segun el sitio|
|**Provider Coordinator**| Actor que contiene la logica de coordinación y tiempo minimo entre requests de proveedores del mismo sitio.|
| **HttpRequester** | Tiene la logica basica de realizar el request Http al sitio y con la palabra que se le indique. Devuelve el body del request |
| **Global Result** | Actor que centraliza el resultado global de la ejecución del programa. Dada una palabra y una lista de sinonimos, los agrupa y lleva el conteo de las repeticiones segun resultados. |
| **Logger** | Actor que tiene la tarea de logear sucesos indicados en el archivo de output |

<br>
<br>
<br>
Y por otra parte tambien podemos ver la lista de Mensajes implementados. A continuación, el detalle:

|Mensaje|Descripción|
|-------|-----------|
|*Init* | Mensaje de inicio del programa. Contiene la ruta del archivo a parsear con las palabras a buscar y el tiempo de espera minimo en milisegundos |
| *FindSynonyms* | Mensaje de petición a un provider para buscar los sinonimos de determinada palabra |
| *SendRequest* | Mensaje de petición de realización de request Http contra un sitio en particular, para buscar los sinonimos de una palabra. Contiene ademas la dirección de la casilla del proveedor que espera recibir el resultado. |
| *RequestResult* | Una vez obtenido el resultado del request (producto del mensaje de arriba), se lo envia utilizando este mensaje|
| *Synonyms* | Luego de realizar la tarea de parseo que le corresponda a cada proveedor, mediante este mensaje se envian la palabra y la lista de sinonimos encontrados |
| *ProviderFinished* | Mensaje que envia un proveedor para indicar que su tarea ha finalizado |
| *Finish* | Mensaje final para indicar que todos los proveedores han finalizado y se espera obtener el resultado final |
| *Log* | Mensaje que contiene la cadena a escribir en el archivo de output del proceso |

<br>
<br>
Luego del detalle de los principales componentes, pasamos a describir las caracteristicas del flujo principal del programa para realizar la busqueda de los sinonimos y obtener el resultado final:

- En la función `start_actors` que se llama para comenzar el programa se inicializan los actores **Main**, **GlobalResult**, **Logger** y **HttpRequester**, y se envia el mensaje *Init* al **Main** para comenzar la ejecución.
  - La caracteristica esencial en este paso es la forma en como se inicializan los actores **HttpRequester** y **Logger**, la cual es clave tanto para limitar la cantidad máxima de requests concurrentes permitidos, como para escribir las operaciones a un archivo en un hilo separado. En lugar de inicializarlo utilizando la forma mas convencional (`Actor.start()`), la cual inicia a los actores dentro del Arbitrer principal, con su hilo y event loop correspondientes, para estos dos actores su inicialización se hace con la función `SyncArbitrer::start` provista por la libreria de actix. Esta forma permite indicarle que cantidad de threads tiene que levantar para inicializar el actor que le pasamos por parámetro y, en consecuencia ese actor se ejecutará en esos hilos. De esta forma, por ejemplo, si quisieramos realizar como máximo 3 requests de forma concurrente, entonces pasandole el número indicado a esta función podriamos crear de alguna manera tres instancias del **HttpRequester** que permitirán procesar tres mensajes *SendRequest* como máximo de forma concurrente. Algo similar es lo que se hace con el actor **Logger** donde, se lo inicializa con la función SyncArbitrer::start con un hilo dedicado, permitiendo que las operaciones de este actor que son escrituras a disco, no afecten a la ejecución de los demás actores.
- Luego de recibir el mensaje *Init*, el actor **Main** instancia un **Provider** de cada tipo, junto con el **ProviderCoordinator** que le corresponda. De esta forma, cada **Provider** de cada tipo tiene su propio **ProviderCoordinator**.
- Luego de instanciar los proveedores, parsea el archivo de palabras y envia un mensaje *FindSynonyms* a cada proveedor que tenga para indicarle que debe buscar los sinonimos de esa palabra.
- Para comenzar, cada **Provider** necesita del material para parsear. Entonces, envia el mensaje *SendRequest* al **HttpProvider**, utilizando a su **ProviderCoordinator** como "middleware" en la operación.
  - La tarea del coordinador es entonces, hacer de mediador entre el provider y el requester para coordinar los tiempos minimos entre requests para cada proveedor.
  - La forma de hacerlo es frenando al Coordinator por un tiempo `MIN_TIME_BEWTWEEN_REQUESTS` de recibir nuevos eventos, y simplemente luego reenviar el mensaje al **HttpRequester**.
- Al recibir el mensaje *SendRequest*, el requester efectua la petición al proveedor indicado, y devuelve la respuesta a la dirección del provider que lo haya solicitado.
- Cuando el provider reciba la respuesta, parsea el HTML del contenido (segun las reglas establecidas para cada proveedor), y envia al **GlobalResult** la lista de sinonimos encontrados. Por último, envia el mensaje *ProviderFinished* al actor **Main** para indicarle que ha finalizado su tarea.
- El actor **Main** lleva el conteo de la cantidad de **Providers** que hayan finalizado su tarea de busqueda de sinonimos. Cuando todos hayan terminado de buscar los sinonimos de todas las palabras, envia el mensaje final *Finish* al **GlobalResult** para que devuelva como output el resultado final.
  - Al recibir el mensaje *Finish*, el actor **GlobalResult** envía un mensaje final al **Logger** con la función `send()` a diferencia de los demás que se ejecutaban con `try_send()`. Esto hace que se espere por la respuesta del **Logger**, lo cual es necesario ya que si no se esperara, podría ocurrir que el **GlobalResult** finalice la ejecución con `System::current().stop()` antes de que el **Logger** terminara de procesar los mensajes en su casilla.


## Conclusiones

Con el desarrollo del presente trabajo práctico, hemos puesto en práctica la mayoria de los conceptos y herramientas vistas durante el curso de la materia hasta el momento, de los cuales podemos destacar:

- La utilizacion de Rust como lenguaje de programacion, junto con sus librerias estándar y otras (como `reqwest` o `actix`) para lograr el manejo de la concurrencia.
- El empleo de herramientas de manejo de concurrencia básicas como ser *Semaforos, Condvars, Mutexes y Canales* durante la ejecucion de un programa multi-thread para lograr la coodinacion entre ellos.
- El empleo de un modelo orientado en *Actores y Mensajes* como opción alternativa de herramienta de manejo de concurrencia.
- Conceptos y técnicas básicas de programacion en Rust, como ser estructuracion de un programa en Rust, escritura en archivos, efectuar consultas HTTP a traves de la web, y hasta testing.
