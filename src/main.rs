use clap::Parser;
use acme_lib::{Result,create_rsa_key};
use suave_andromeda_revm::{StatefulExecutor,sgx_precompiles};
use openssl::x509::{X509Req, X509ReqBuilder};
use openssl::x509::extension::SubjectAlternativeName;
use openssl::pkey::{self, PKey};
use openssl::hash::MessageDigest;
use openssl::stack::Stack;
use openssl::symm::{decrypt, encrypt, Cipher};
use warp::{http::Response,Filter};
use std::{sync::{Arc}};
use tokio::{io::{AsyncBufReadExt},sync::{Mutex,Notify}};
use futures::future::join_all;
use async_std::{task};
use revm::primitives::{Address,Env,Output,ExecutionResult};
use revm::precompile::{Precompile};
use std::collections::HashMap;
use ethers_core::abi::{decode,ParamType};

pub fn create_csr(pkey: &PKey<pkey::Private>) -> Result<X509Req> {
    //
    // the csr builder
    let mut req_bld = X509ReqBuilder::new().expect("X509ReqBuilder");

    let mut x509_name = openssl::x509::X509NameBuilder::new().unwrap();
    x509_name.append_entry_by_text("C", "US").unwrap();
    x509_name.append_entry_by_text("ST", "IL").unwrap();
    x509_name.append_entry_by_text("O", "n/a").unwrap();
    x509_name.append_entry_by_text("CN", "*.k37713.xyz").unwrap();
    let x509_name = x509_name.build();

    req_bld.set_subject_name(&x509_name).unwrap();
    

    // set private/public key in builder
    req_bld.set_pubkey(pkey).expect("set_pubkey");

    // set all domains as alt names
    let mut stack = Stack::new().expect("Stack::new");
    let ctx = req_bld.x509v3_context(None);
    let mut an = SubjectAlternativeName::new();
    an.dns("*.k37713.xyz");

    let ext = an.build(&ctx).expect("SubjectAlternativeName::build");
    stack.push(ext).expect("Stack::push");
    req_bld.add_extensions(&stack).expect("add_extensions");

    // sign it
    req_bld
        .sign(pkey, MessageDigest::sha256())
        .expect("csr_sign");

    // the csr
    Ok(req_bld.build())
}

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

	let t = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)
	    .unwrap().as_secs();

 	warp::reply::html(r###"
        <!DOCTYPE html><html><head><title>Kettle</title>
        <meta property="fc:frame" content="vNext">
        <meta property="fc:frame:image" content="https://www.star-facts.com/wp-content/uploads/2020/08/Rigil-Kentaurus.webp">
        <meta property="og:image" content="https://www.star-facts.com/wp-content/uploads/2020/08/Rigil-Kentaurus.webp">
        <meta property="fc:frame:button:1" content="Refresh">
        <meta property="fc:frame:post_url" content="https://173-230-135-104-00002.k37713.xyz:5001/post">
        </head><body>
       <img width=200 src="https://www.star-facts.com/wp-content/uploads/2020/08/Rigil-Kentaurus.webp" /><br>
       <input type="button" value="Refresh" onClick="window.location.reload()"><br>
       This page is served by an SGX node. You too can join an SGX node to this network, and serve the same page yourself... including passing the HTTPS security check.
       </body></html>"###)
    });

    use std::time::SystemTime;
    // POST /button
    let route_post = warp::path!("post")
    //	.and(warp::body::json()).map(|obj: HashMap<String, serde_json::Value>| {
	.then(move || {
	    let myservice = myservice2.clone();
	    async move {
		let cmd = r#"execute {"caller":"0x0000000000000000000000000000000000000000","gas_limit":21000000,"gas_price":"0x0","transact_to":{"Call":"0xa3B3F75f05e8A1A2Ed31C65B6bA2339D7050cAfe"},"value":"0x0","data":"0x7a5b4f59","nonce":0,"chain_id":null,"access_list":[],"gas_priority_fee":null,"blob_hashes":[],"max_fee_per_blob_gas":null}"#;
		
		let mut service = myservice.lock().await;
		let _ = service
		    .execute_command("advance", false)
		.await;
		let res = service
		    .execute_command(cmd, false)
		    .await.unwrap();
		let res : ExecutionResult = serde_json::from_str(&res).unwrap();
		let output = res.output().unwrap();
		
		let x = decode(&[ParamType::Tuple(vec![
		    ParamType::Uint(256),
		    ParamType::Uint(256),
		    ParamType::FixedBytes(32)])],output).unwrap();
		let x = x[0].clone().into_tuple().unwrap();
		let blockheight = x[0].clone().into_uint().unwrap();
		let timestamp = x[1].clone().into_uint().unwrap();
		let _blockhash = x[2].clone().into_fixed_bytes().unwrap();
		
 		warp::reply::html(format!(r###"
        <!DOCTYPE html><html><head><title>Kettle</title>
        <meta property="fc:frame" content="vNext">
        <meta property="fc:frame:image" content="https://www.star-facts.com/wp-content/uploads/2020/08/Rigil-Kentaurus.webp">
        <meta property="og:image" content="https://www.star-facts.com/wp-content/uploads/2020/08/Rigil-Kentaurus.webp">
        <meta property="fc:frame:button:1" content="Height: {}">
        <meta property="fc:frame:button:2" content="Time: {}">
        <meta property="fc:frame:button:3" content="Refresh">
        <meta property="fc:frame:post_url" content="https://173-230-135-104-00002.k37713.xyz:5001/post">
        </head></html>"###, blockheight, timestamp))
	    }
	});

    let route_gets = warp::any().and(route_index);
    let routes = route_gets.or(route_post);

    let shared_state_for_reader = pkey_state.clone();
    let notify_for_reader = notify.clone();
    let warpserver = task::spawn(async move {
	notify_for_reader.notified().await;
	let (pkey,cert) : (Vec<u8>,Vec<u8>) = shared_state_for_reader.lock().await.clone().unwrap();
	warp::serve(routes)
            .tls()
	    .cert(cert)
	    .key(pkey)
	    .run(([0, 0, 0, 0], 5001)).await
    });

    // Thread 2: Read local commands on stdin
    let mut reader = tokio::io::BufReader::new(tokio::io::stdin());
    let task = task::spawn(async move {
	let mut privkey : [u8; 16] = [0; 16];
	let iv = b"\x00\x01\x02\x03\x04\x05\x06\x07\x00\x01\x02\x03\x04\x05\x06\x07";
	
	loop {
	    let mut buffer = Vec::new();
	    let _fut = reader.read_until(b'\n', &mut buffer).await;
	    let line = String::from_utf8(buffer).unwrap().strip_suffix("\n").unwrap().to_owned();
            let (command, args) = match line.split_once(' ') {
		Some((command, args)) => (command, Some(args)),
		None => (line.as_str(), None),
            };
	    
	    match command
	    {
		// Call "Loadup" with the address of the Frame contract
		"loadup" => {
		    /* Fetch the privkey from the volatileGet. 
		    - The "caller" should be the Frame contract, passed as arg 
		    - The tag is just "priv" */
		    let mut buf = [0; 20];
		    hex::decode_to_slice(&args.unwrap()[2..], &mut buf as &mut [u8]).unwrap();
		    let frame_addr = Address::from_slice(&buf);
		    let mut input_data = [0; 32];
		    input_data[0..4].copy_from_slice(b"priv");
		    
		    let mut env = Env::default();
		    env.msg.caller = frame_addr;

		    // The hashmap itself will lock, so no lock needed her
		    let (_addr, prec) = sgx_precompiles().inner[2].to_owned().into();
		    let (_gas,out) = match prec {
			Precompile::Env(fun) => fun(&input_data, 21000000, &env),
			_ => panic!("blah")
		    }.unwrap();
		    privkey.copy_from_slice(&out[..16]);
		    println!("Output: {:?} {:?}", frame_addr, &hex::encode(&out));
		}

		"bootstrap" => {
		    /* For bootstrapping, 
			we will encrypt the Key, and output the CSR */
		    //let pkey = create_rsa_key(2048);
		    //let csr = create_csr(&pkey);
		    //let pkey_b = pkey.rsa().unwrap().private_key_to_pem().unwrap();
		    //let csr_b = csr.unwrap().to_pem().unwrap();
		    let pkey_b = include_bytes!("../ssl-key.pem");
		    let csr_b = b"" as &[u8];
		    let cipher = Cipher::aes_128_cbc();
		    let ciphertext = encrypt(
			cipher,
			&privkey as &[u8],
			Some(iv),
			pkey_b).unwrap();
		    println!("{:?} {:?}",
			     hex::encode(ciphertext),
			     String::from_utf8(csr_b.to_vec()).unwrap());
		}

		"serve" => {
		    // To start serving, we need to pass in the encrypted key and the cert
		    let (enckey, cert) = args.unwrap().split_once(' ').unwrap();
		    let cipher = Cipher::aes_128_cbc();
		    let pkey = decrypt(
			cipher,
			&privkey as &[u8],
			Some(iv),
			&hex::decode(enckey).unwrap()).unwrap();
		    let mut p = pkey_state.lock().await;
		    *p = Some((pkey.clone(), hex::decode(cert).unwrap()));
		    notify.notify_one();
		    println!("serving");
		}
		
		// Anything else passes through to the Kettle
		_ => {
		    let mut service = myservice.lock().await;
		    match service
			.execute_command(&line, false)
			.await
		    {
			Ok(res) => println!("{:?}", res),
			Err(e) => println!("{:?}", e),
		    }
		}
	    }
	}
    });
			   
    join_all(vec![warpserver, task]).await;
}
