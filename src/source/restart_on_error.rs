
use crate::{source::Source, pipeline_builder::PipelineBuilder};

use std::sync::{Arc, Mutex};

pub trait RestartOnErrorInterpreter {
    fn interpret(&self, source:&Box<Source>) -> Arc<Mutex<PipelineBuilder>>;
}