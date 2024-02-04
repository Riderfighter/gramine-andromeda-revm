use std::collections::HashMap;
use std::sync::Arc;

use async_std::task;
use clap::Parser;
use ethers_core::k256::pkcs8::der::EncodeValue;
use futures::future::join_all;
use serde::{Deserialize, Serialize};
use suave_andromeda_revm::StatefulExecutor;
use tokio::{io::AsyncBufReadExt, sync::{Mutex, Notify}};
use warp::{Filter, http::Response};

mod ssl;
mod farcaster;
mod wallet;
mod error;

#[derive(Parser)]
struct Cli {
    /// The rpc endpoint to connect to
    #[arg(short, long, default_value_t = String::from("http://127.0.0.1:8545"))]
    rpc: String,
    #[arg(short, long, default_value_t = false)]
    trace: bool,
    #[arg(short, long, default_value_t = false)]
    bootstrap: bool,
}

#[tokio::main]
async fn main() {
    let cli_args = Cli::parse();
    let service = StatefulExecutor::new_with_rpc(cli_args.rpc.clone());

    // Need to clone the lock like this :shrug:
    let myservice = Arc::new(Mutex::new(service));
    let myservice2 = myservice.clone(); // clone for the reader


    let pkey_state = Arc::new(Mutex::new(None)); // Shared state initialized to None
    let notify = Arc::new(Notify::new());

    // Thread 1: Serve a TLS webserver to the world
    // Match any request and return hello world!

    // GET /
    let route_index = warp::path::end().map(move || {
        warp::reply::html(r###"
        <!DOCTYPE html><html><head><title>Kettle</title>
        <meta property="fc:frame" content="vNext">
        <meta property="fc:frame:image" content="https://173-230-135-104.k37713.xyz:5001/image.svg">
        <meta property="og:image" content="https://173-230-135-104.k37713.xyz:5001/image.svg">
        <meta property="fc:frame:button:1" content="Show My Wallet">
        <meta property="fc:frame:post_url" content="https://173-230-135-104.k37713.xyz:5001/post">
        </head></html>"###)
    });

    let route_image = warp::path!("image.svg").map(|| {
        use svg::Document;
        use svg::node::element::Path;
        use svg::node::element::path::Data;

        let data = Data::new()
            .move_to((10, 10))
            .line_by((0, 50))
            .line_by((50, 0))
            .line_by((0, -50))
            .close();

        let path = Path::new()
            .set("fill", "none")
            .set("stroke", "black")
            .set("stroke-width", 3)
            .set("d", data);

        let document = Document::new()
            .set("viewBox", (0, 0, 70, 70))
            .add(path);

        Response::builder()
            .header("Content-Type", "image/svg+xml")
            .body(document.to_string())
    });

    // POST /button
    let route_post = warp::path!("post")
        .and(warp::body::json()).map(|obj: HashMap<String, serde_json::Value>| {
        warp::reply::html(format!(r###"
        <!DOCTYPE html><html><head><title>Kettle</title>
        <meta property="fc:frame" content="vNext">
        <meta property="fc:frame:image" content="https://173-230-135-104.k37713.xyz:5001/image.svg">
        <meta property="og:image" content="https://173-230-135-104.k37713.xyz:5001/image.svg">
        <meta property="fc:frame:button:1" content="ok">
        <meta property="fc:frame:post_url" content="https://173-230-135-104.k37713.xyz:5001/post">
        </head></html>"###)) //, format!("{:?}", obj).replace(r#"""#,"&quot;")))
        /*
        let myservice = myservice2.clone();
        async move {
            let mut service = myservice.lock().await;
            match service
                .execute_command("advance", false)
                .await
            {
                Ok(res) => format!("{:?}", res),
                Err(e) => format!("{:?}", e),
            }
        }*/
    });
    let cli_args = Cli::parse();
    let service = StatefulExecutor::new_with_rpc(cli_args.rpc.clone());

    // Need to clone the lock like this :shrug:
    let myservice = Arc::new(Mutex::new(service));
    let myservice2 = myservice.clone(); // clone for the reader


    let pkey_state = Arc::new(Mutex::new(None)); // Shared state initialized to None
    let notify = Arc::new(Notify::new());

    // Thread 1: Serve a TLS webserver to the world
    // Match any request and return hello world!

    // GET /
    let route_index = warp::path::end().map(move || {
        warp::reply::html(r###"
        <!DOCTYPE html><html><head><title>Kettle</title>
        <meta property="fc:frame" content="vNext">
        <meta property="fc:frame:image" content="https://173-230-135-104.k37713.xyz:5001/image.svg">
        <meta property="og:image" content="https://173-230-135-104.k37713.xyz:5001/image.svg">
        <meta property="fc:frame:button:1" content="Show My Wallet">
        <meta property="fc:frame:post_url" content="https://173-230-135-104.k37713.xyz:5001/post">
        </head></html>"###)
    });

    let route_image = warp::path!("image.svg").map(|| {
        use svg::Document;
        use svg::node::element::Path;
        use svg::node::element::path::Data;

        let data = Data::new()
            .move_to((10, 10))
            .line_by((0, 50))
            .line_by((50, 0))
            .line_by((0, -50))
            .close();

        let path = Path::new()
            .set("fill", "none")
            .set("stroke", "black")
            .set("stroke-width", 3)
            .set("d", data);

        let document = Document::new()
            .set("viewBox", (0, 0, 70, 70))
            .add(path);

        Response::builder()
            .header("Content-Type", "image/svg+xml")
            .body(document.to_string())
    });

    // POST /button
    let route_post = warp::path!("post")
        .and(warp::body::json()).map(|obj: HashMap<String, serde_json::Value>| {
        warp::reply::html(format!(r###"
        <!DOCTYPE html><html><head><title>Kettle</title>
        <meta property="fc:frame" content="vNext">
        <meta property="fc:frame:image" content="https://173-230-135-104.k37713.xyz:5001/image.svg">
        <meta property="og:image" content="https://173-230-135-104.k37713.xyz:5001/image.svg">
        <meta property="fc:frame:button:1" content="ok">
        <meta property="fc:frame:post_url" content="https://173-230-135-104.k37713.xyz:5001/post">
        </head></html>"###)) //, format!("{:?}", obj).replace(r#"""#,"&quot;")))
        /*
        let myservice = myservice2.clone();
        async move {
            let mut service = myservice.lock().await;
            match service
                .execute_command("advance", false)
                .await
            {
                Ok(res) => format!("{:?}", res),
                Err(e) => format!("{:?}", e),
            }
        }*/
    });

    let route_qr_code = warp::path!("code").and(warp::body::json()).map(|obj: HashMap<String, >|)

    let warpserver = task::spawn(warp::serve(routes)
        .tls()
        .cert(include_bytes!("../ssl-cert.pem").to_vec())
        .key(include_bytes!("../ssl-key.pem").to_vec())
        .run(([0, 0, 0, 0], 5001)));

    let mut reader = tokio::io::BufReader::new(tokio::io::stdin());
    let task = task::spawn(async move {
        loop {
            let mut buffer = Vec::new();
            let _fut = reader.read_until(b'\n', &mut buffer).await;
            let mut service = myservice.lock().await;
            match service
                .execute_command(&String::from_utf8(buffer)
                    .expect("utf8 failed")
                    .strip_suffix("\n")
                    .expect("newlin failed"), cli_args.trace)
                .await
            {
                Ok(res) => println!("{:?}", res),
                Err(e) => println!("{:?}", e),
            }
        }
    });

    join_all(vec![warpserver, task]).await;

    /*
    Usage plan:
    1. Advance the chain
    2. If not bootstrapped, create a key and certificate request
    2a. Out of band, satisfy the certificate request
    2b. Post the certificate request on chain
    3. Support onboarding
    // If not bootstrapped:
    */

    // We support two commands: advance <block number|latest|empty(latest)> and execute <TxEnv json>

    let route_qr_code = warp::path!("code").and(warp::body::json()).map(|obj: HashMap<String, >|)

    let warpserver = task::spawn(warp::serve(routes)
        .tls()
        .cert(include_bytes!("../ssl-cert.pem").to_vec())
        .key(include_bytes!("../ssl-key.pem").to_vec())
        .run(([0, 0, 0, 0], 5001)));

    let mut reader = tokio::io::BufReader::new(tokio::io::stdin());
    let task = task::spawn(async move {
        loop {
            let mut buffer = Vec::new();
            let _fut = reader.read_until(b'\n', &mut buffer).await;
            let mut service = myservice.lock().await;
            match service
                .execute_command(&String::from_utf8(buffer)
                    .expect("utf8 failed")
                    .strip_suffix("\n")
                    .expect("newlin failed"), cli_args.trace)
                .await
            {
                Ok(res) => println!("{:?}", res),
                Err(e) => println!("{:?}", e),
            }
        }
    });

    join_all(vec![warpserver, task]).await;

    /*
    Usage plan:
    1. Advance the chain
    2. If not bootstrapped, create a key and certificate request
    2a. Out of band, satisfy the certificate request
    2b. Post the certificate request on chain
    3. Support onboarding
    // If not bootstrapped:
    */

    // We support two commands: advance <block number|latest|empty(latest)> and execute <TxEnv json>
}
