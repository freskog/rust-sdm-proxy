use gstreamer::{prelude::*};
use gstreamer::{Element, ElementFactory, Bin};
use gstreamer::glib::Value;
use std::time::{Instant};
use anyhow::{Error};

use glib::*;


#[derive(Debug, Clone)]
pub struct BinBuilder {
    bin: Bin,
    link_to: Option<Element>
}

impl BinBuilder {
    pub fn new(name: &str) -> Self {
        Self {
            bin: Bin::new(Some(name)),
            link_to: None
        }
    }
    pub fn post_error_message(&self, msg: &str) -> &Self {
        let error_message = gstreamer::ErrorMessage::new(
            gstreamer::CoreError::Failed,
            Some(msg),
            gstreamer::Debug::new(None, None, None),
        );
    
        let message = gstreamer::Message::new_error(self.bin.upcast_ref(), error_message);
    
        let bus = self.bin.bus().expect("Bin without bus");
        bus.post(&message).expect("Failed to post error message");
    
        self
    }

    pub fn handle_bus_error<F>(&self, mut callback: F)
    where
        F: FnMut(String, String) + Send + 'static,
    {
        let bus = self.bin.bus().expect("Failed to get bus from bin");

        bus.connect_message(None, move |_, msg| {
            if let gstreamer::MessageView::Error(err) = msg.view() {
                let src = err.src().unwrap();
                let factory_name = if let Ok(element) = src.downcast::<gstreamer::Element>() {
                    if let Some(factory) = element.factory() {
                        factory.name()
                    } else {
                        "<non-gstreamer-error>".to_string()
                    }
                } else {
                    "<non-gstreamer-error>".to_string()
                };

                let error_description = err.error().message().to_string();
                callback(factory_name, error_description);
            }
        });
    }

    pub fn set_link_element(&mut self, name: &str) -> &BinHandler {
        self.link_element = self.bin.by_name(name).map(|element| element.clone());
        self
    }

    pub fn add_ghost_src_pads(&mut self) -> &Self {
        if let Some(ref link_element) = self.link_element {
            // Iterate over pad templates and handle all kinds of src pads
            for pad_template in link_element.pad_templates() {
                if pad_template.direction() == PadDirection::Src {
                    match pad_template.presence() {
                        gstreamer::PadPresence::Always => {
                            let src_pads = link_element.src_pads();
                            for src_pad in src_pads {
                                let ghost_pad = GhostPad::new(None, &src_pad).expect("Failed to create ghost pad");
                                self.bin.add_pad(&ghost_pad).expect("Failed to add ghost pad to bin");
                            }
                        }
                        gstreamer::PadPresence::Sometimes => {
                            // Handle dynamic linking
                            link_element.connect_pad_added(move |element, src_pad| {
                                let ghost_pad = GhostPad::new(None, &src_pad).expect("Failed to create ghost pad");
                                let _ = element.upcast_ref::<Bin>().add_pad(&ghost_pad);
                            });
                        }
                        gstreamer::PadPresence::Request => {
                            // Handle request pads
                            let src_pad = link_element.request_pad(&pad_template, None, None).expect("Failed to request pad");
                            let ghost_pad = GhostPad::new(None, &src_pad).expect("Failed to create ghost pad");
                            self.bin.add_pad(&ghost_pad).expect("Failed to add ghost pad to bin");
                        }
                    }
                }
            }
        } else {
            panic!("No last element to link to - are you adding a ghost pad to an empty bin?");
        }

        self
    }




    pub fn failure(&self, _:Error) {
        self.bin.set_state(gstreamer::State::Null).unwrap();

        let mut elements = Vec::new();
        let mut iterator = self.bin.iterate_elements();

        while let Ok(Some(element)) = iterator.next() {
            elements.push(element);
        }

        for element in elements {
            self.bin.remove(&element).expect("Failed to remove element from pipeline");
        }
    }

    pub fn add_element(&self, factory_name: &str, name: &str) -> &Self {
        ElementFactory::make(factory_name)
            .name(name)
            .build()
            .map_err(|error| error.into())
            .and_then(|element| self.bin.add(&element))
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
        if let Some(src_element) = self.bin.by_name(src_element_name) {
            src_element.connect_pad_added(move |_, src_pad| { callback(&src_pad.name()) });
        }
        self
    }

    pub fn connect_src_pad_to_static_sink_pad(&self, src_pad_name:&str, element_name:&str) -> &Self {
        let element = 
            self
                .bin.by_name(element_name)
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

    pub fn unlink_and_remove_src_elements(&self, element_name: &str) -> &BinBuilder {
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
            .bin
            .remove(element)
            .expect(&format!("Can't remove element: {}", element.name()));
    }

    fn get_element(&self, element_name:&str) -> Element {
        self
        .bin
        .by_name(element_name)
        .expect(&format!("No such element")) 
    }

}
