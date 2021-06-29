extern crate actix;
use self::actix::{AsyncContext, SyncArbiter, SyncContext};
use crate::synonym::helpers::file_parser;
use crate::synonym::helpers::http_requester;
use crate::synonym::logger::file_logger;
use crate::synonym::providers::base::Provider::{MerriamWebster, Thesaurus, Thesaurus2};
use crate::synonym::providers::{base, merriam_webster, thesaurus, thesaurus2};
use actix::ResponseFuture;
use actix::{Actor, Addr, Context, Handler, Message, System, WrapFuture};
use std::collections::HashMap;
use std::time::Duration;

#[derive(Message)]
#[rtype(result = "()")]
struct FindSynonyms(String);

#[derive(Message)]
#[rtype(result = "()")]
struct Synonyms {
    word: String,
    synonyms: Vec<String>,
}

#[derive(Message)]
#[rtype(result = "Result<(), ()>")]
struct Finish;

#[derive(Message)]
#[rtype(result = "()")]
struct Init {
    filename: String,
    min_wait: u64,
}

#[derive(Message)]
#[rtype(result = "()")]
struct ProviderFinished();

#[derive(Message)]
#[rtype(result = "()")]
struct SendRequest {
    url: String,
    word: String,
    provider_addr: Addr<Provider>,
}

#[derive(Message)]
#[rtype(result = "()")]
struct RequestResult {
    response: String,
    word: String,
}

#[derive(Message)]
#[rtype(result = "()")]
struct Log {
    content: String,
}

struct Logger {}

impl Actor for Logger {
    type Context = SyncContext<Self>;
}

impl Handler<Log> for Logger {
    type Result = ();

    fn handle(&mut self, msg: Log, _ctx: &mut Self::Context) -> Self::Result {
        let content: &str = &*("[actors] ".to_owned() + &*msg.content);
        file_logger::log(content);
    }
}

struct ProviderCoordinator {
    id: usize,
    requester_addr: Addr<HttpRequester>,
    min_wait_millis_between_requests: u64,
    logger_addr: Addr<Logger>,
}

impl Actor for ProviderCoordinator {
    type Context = Context<Self>;
}

struct Provider {
    result_addr: Addr<GlobalResult>,
    logger_addr: Addr<Logger>,
    main_addr: Addr<Main>,
    coordinator_addr: Addr<ProviderCoordinator>,
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

impl Handler<SendRequest> for ProviderCoordinator {
    type Result = ();

    fn handle(&mut self, msg: SendRequest, ctx: &mut Self::Context) -> Self::Result {
        let cloned_word = msg.word.clone();
        self.logger_addr.do_send(Log {
            content: format!(
                "Handle SendRequest on ProviderCoordinator ID = {} - URL = {} - Word = {}",
                self.id, msg.url, cloned_word
            ),
        });
        let min_wait_millis = Duration::from_millis(self.min_wait_millis_between_requests);
        ctx.wait(actix::clock::sleep(min_wait_millis).into_actor(self));

        self.requester_addr.do_send(msg);
    }
}

impl Handler<SendRequest> for HttpRequester {
    type Result = ();

    fn handle(&mut self, msg: SendRequest, _ctx: &mut Self::Context) -> Self::Result {
        self.logger_addr.do_send(Log {
            content: format!(
                "Handle SendRequest for word {} and URL {} on HttpRequester",
                msg.word, msg.url
            ),
        });
        let cloned_word = msg.word.clone();
        let cloned_url = msg.url.clone();
        let raw_response = http_requester::fetch_synonyms_raw_response(msg.word, msg.url);
        match raw_response {
            Err(error) => {
                panic!("{}", error);
            }
            Ok(result) => {
                self.logger_addr.do_send(Log {
                    content: format!(
                        "Sending RequestResult for synonyms of {} from URL {}",
                        cloned_word.clone(),
                        cloned_url
                    ),
                });
                msg.provider_addr.do_send(RequestResult {
                    response: result,
                    word: cloned_word,
                });
            }
        }
    }
}

impl Handler<RequestResult> for Provider {
    type Result = ();
    fn handle(&mut self, _msg: RequestResult, _ctx: &mut Context<Self>) -> Self::Result {
        let result = _msg.response;
        match match self.provider_type {
            base::Provider::Thesaurus => thesaurus::raw_response_to_synonyms(result),
            base::Provider::MerriamWebster => merriam_webster::raw_response_to_synonyms(result),
            base::Provider::Thesaurus2 => thesaurus2::raw_response_to_synonyms(result),
        } {
            Ok(synonyms) => {
                self.result_addr.do_send(Synonyms {
                    word: _msg.word.clone(),
                    synonyms,
                });
                self.main_addr.do_send(ProviderFinished());
            }
            Err(error) => {
                panic!("{}", error);
            }
        }
    }
}

impl Handler<FindSynonyms> for Provider {
    type Result = ();

    fn handle(&mut self, _msg: FindSynonyms, _ctx: &mut Context<Self>) -> Self::Result {
        let word = _msg.0;
        self.logger_addr.do_send(Log {
            content: format!(
                "Handle FindSynonyms on Provider {} for word {}",
                self.provider_type, word
            ),
        });

        let base_url = match self.provider_type {
            base::Provider::Thesaurus => "https://thesaurus.yourdictionary.com/",
            base::Provider::MerriamWebster => "https://www.merriam-webster.com/thesaurus/",
            base::Provider::Thesaurus2 => "https://www.thesaurus.com/browse/",
        }
        .to_string();

        self.coordinator_addr.do_send(SendRequest {
            word: word.clone(),
            url: base_url.clone(),
            provider_addr: _ctx.address(),
        });
    }
}

struct GlobalResult {
    synonyms: HashMap<String, HashMap<String, usize>>,
    logger_addr: Addr<Logger>,
}

impl Actor for GlobalResult {
    type Context = Context<Self>;
}

impl Handler<Synonyms> for GlobalResult {
    type Result = ();

    fn handle(&mut self, _msg: Synonyms, _ctx: &mut Context<Self>) -> Self::Result {
        let word_partial_result = self.synonyms.entry(_msg.word).or_insert(HashMap::new());

        for synonym in _msg.synonyms {
            let synonym_word_count = word_partial_result.entry(synonym).or_insert(0);
            *synonym_word_count += 1;
        }
    }
}

impl Handler<Finish> for GlobalResult {
    type Result = ResponseFuture<Result<(), ()>>;

    fn handle(&mut self, _msg: Finish, _ctx: &mut Context<Self>) -> Self::Result {
        self.logger_addr.do_send(Log {
            content: format!("Final result: {:?}", self.synonyms),
        });
        // Wait for last sent message to finish before stoping actix arbitrer
        let fut = self.logger_addr.send(Log {
            content: format!("GlobalResult actor received Finish message. Terminating..."),
        });

        Box::pin(async move {
            match fut.await {
                Ok(_) => {
                    System::current().stop();
                    Ok(())
                }
                Err(e) => {
                    println!("{:?}", e);
                    System::current().stop();
                    Err(())
                }
            }
        })
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
        let mut coordinator_id = 0;
        for provider in [MerriamWebster, Thesaurus, Thesaurus2].iter() {
            let coordinator_addr = ProviderCoordinator {
                id: coordinator_id,
                requester_addr: self.requester_addr.clone(),
                min_wait_millis_between_requests: _msg.min_wait,
                logger_addr: self.logger_addr.clone(),
            }
            .start();
            self.providers_addr.push(
                Provider {
                    result_addr: self.result_addr.clone(),
                    logger_addr: self.logger_addr.clone(),
                    main_addr: _ctx.address(),
                    coordinator_addr: coordinator_addr.clone(),
                    provider_type: *provider,
                }
                .start(),
            );

            coordinator_id += 1;
        }
        self.num_providers = self.providers_addr.len();

        let cloned_filename = _msg.filename.clone();
        let lines = file_parser::read_lines(cloned_filename);
        match lines {
            Ok(lines) => {
                for word in lines {
                    match word {
                        Ok(word) => {
                            self.num_words += 1;
                            for provider in self.providers_addr.iter() {
                                provider.do_send(FindSynonyms(word.to_string()));
                            }
                        }
                        Err(error) => {
                            panic!("{}", error)
                        }
                    };
                }
            }
            Err(error) => {
                panic!("Couldn't read {}, error: {}", _msg.filename, error)
            }
        }
    }
}

impl Handler<ProviderFinished> for Main {
    type Result = ();

    fn handle(&mut self, _msg: ProviderFinished, _ctx: &mut Context<Self>) -> Self::Result {
        self.num_processed_words += 1;

        if self.num_words * self.num_providers == self.num_processed_words {
            self.result_addr.do_send(Finish); // Condicion de corte
        }
    }
}

pub fn start_actors(filename: &String, max_requests: usize, min_wait_millis: u64) {
    file_logger::log("Actors solution starting");
    let system = System::new(); // Creates an Arbitrer where the actors will be running on
    system.block_on(async {
        // With SyncArbitrer, these actors will have it's different arbitrer and consequently
        // run in separate threads
        let logger_addr = SyncArbiter::start(1, move || Logger {});
        let logger_addr_cloned = logger_addr.clone();

        let requester_addr = SyncArbiter::start(max_requests, move || HttpRequester {
            logger_addr: logger_addr_cloned.clone(),
        });

        let result_addr = GlobalResult {
            synonyms: HashMap::new(),
            logger_addr: logger_addr.clone(),
        }
        .start();

        let main_addr = Main {
            num_words: 0,
            num_processed_words: 0,
            num_providers: 0,
            result_addr,
            requester_addr,
            logger_addr,
            providers_addr: vec![],
        }
        .start();

        main_addr.do_send(Init {
            filename: filename.to_string(),
            min_wait: min_wait_millis,
        });
    });

    system.run().unwrap();
}
