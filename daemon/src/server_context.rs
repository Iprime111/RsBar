use rayon::{iter::ParallelIterator, prelude::IntoParallelRefMutIterator};

const MIN_REQUEST_PARTS:   usize = 3;
const QUERY_REQUEST_PARTS: usize = 3;
const CALL_REQUEST_PARTS:  usize = 4;

const QUERY_SPECIFIER: &str = "q";
const CALL_SPECIFIER:  &str = "c";


pub trait RsbarContextContent {
    fn init(&mut self);
    fn update(&mut self);

    // Query args format:     "q/<context name>/<parameter name>"
    // Query responce format: "<parameter content>" or None in case of error
    fn query(&self, parameter: &str) -> Option<String>;

    // Call args format:   "c/<context name>/<procedure name>/<arg string>"
    // Call result format: "<return value>" or None in case of error
    fn call(&mut self, procedure: &str, args: &str) -> Option<String>;
}

pub struct RsbarContext {
    name:    String,
    context: Box<dyn RsbarContextContent + Send>,
}

impl RsbarContext {
    pub fn new(name: &str, context: Box<dyn RsbarContextContent + Send>) -> Self {
        RsbarContext {
            name: String::from(name),
            context,
        }
    }
}

pub struct ServerContext {
    contexts: Vec<RsbarContext>,
}

impl ServerContext {
    pub fn new() -> ServerContext {
        ServerContext { contexts: Vec::new(), }
    }

    pub fn new_request(&mut self, request: &str) -> Option<String> {
        let request_parts: Vec<&str> = request.split('/').collect();
        let parts_count = request_parts.len();

        if parts_count < MIN_REQUEST_PARTS {
            return None;
        }
        
        for context in self.contexts.iter_mut() {
            if (*context).name == request_parts[1] {
                
                return match request_parts[0] {
                    QUERY_SPECIFIER => {
                        if parts_count != QUERY_REQUEST_PARTS {
                            return None;
                        }

                        (*context).context.query(request_parts[2])
                    },
                    CALL_SPECIFIER => {
                        if parts_count != CALL_REQUEST_PARTS {
                            return None;
                        }

                        (*context).context.call(request_parts[2], request_parts[3])
                    },
                    _ => None,
                }
            }
        }

        return None;
    }

    pub fn add_context(&mut self, context: RsbarContext) {

        self.contexts.push(context);
    }

    pub fn update(&mut self) {
        self.contexts.par_iter_mut().for_each(|context| {
            context.context.update();
        })
    }
}
