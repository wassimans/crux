mod cmd;
mod continuations;
mod http;
mod key_value;
mod platform;
mod time;

use bcs::{from_bytes, to_bytes};
pub use cmd::*;
use serde::Deserialize;
use std::sync::RwLock;

pub trait App: Default {
    type Msg;
    type Model: Default;
    type ViewModel;

    fn update(
        &self,
        msg: <Self as App>::Msg,
        model: &mut <Self as App>::Model,
        cmd: &Cmd<<Self as App>::Msg>,
    ) -> Vec<Request>;

    fn view(&self, model: &<Self as App>::Model) -> <Self as App>::ViewModel;
}

pub struct AppCore<A: App> {
    model: RwLock<A::Model>,
    cmd: Cmd<A::Msg>,
    app: A,
}

impl<A: App> PartialEq for AppCore<A> {
    fn eq(&self, _other: &Self) -> bool {
        false // Core has all kinds of interior mutability
    }
}

impl<A: App> Default for AppCore<A> {
    fn default() -> Self {
        Self {
            model: Default::default(),
            cmd: Default::default(),
            app: Default::default(),
        }
    }
}

impl<A: App> AppCore<A> {
    pub fn new() -> Self {
        Self::default()
    }

    // Direct message
    pub fn message<'de>(&self, msg: &'de [u8]) -> Vec<u8>
    where
        <A as App>::Msg: Deserialize<'de>,
    {
        let msg: <A as App>::Msg = from_bytes(msg).unwrap();

        let mut model = self.model.write().unwrap();

        let requests = self.app.update(msg, &mut model, &self.cmd);

        to_bytes(&requests).unwrap()
    }

    // Return from capability
    pub fn response<'de>(&self, res: &'de [u8]) -> Vec<u8>
    where
        <A as App>::Msg: Deserialize<'de>,
    {
        let res: Response = from_bytes(res).unwrap();

        let msg = self.cmd.resume(res);

        let mut model = self.model.write().unwrap();

        let requests = self.app.update(msg, &mut model, &self.cmd);

        to_bytes(&requests).unwrap()
    }

    pub fn view(&self) -> A::ViewModel {
        let model = self.model.read().unwrap();

        self.app.view(&model)
    }
}