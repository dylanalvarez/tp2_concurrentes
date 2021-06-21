extern crate actix;
use crate::synonym::providers::{base, http_requester, thesaurus, thesaurus2, merriam_webster};
use crate::synonym::providers::base::Provider::{MerriamWebster, Thesaurus, Thesaurus2};
use actix::{Actor, ActorFutureExt, Addr, Context, Handler, Message, ResponseFuture, System, WrapFuture};
use std::collections::HashMap;
use self::actix::{AsyncContext, Recipient, SyncArbiter, SyncContext};

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
#[derive(Message)]
#[rtype(result = "Result<String, String>")]
struct SendRequest{
    url: String,
    word: String
}

struct Logger {}

impl Actor for Logger {
    type Context = Context<Self>;
}

struct Provider {
    result_addr: Addr<GlobalResult>,
    requester_addr: Addr<HttpRequester>,
    logger_addr: Addr<Logger>,
    main_addr: Addr<Main>,
    provider_type: base::Provider,
}

impl Actor for Provider {
    type Context = Context<Self>;
}

struct HttpRequester {
    logger_addr: Addr<Logger>,
}

impl Actor for HttpRequester {
    type Context = SyncContext<Self>;
}

impl Handler<SendRequest> for HttpRequester {
    type Result = ResponseFuture<Result<String, String>>;

    fn handle(&mut self, msg: SendRequest, ctx: &mut Self::Context) -> Self::Result {
        Box::pin(async move {
            http_requester::fetch_synonyms_raw_response(msg.word, msg.url)
        })
    }
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
        Box::pin(async {
            let response = self.requester_addr
                .send(SendRequest { word: word.clone(), url: base_url.clone()}).await;
            match response {
                Err(error) => { panic!("{}", error); },
                Ok(result) => {
                    match result {
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
        }).into_actor(self);
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
        print!("{:?}", self.synonyms);
        System::current().stop();
    }
}

struct Main {
    num_words: usize,
    num_processed_words: usize,
    num_providers: usize,
    result_addr: Addr<GlobalResult>,
    requester_addr: Addr<HttpRequester>,
    logger_addr: Addr<Logger>,
    providers_addr: Vec<Addr<Provider>>,
}

impl Actor for Main {
    type Context = Context<Self>;
}

impl Handler<Init> for Main {
    type Result = ();

    fn handle(&mut self, _msg: Init, _ctx: &mut Context<Self>) -> Self::Result {
        for provider in [MerriamWebster, Thesaurus, Thesaurus2].iter() {
            self.providers_addr.push(Provider {
                result_addr: self.result_addr.clone(),
                logger_addr: self.logger_addr.clone(),
                requester_addr: self.requester_addr.clone(),
                main_addr: _ctx.address(),
                provider_type: *provider
            }.start());
        }
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
    system.block_on(async {
        
        let result_addr = GlobalResult {
            synonyms: HashMap::new(),
        }
        .start();
        
        
        let logger_addr = Logger {}.start();
        // let requester_addr = HttpRequester {
        //     logger_addr: logger_addr.clone()};
        let logger_addr_cloned = logger_addr.clone();
            
        let requester_addr = SyncArbiter::start(10,move || HttpRequester {
                logger_addr: logger_addr_cloned.clone()});
        
        let main_addr = Main {
            num_words: 1,
            num_processed_words: 0,
            num_providers: 3,
            result_addr: result_addr,
            requester_addr: requester_addr,
            logger_addr: logger_addr,
            providers_addr: vec![],
        }.start();

        main_addr.do_send(Init());
    });
    // TODO
    system.run().unwrap();
}
