#[macro_use]
extern crate serde_derive;
use futures::prelude::*;
use libp2p::{
    identity,
    tokio_codec::{FramedRead, LinesCodec},
    NetworkBehaviour, PeerId, Swarm,
};
mod blockchain;

use blockchain::*;

use std::fs::File;
use std::io::{Read, Write};

fn process_block(new_txn_text: &str) {
    println!("Recvd in process_block {}", new_txn_text);

    let mut file = File::open("Blockchain.json").expect("Unable to open");
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .expect("Unable to read file");

    let mut bc: Vec<Block> = serde_json::from_str(&contents).unwrap();

    println!(
        " {}. Please enter transaction details and press enter",
        bc.len()
    );

    let new_txn = Transaction {
        id: String::from("1"),
        timestamp: 0,
        payload: String::from(new_txn_text),
    };
    let mut new_block = Block::new(0, vec![new_txn], &bc[bc.len() - 1]);

    Block::mine_without_iterator(&mut new_block, &PREFIX);
    bc.push(new_block);
    /*let mut file2 = OpenOptions::new()
                        .append(true)
                        .open("Blockchain.json")
                        .unwrap();
    file2.write_all(serde_json::to_string(&bc).unwrap().as_bytes());
    */
    let mut file2 = File::create("Blockchain.json").expect("Unable to write file");
    file2
        .write_all(serde_json::to_string(&bc).unwrap().as_bytes())
        .expect("Unable to write to file");

    //File::create("Blockchain.json").expect("Unable to write file");

    for block in bc.iter() {
        println!("Block for index {} is {}", block.index, block.to_json());
    }
}

fn main() {
    println!("JGD!");

    //create blockchain
    let p2p_bc: Vec<Block> = vec![Block::genesis()];

    let mut file = File::create("Blockchain.json").expect("Unable to write file");
    file.write_all(serde_json::to_string(&p2p_bc).unwrap().as_bytes())
        .expect("Unable to write to file");

    // Create a random PeerId
    let local_key = identity::Keypair::generate_ed25519();
    let local_peer_id = PeerId::from(local_key.public());
    println!("Local peer id: {:?}", local_peer_id);

    // Set up a an encrypted DNS-enabled TCP Transport over the Mplex and Yamux protocols
    let transport = libp2p::build_development_transport(local_key);

    // Create a Floodsub topic
    let floodsub_topic = libp2p::floodsub::TopicBuilder::new("chat").build();

    // We create a custom network behaviour that combines floodsub and mDNS.
    // In the future, we want to improve libp2p to make this easier to do.
    #[derive(NetworkBehaviour)]
    struct MyBehaviour<TSubstream: libp2p::tokio_io::AsyncRead + libp2p::tokio_io::AsyncWrite> {
        floodsub: libp2p::floodsub::Floodsub<TSubstream>,
        mdns: libp2p::mdns::Mdns<TSubstream>,
        //bc: Blockchain,
    }

    impl<TSubstream: libp2p::tokio_io::AsyncRead + libp2p::tokio_io::AsyncWrite>
        libp2p::swarm::NetworkBehaviourEventProcess<libp2p::mdns::MdnsEvent>
        for MyBehaviour<TSubstream>
    {
        fn inject_event(&mut self, event: libp2p::mdns::MdnsEvent) {
            match event {
                libp2p::mdns::MdnsEvent::Discovered(list) => {
                    for (peer, _) in list {
                        self.floodsub.add_node_to_partial_view(peer);
                    }
                }
                libp2p::mdns::MdnsEvent::Expired(list) => {
                    for (peer, _) in list {
                        if !self.mdns.has_node(&peer) {
                            self.floodsub.remove_node_from_partial_view(&peer);
                        }
                    }
                }
            }
        }
    }

    impl<TSubstream: libp2p::tokio_io::AsyncRead + libp2p::tokio_io::AsyncWrite>
        libp2p::swarm::NetworkBehaviourEventProcess<libp2p::floodsub::FloodsubEvent>
        for MyBehaviour<TSubstream>
    {
        // Called when `floodsub` produces an event.
        fn inject_event(&mut self, message: libp2p::floodsub::FloodsubEvent) {
            if let libp2p::floodsub::FloodsubEvent::Message(message) = message {
                println!(
                    "Received: '{:?}' from {:?}",
                    String::from_utf8_lossy(&message.data),
                    message.source
                );

                process_block(&mut String::from_utf8_lossy(&message.data));
            }
        }
    }

    // Create a Swarm to manage peers and events
    let mut swarm = {
        let mut behaviour = MyBehaviour {
            floodsub: libp2p::floodsub::Floodsub::new(local_peer_id.clone()),
            mdns: libp2p::mdns::Mdns::new().expect("Failed to create mDNS service"),
            //bc: p2p_bc,
        };

        behaviour.floodsub.subscribe(floodsub_topic.clone());
        libp2p::Swarm::new(transport, behaviour, local_peer_id)
    };

    // Reach out to another node if specified
    if let Some(to_dial) = std::env::args().nth(1) {
        let dialing = to_dial.clone();
        match to_dial.parse() {
            Ok(to_dial) => match libp2p::Swarm::dial_addr(&mut swarm, to_dial) {
                Ok(_) => println!("Dialed {:?}", dialing),
                Err(e) => println!("Dial {:?} failed: {:?}", dialing, e),
            },
            Err(err) => println!("Failed to parse address to dial: {:?}", err),
        }
    }

    // Read full lines from stdin
    let stdin = tokio_stdin_stdout::stdin(0);
    let mut framed_stdin = FramedRead::new(stdin, LinesCodec::new());

    // Listen on all interfaces and whatever port the OS assigns
    libp2p::Swarm::listen_on(&mut swarm, "/ip4/0.0.0.0/tcp/0".parse().unwrap()).unwrap();

    // Kick it off
    let mut listening = false;
    tokio::run(futures::future::poll_fn(move || -> Result<_, ()> {
        loop {
            match framed_stdin.poll().expect("Error while polling stdin") {
                Async::Ready(Some(line)) => {
                    swarm.floodsub.publish(&floodsub_topic, line.as_bytes())
                }
                Async::Ready(None) => panic!("Stdin closed"),
                Async::NotReady => break,
            };
        }

        loop {
            match swarm.poll().expect("Error while polling swarm") {
                Async::Ready(Some(_)) => {}
                Async::Ready(None) | Async::NotReady => {
                    if !listening {
                        if let Some(a) = Swarm::listeners(&swarm).next() {
                            println!("Listening on {:?}", a);
                            listening = true;
                        }
                    }
                    break;
                }
            }
        }

        Ok(Async::NotReady)
    }));
}
