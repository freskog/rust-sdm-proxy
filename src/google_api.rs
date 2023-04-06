use std::time::Instant;

use anyhow::Result;

pub trait GoogleApiClient:Sync + Send {
    fn generate_rtsp_stream(&self,device_id: &str) -> Result<RtspStreamInfo>;
    fn extend_rtsp_stream(&self, device_id: &str, stream_extension_token:String) -> Result<RtspStreamInfo>;
}

#[derive(Debug, Clone)]
pub struct RtspStreamInfo {
    pub base_rtsp_url: String,
    pub stream_token: String,
    pub stream_extension_token: String,
    pub expires_at: Instant,
}


#[derive(Debug)]
pub struct ExtendRtspStreamResponse {
    pub stream_token: String,
    pub expires_at: Instant,
    pub stream_extension_token: String,
}