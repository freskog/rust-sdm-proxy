

use crate::bin_builder::BinBuilder;
use crate::google_api::*;

use std::{time::{Duration}, sync::{Arc, Mutex}};

pub trait CameraRtspInterpreter {
    fn interpret(&self, device_id:&str) -> Arc<Mutex<BinBuilder>>;
}

pub struct CameraRtspInterpreterImpl {
    google_api: &'static dyn GoogleApiClient
}

impl CameraRtspInterpreter for CameraRtspInterpreterImpl {
    fn interpret(&self, device_id:&str) -> Arc<Mutex<BinBuilder>> {
        CameraRtspInterpreterImpl::camera_rtsp_stream(self.google_api, device_id)
    }
}

impl CameraRtspInterpreterImpl {
    

    fn camera_rtsp_stream(google_api: &'static dyn GoogleApiClient, device_id: &str) -> Arc<Mutex<BinBuilder>> {
        let rtsp_info = 
            google_api
                .generate_rtsp_stream(device_id)
                .expect(&format!("Can't generate rtsp stream for device_id: {}", device_id));

        let pipeline_builder_original = 
            Arc::new(Mutex::new(BinBuilder::new(&format!("RTSP Camera: {}", device_id))));

        let pipeline_builder = pipeline_builder_original.clone();
        let url = format!("{}?auth={}", rtsp_info.base_rtsp_url, rtsp_info.stream_token);

        pipeline_builder
            .clone()
            .lock()
            .unwrap()
            .add_element("rtspsrc", "rtspsrc")
            .add_element("decodebin", "decodebin")
            .set_element_property("rtspsrc", "location", &url)
            .on_pad_connected("rtspsrc", {
                let device_id = device_id.to_string();
                
                move |src_pad_name,| {

                    pipeline_builder
                        .clone()
                        .lock()
                        .unwrap()
                        .connect_src_pad_to_static_sink_pad(src_pad_name, "decodebin");

                    CameraRtspInterpreterImpl::handle_rtsp_stream_extension(
                        pipeline_builder.clone(),
                        &device_id,
                        rtsp_info.clone(),
                        google_api.clone()
                    )
                }
            });

        pipeline_builder_original
    }

    fn handle_rtsp_stream_extension(
        pipeline_builder: Arc<Mutex<BinBuilder>>,
        device_id: &str,
        rtsp_info: RtspStreamInfo,
        google_api: &'static dyn GoogleApiClient
    ) {
        let callback_time = rtsp_info.expires_at - Duration::from_secs(30);

        pipeline_builder
            .clone()
            .lock()
            .unwrap()
            .add_scheduled_callback(callback_time, {

            let device_id = device_id.to_string();
            let stream_extension_token = rtsp_info.stream_extension_token.clone();

            move || {
                                
                match google_api.extend_rtsp_stream(&device_id, stream_extension_token.clone()) {
                    Ok(new_rtsp_info) => {

                        CameraRtspInterpreterImpl::handle_rtsp_stream_switch(
                                pipeline_builder.clone(),
                                rtsp_info.clone(),
                        );

                        CameraRtspInterpreterImpl::handle_rtsp_stream_extension(
                            pipeline_builder.clone(),
                            &device_id,
                            new_rtsp_info,
                            google_api);
                    },
                    Err(err) => {
                        ()
                    }
                } 

            }
            
        });
    }

    fn handle_rtsp_stream_switch(
        pipeline_builder: Arc<Mutex<BinBuilder>>,
        new_rtsp_info: RtspStreamInfo,
    ) {
        
        let new_url = format!("{}?auth={}",new_rtsp_info.base_rtsp_url, &new_rtsp_info.stream_token);
        
        let new_rtspsrc_name = format!("rtspsrc{}", new_rtsp_info.stream_extension_token);
        
        pipeline_builder
            .clone()
            .lock()
            .unwrap()
            .add_element("rtspsrc", &new_rtspsrc_name)
            .set_element_property(&new_rtspsrc_name, "location", &new_url)
            .on_pad_connected(
            &new_rtspsrc_name, 
            move |new_src_pad:&str| {
  
                pipeline_builder
                    .lock()
                    .unwrap()
                    .unlink_and_remove_src_elements("decodebin")
                    .connect_src_pad_to_static_sink_pad(new_src_pad, "decodebin");
            }
        );
    
    }


}
