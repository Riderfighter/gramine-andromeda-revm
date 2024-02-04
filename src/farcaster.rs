use serde::{Deserialize, Serialize};
use surf::{Request, Url};
use surf::http::Method;
use crate::error::FarcasterError;

pub(crate) const FARCASTER_HUB_URL: String = String::from("https://nemes.farcaster.xyz:2281/v1/validateMessage");

#[derive(Debug, Serialize, Deserialize)]
struct CastId {
    fid: u64,
    hash: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct FrameActionBody {
    url: String,
    buttonIndex: u64,
    castId: CastId,
}

#[derive(Debug, Serialize, Deserialize)]
struct MessageData {
    #[serde(rename = "type")]
    message_type: String,
    fid: u64,
    timestamp: u64,
    network: String,
    frameActionBody: FrameActionBody,
}

#[derive(Debug, Serialize, Deserialize)]
struct Message {
    data: MessageData,
    hash: String,
    hashScheme: String,
    signature: String,
    signer: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct FarcasterAuthMessage {
    valid: bool,
    message: Message
}

#[derive(Debug, Serialize, Deserialize)]
struct UntrustedData {
    fid: u64,
    url: String,
    messageHash: String,
    timestamp: u64,
    network: u64,
    buttonIndex: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    inputText: Option<String>,
    castId: CastId,
}

#[derive(Debug, Serialize, Deserialize)]
struct TrustedData {
    messageBytes: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct FarcasterUntrustedAuthMessage {
    untrustedData: UntrustedData,
    trustedData: TrustedData,
}

/// Given the Farcaster Untrusted Auth Message, we will contact the Farcaster Hub, verify the UntrustedAuthMessage, and we return the verified User ID.
pub(crate) async fn verify_user_auth(user_auth: FarcasterUntrustedAuthMessage) -> Result<String> {
    // I'm literally praying to the blockchain gods that this is the message bytes that you're meant to give to hub service...
    let opaque_message_hex = user_auth.trustedData.messageBytes;

    let url = Url::parse(FARCASTER_HUB_URL.as_str()).expect("Not a real URL, check the Farcaster hub URL used.");

    let mut request = Request::new(Method::Post, url);
    request.set_body(surf::Body::from(opaque_message_hex));
    request.set_header("Content-Type", "application/octet-stream");

    let mut response = surf::client().send(request).await.expect("Failed to reach the Farcaster hub.");

    if response.status().is_success() {
        let resp_body: FarcasterAuthMessage = response.body_json();

        if !resp_body.valid {
            return Err(FarcasterError::new("Invalid Farcaster message."))
        }

        let user_id = resp_body.message.data.fid;

    } else {
        return Err(FarcasterError::new("Unsuccessful Farcaster request."))
    }

    Ok(String::new())

}