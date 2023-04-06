use std::sync::{Arc, Mutex};

use crate::pipeline_builder::PipelineBuilder;


use crate::source::Source;

pub trait ConcatInterpreter {
    fn interpret(&self, first:&Box<Source>, second:&Box<Source>) -> Arc<Mutex<PipelineBuilder>>;
}