extern crate actix;
use crate::synonym::providers::{base, http_requester, thesaurus, thesaurus2, merriam_webster};
use actix::{Actor, Addr, Context, Handler, Message, System};
use std::collections::HashMap;

use self::actix::{AsyncContext};

#[derive(Message)]
#[rtype(result = "()")]
struct FindSynonyms(String);

#[derive(Message)]
#[rtype(result = "()")]
struct ExecuteRequest();

#[derive(Message)]
#[rtype(result = "()")]
struct Synonyms {
    word: String,
    synonyms: Vec<String>
}

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
    provider_type: base::Provider,
}

impl Actor for Provider {
    type Context = Context<Self>;
}

impl Handler<FindSynonyms> for Provider {
    type Result = ();

    fn handle(&mut self, _msg: FindSynonyms, _ctx: &mut Context<Self>) -> Self::Result {
        // TODO: Esta clase podria hacer el sleep de MIN_TIME_BETWEEN_REQUESTS ya que es la
        // encargada de hacer la peticion de buscar sinonimos segun provider_type
        let base_url = match self.provider_type {
            base::Provider::Thesaurus => "https://thesaurus.yourdictionary.com/",
            base::Provider::MerriamWebster => "https://www.merriam-webster.com/thesaurus/",
            base::Provider::Thesaurus2 => "https://www.thesaurus.com/browse/",
        }.to_string();
        
        let word = _msg.0;
        match http_requester::fetch_synonyms_raw_response(word.clone(), base_url) {
            Err(error) => { panic!("{}", error); },
            Ok(result) => {
                match match self.provider_type {
                    base::Provider::Thesaurus => thesaurus::raw_response_to_synonyms(result),
                    base::Provider::MerriamWebster => merriam_webster::raw_response_to_synonyms(result),
                    base::Provider::Thesaurus2 => thesaurus2::raw_response_to_synonyms(result)
                } {
                    Ok(synonyms) => {
                        self.result_addr.do_send(Synonyms{word: word.clone(), synonyms});
                        self.main_addr.do_send(ProviderFinished());
                    }
                    Err(error) => { panic!("{}", error); }
                }
            }
        };
    }
}

// No recuerdo para que habiamos definido este mensaje
impl Handler<ExecuteRequest> for Provider {
    type Result = ();

    fn handle(&mut self, _msg: ExecuteRequest, _ctx: &mut Context<Self>) -> Self::Result {
        return ();
    }
}

struct GlobalResult {
    synonyms: HashMap<String, HashMap<String, usize>>, // Nested HashMap
}

impl Actor for GlobalResult {
    type Context = Context<Self>;
}

impl Handler<Synonyms> for GlobalResult {
    type Result = ();

    fn handle(&mut self, _msg: Synonyms, _ctx: &mut Context<Self>) -> Self::Result {
        // 1. agregar/incrementar en this.synonyms para la word actual
        let word_partial_result = 
            self.synonyms.entry(_msg.word)
            .or_insert(HashMap::new());

        for synonym in _msg.synonyms {
            let synonym_word_count = word_partial_result
                                .entry(synonym)
                                .or_insert(0);
            *synonym_word_count += 1;
        }
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
        self.result_addr = GlobalResult {
            synonyms: HashMap::new(),
        }
        .start();
        self.logger_addr = Logger {}.start();
        self.providers_addr[0] = Provider {
            result_addr: self.result_addr.clone(),
            logger_addr: self.logger_addr.clone(),
            main_addr: _ctx.address(),
            provider_type: base::Provider::MerriamWebster
        }
        .start();
        self.providers_addr[1] = Provider {
            result_addr: self.result_addr.clone(),
            logger_addr: self.logger_addr.clone(),
            main_addr: _ctx.address(),
            provider_type: base::Provider::Thesaurus
        }
        .start();
        self.providers_addr[2] = Provider {
            result_addr: self.result_addr.clone(),
            logger_addr: self.logger_addr.clone(),
            main_addr: _ctx.address(),
            provider_type: base::Provider::Thesaurus2
        }
        .start();
        // Instanciar los demas providers...
        self.num_providers = self.providers_addr.len();

        // 1. leer archivo, por cada palabra:
        self.num_words += 1;
        self.providers_addr[0].do_send(FindSynonyms("car".to_string()));
        // ...
    }
}

impl Handler<ProviderFinished> for Main {
    type Result = ();

    fn handle(&mut self, _msg: ProviderFinished, _ctx: &mut Context<Self>) -> Self::Result {
        self.num_processed_words += 1;

        if self.num_words * self.num_providers == self.num_processed_words * self.num_providers {
            self.result_addr.do_send(Finish()); // Condicion de corte
        }
    }
}

pub async fn main() {
    let system = System::new();
    system.block_on(async {});
    // TODO
    system.run().unwrap();
}
