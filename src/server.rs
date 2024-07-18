// Copyright (c) 2013-2015 Sandstorm Development Group, Inc. and contributors
// Licensed under the MIT License:
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in
// all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
// THE SOFTWARE.

use std::net::ToSocketAddrs;

use capnp::capability::Promise;
use capnp::Error;
use capnp_rpc::{pry, rpc_twoparty_capnp, RpcSystem, twoparty};
use futures::AsyncReadExt;

use crate::hello_world_capnp::hello_world;
use crate::ocs365_capnp::authenticate;
use crate::ocs365_capnp::authenticate::{AuthenticateParams, AuthenticateResults};

struct HelloWorldImpl;

impl hello_world::Server for HelloWorldImpl {
    fn say_hello(
        &mut self,
        params: hello_world::SayHelloParams,
        mut results: hello_world::SayHelloResults,
    ) -> Promise<(), ::capnp::Error> {
        let request = pry!(pry!(params.get()).get_request());
        let name = pry!(pry!(request.get_name()).to_str());
        let message = format!("Hello, {name}!");
        results.get().init_reply().set_message(message);

        Promise::ok(())
    }
}

impl authenticate::Server for HelloWorldImpl {
    fn authenticate(&mut self, params: AuthenticateParams, mut result: AuthenticateResults) -> Promise<(), Error> {
        let request = pry!(pry!(params.get()).get_auth());
        let _user_name = pry!(pry!(request.get_user_name()).to_str());
        let _pass_word = pry!(pry!(request.get_user_password()).to_str());
        println!("user {}", _user_name);
        println!("password {}", _pass_word);
        // do some authentication
        let mut out = result.get().init_result();
        out.set_description("User authenticated successfully");
        out.set_message("Message");
        out.set_result("Okay");
        out.set_code(45);
        Promise::ok(())
    }
}

pub async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = ::std::env::args().collect();
    if args.len() != 3 {
        println!("usage: {} server ADDRESS[:PORT]", args[0]);
        return Ok(());
    }

    let addr = args[2]
        .to_socket_addrs()?
        .next()
        .expect("could not parse address");

    tokio::task::LocalSet::new().run_until(async move {
        let listener = tokio::net::TcpListener::bind(&addr).await?;
        //let hello_client: hello_world::Client = capnp_rpc::new_client(HelloWorldImpl);
        let auth_client: authenticate::Client = capnp_rpc::new_client(HelloWorldImpl);

        loop {
            let (stream, _) = listener.accept().await?;
            stream.set_nodelay(true)?;
            let (reader, writer) =
                tokio_util::compat::TokioAsyncReadCompatExt::compat(stream).split();
            let network = twoparty::VatNetwork::new(
                futures::io::BufReader::new(reader),
                futures::io::BufWriter::new(writer),
                rpc_twoparty_capnp::Side::Server,
                Default::default(),
            );

            let rpc_system =
                RpcSystem::new(Box::new(network), Some(auth_client.clone().client));

            tokio::task::spawn_local(rpc_system);
        }
    })
        .await
}