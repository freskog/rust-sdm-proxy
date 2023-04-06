use gstreamer::prelude::*;
use gstreamer::{Element, ElementFactory, Pipeline};
use gstreamer::glib::Value;
use std::time::{Instant};

use anyhow::{Error};

#[derive(Debug, Clone)]
pub struct PipelineBuilder {
    pipeline: Pipeline
}


impl PipelineBuilder {
    pub fn new(name: &str) -> Self {
        Self {
            pipeline: Pipeline::new(Some(name))
        }
    }

    pub fn failure(&self, _:Error) {
        self.pipeline.set_state(gstreamer::State::Null).unwrap();

        let mut elements = Vec::new();
        let mut iterator = self.pipeline.iterate_elements();

        while let Ok(Some(element)) = iterator.next() {
            elements.push(element);
        }

        for element in elements {
            self.pipeline.remove(&element).expect("Failed to remove element from pipeline");
        }
    }

    pub fn add_element(&self, factory_name: &str, name: &str) -> &Self {
        ElementFactory::make(factory_name)
            .name(name)
            .build()
            .map_err(|error| error.into())
            .and_then(|element| self.pipeline.add(&element))
            .expect("Can't create element and add it to the pipeline.");
        self
    }

    pub fn set_element_property<T: Into<Value> + std::fmt::Debug>(&self, element_name: &str, property_name: &str, value: T) -> &Self {
        self
            .get_element(element_name)
            .set_property(property_name, &value.into());
        self
    }

    pub fn on_pad_connected<F>(
        &self,
        src_element_name: &str,
        callback: F,
    ) -> &Self
    where
        F: Fn(&str) + Send + Sync + 'static,
    {
        if let Some(src_element) = self.pipeline.by_name(src_element_name) {
            src_element.connect_pad_added(move |_, src_pad| { callback(&src_pad.name()) });
        }
        self
    }

    pub fn connect_src_pad_to_static_sink_pad(&self, src_pad_name:&str, element_name:&str) -> &Self {
        let element = 
            self
                .pipeline.by_name(element_name)
                .expect(&format!("No such element in pipeline: {}", element_name));

        let mut pads = 
            element.iterate_src_pads();
        
        let src_pad = 
            pads
                .find(|pad| pad.name() == src_pad_name)
                .expect(&format!("No source pad found with name: {}", src_pad_name));
        
        let sink_pad = 
            element
                .static_pad("sink")
                .expect(&format!("No static sink pad available on: {}", element_name));

        src_pad
            .link(&sink_pad)
            .expect(&format!("Couldn't link pad: {} to {}", src_pad_name, element_name));
        
        self
    }

    pub fn unlink_and_remove_src_elements(&self, element_name: &str) -> &PipelineBuilder {
        let element = self.get_element(element_name);

        for srcpad in element.pads() {
            if let Some(sinkpad) = srcpad.peer() {

                srcpad
                    .unlink(&sinkpad)
                    .expect(&format!("Unable to unlink: {} from {}", srcpad.name(), sinkpad.name()));

                if let Some(parent_element) = sinkpad.parent() {
                    if let Ok(parent_element) = parent_element.downcast::<Element>() {
                        self.remove_element(&parent_element);
                    }
                }
            }
        }
        self
    }

    pub fn add_scheduled_callback<F>(&self, timestamp: Instant, callback: F) -> &Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        let duration = timestamp.saturating_duration_since(Instant::now());

        let main_context = glib::MainContext::default();
        main_context.spawn_local(async move {
            glib::timeout_future(duration).await;
            callback();
        });

        &self
    }
    
    fn remove_element(&self, element: &Element) {
        self
            .pipeline
            .remove(element)
            .expect(&format!("Can't remove element: {}", element.name()));
    }

    fn get_element(&self, element_name:&str) -> Element {
        self
        .pipeline
        .by_name(element_name)
        .expect(&format!("No such element")) 
    }

}
