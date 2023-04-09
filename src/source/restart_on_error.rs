
use crate::{source::Source, bin_builder::BinBuilder};

use std::sync::{Arc, Mutex};

pub trait RestartOnErrorInterpreter {
    fn interpret(&self, source:&Box<Source>) -> Arc<Mutex<BinBuilder>>;
}