extern crate actix;
use actix::{Addr, Actor, Context, Handler, System, Message};
use std::collections::HashMap;
use self::actix::{Recipient, AsyncContext};

#[derive(Message)]
#[rtype(result = "()")]
struct FindSynonyms(String);

#[derive(Message)]
#[rtype(result = "()")]
struct Synonyms(HashMap<String, Vec<String>>); // Nested HashMap

#[derive(Message)]
#[rtype(result = "()")]
struct Finish();

#[derive(Message)]
#[rtype(result = "()")]
struct Init();

#[derive(Message)]
#[rtype(result = "()")]
struct ProviderFinished();

struct Logger {}

impl Actor for Logger {
    type Context = Context<Self>;
}

struct Provider {
    result_addr: Addr<GlobalResult>,
    logger_addr: Addr<Logger>,
    main_addr: Addr<Main>,
}

impl Actor for Provider {
    type Context = Context<Self>;
}

impl Handler<FindSynonyms> for Provider {
    type Result = ();

    fn handle(&mut self, _msg: FindSynonyms, _ctx: &mut Context<Self>) -> Self::Result {
        // TODO: evaluar requests en vuelo y tiempo min entre requests
        // 1. request http
        // 2. parse html
        // 3. res_addr.send(Synonyms)
        // 4. main_addr.send(ProviderFinished)
    }
}

struct GlobalResult {
    synonyms: HashMap<String, Vec<String>>, // Nested HashMap
}

impl Actor for GlobalResult {
    type Context = Context<Self>;
}

impl Handler<Synonyms> for GlobalResult {
    type Result = ();

    fn handle(&mut self, _msg: Synonyms, _ctx: &mut Context<Self>) -> Self::Result {
        // 1. agregar/incrementar en this.synonyms para la word actual
    }
}

impl Handler<Finish> for GlobalResult {
    type Result = ();

    fn handle(&mut self, _msg: Finish, _ctx: &mut Context<Self>) -> Self::Result {
        // 1. imprimir resultado global desde this.synonyms
    }
}

struct Main {
    num_words: usize,
    num_processed_words: usize,
    num_providers: usize,
    result_addr: Addr<GlobalResult>,
    logger_addr: Addr<Logger>,
    providers_addr: Vec<Addr<Provider>>,
}

impl Actor for Main {
    type Context = Context<Self>;
}

impl Handler<Init> for Main {
    type Result = ();

    fn handle(&mut self, _msg: Init, _ctx: &mut Context<Self>) -> Self::Result {
        self.result_addr = GlobalResult { synonyms: HashMap::new() }.start();
        self.logger_addr = Logger {}.start();
        self.providers_addr[0] = Provider { result_addr: self.result_addr.clone(), logger_addr: self.logger_addr.clone(), main_addr: _ctx.address() }.start();
        // Instanciar los demas providers...
        self.num_providers = self.providers_addr.len();

        // 1. leer archivo, por cada palabra:
            self.num_words += 1;
            self.providers_addr[0].send(FindSynonyms("car".to_string()));
            // ...
    }
}

impl Handler<ProviderFinished> for Main {
    type Result = ();

    fn handle(&mut self, _msg: ProviderFinished, _ctx: &mut Context<Self>) -> Self::Result {
        self.num_processed_words += 1;

        if self.num_words * self.num_providers == self.num_processed_words * self.num_providers {
            self.result_addr.send(Finish()); // Condicion de corte
        }
    }
}


pub async fn main() {
    let system = System::new();
    system.block_on(async {});
    // TODO
    system.run().unwrap();
}