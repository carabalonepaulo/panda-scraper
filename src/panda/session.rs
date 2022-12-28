use std::rc::Rc;

use fantoccini::Client;

use crate::panda::Result;

#[derive(Debug)]
pub struct Session {
    browser: Rc<Client>,
}

impl Session {
    pub async fn new(browser: Rc<Client>) -> Result<Self> {
        Ok(Self { browser })
    }
    async fn library(&self) {}
}
