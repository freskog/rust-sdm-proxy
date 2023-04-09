use std::sync::{Arc, Mutex};

use crate::bin_builder::BinBuilder;


use crate::source::Source;

pub trait ConcatInterpreter {
    fn interpret(&self, first:&Box<Source>, second:&Box<Source>) -> Arc<Mutex<BinBuilder>>;
}