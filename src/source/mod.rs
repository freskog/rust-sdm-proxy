pub mod camera_rtsp;
pub mod concat;
pub mod restart_on_error;


use std::sync::{Arc, Mutex};

use self::camera_rtsp::CameraRtspInterpreter;
use self::concat::ConcatInterpreter;
use self::restart_on_error::RestartOnErrorInterpreter;

use crate::bin_builder::BinBuilder;

#[derive(Debug, Clone)]
pub enum Source {
    CameraRtsp     { device_id: String },
    Concat         { first: Box<Source>, second: Box<Source> },
    RestartOnError { source: Box<Source> }
}


pub struct SourceInterpreter<'a> {
    camera_rtsp:&'a dyn CameraRtspInterpreter,
    concat:&'a dyn ConcatInterpreter,
    restart_on_error:&'a dyn RestartOnErrorInterpreter
}

impl<'a> SourceInterpreter<'a> {

    pub fn new(
        camera_rtsp: &'a dyn CameraRtspInterpreter,
        concat:&'a dyn ConcatInterpreter,
        restart_on_error:&'a dyn RestartOnErrorInterpreter
    ) -> Self {
        Self { 
            camera_rtsp,
            concat,
            restart_on_error
        }
    }
    
    #[rustfmt::skip]
    pub fn interpret_stream(self: &Self, stream: &Source) -> Arc<Mutex<BinBuilder>> {    
        match stream {
            Source::CameraRtsp { device_id }                     => self.camera_rtsp.interpret(device_id),
            Source::Concat { first, second }  => self.concat.interpret(first, second),
            Source::RestartOnError { source}                => self.restart_on_error.interpret(source),
        }
    }
}