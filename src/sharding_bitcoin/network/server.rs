use crate::{
    types::address::Address,
    optchain::network::{
        peer,
        message::Message,
    },
};

use rand::seq::SliceRandom;
use rand::thread_rng;


use async_dup::Arc as AsyncArc;
use futures::{
    io::{
        AsyncReadExt, 
        AsyncWriteExt,
        BufReader, 
        BufWriter,
    },
    channel::oneshot, stream::StreamExt
};
use smol::{Async, Executor};
use log::{info, trace};
use std::{
    net,
    sync::Arc,
    thread,
};


pub fn new(
    addr: std::net::SocketAddr,
    msg_sink: smol::channel::Sender<(Vec<u8>, peer::Handle)>,
    shard_id: usize,
) -> std::io::Result<(Context, Handle)> {
    let (control_signal_sender, control_signal_receiver) = smol::channel::bounded(10000);
    let handle = Handle {
        control_chan: control_signal_sender.clone(),
    };
    let ctx = Context {
        peers: std::collections::HashMap::new(),
        peers_by_shard: std::collections::HashMap::new(),
        addr,
        control_chan: control_signal_receiver,
        control_sender: control_signal_sender,
        new_msg_chan: msg_sink,
        shard_id,
    };
    Ok((ctx, handle))
}

pub struct Context {
    peers: std::collections::HashMap<std::net::SocketAddr, peer::Handle>,
    peers_by_shard: std::collections::HashMap<usize, Vec<std::net::SocketAddr>>,
    addr: std::net::SocketAddr,
    control_chan: smol::channel::Receiver<ControlSignal>,
    control_sender: smol::channel::Sender<ControlSignal>,
    new_msg_chan: smol::channel::Sender<(Vec<u8>, peer::Handle)>,
    shard_id: usize,
}

impl Context {
    /// Start a new server context.
    pub fn start(self) -> std::io::Result<()> {
        // initialize the server socket
        let listener = Async::<net::TcpListener>::bind(self.addr)?;
        info!("P2P server listening at {}", self.addr);
        let control_chan = self.control_sender.clone();
        let ex = Executor::new();
        let ex = Arc::new(ex);
        let ex_clone = ex.clone();
        ex.spawn(async move {
            self.dispatch_control(ex_clone).await.unwrap();
        })
            .detach();
        ex.spawn(async move {
            Self::listener_loop(listener, control_chan).await.unwrap();
        })
            .detach();
        thread::spawn(move || smol::block_on(ex.run(futures::future::pending::<()>())));
        return Ok(());
    }

    /// the loop that endlessly accept incoming peers
    async fn listener_loop(
        listener: Async<net::TcpListener>,
        control_chan: smol::channel::Sender<ControlSignal>,
    ) -> std::io::Result<()> {
        loop {
            let (stream, addr) = listener.accept().await?;
            control_chan
                .send(ControlSignal::GetNewPeer(stream))
                .await
                .unwrap();
            info!("Incoming peer from {}", addr);
        }
    }

    async fn dispatch_control(mut self, ex: Arc<Executor<'_>>) -> std::io::Result<()> {
        // read the next control signal
        while let Ok(ctrl) = self.control_chan.recv().await {
            match ctrl {
                ControlSignal::ConnectNewPeer(addr, result_chan) => {
                    trace!("Processing ConnectNewPeer command");
                    let handle = self.connect(&addr, ex.clone()).await;
                    result_chan.send(handle).unwrap();
                }
                ControlSignal::BroadcastMessage(msg) => {
                    trace!("Processing BroadcastMessage command");
                    let mut rng = thread_rng();

                    // Collect mutable references to the peer handles into a vector
                    let mut peer_handles: Vec<_> = self.peers.iter_mut().collect();

                    // Shuffle the order
                    peer_handles.shuffle(&mut rng);

                    // Now iterate in random order
                    for (_, hd) in peer_handles {
                        hd.write(msg.clone());
                    }
                    // for (_, hd) in self.peers.iter_mut() {
                    //     hd.write(msg.clone());
                    // }
                }
                ControlSignal::BroadcastMessageWithShard((msg, shard_id)) => {
                    trace!("Processing BroadcastMessageWithShard command");
                    let shard_id = shard_id as usize;
                    
                    // match self.peers_by_shard.get_mut(&shard_id) {
                    //     Some(mut peers) => {
                    //         for peer in peers.iter_mut() {
                    //         let mut hd = self.peers.get_mut(peer).unwrap();
                    //             hd.write(msg.clone());
                    //         }
                    //     }
                    //     None => {}
                    // }
                    match self.peers_by_shard.get_mut(&shard_id) {
                        Some(peers) => {
                            for peer in peers.iter_mut() {
                            let hd = self.peers.get_mut(peer).unwrap();
                                hd.write(msg.clone());
                            }
                        }
                        None => {}
                    }
                    
                }
                ControlSignal::GetNewPeer(stream) => {
                    trace!("Processing GetNewPeer command");
                    self.accept(stream, ex.clone()).await?;
                }
                ControlSignal::DroppedPeer(addr) => {
                    trace!("Processing DroppedPeer({})", addr);
                    self.peers.remove(&addr);
                    info!("Peer {} disconnected", addr);
                }
                ControlSignal::SendToPeer((_receiver, _msg)) => {
                    unimplemented!()
                }
            }
        }
        return Ok(());
    }

    /// Connect to a peer, and register this peer
    async fn connect(
        &mut self,
        addr: &std::net::SocketAddr,
        ex: Arc<Executor<'_>>,
    ) -> std::io::Result<peer::Handle> {
        info!("Establishing connection to peer {}", addr);
        let mut stream = Async::<std::net::TcpStream>::connect(addr.clone()).await?;
        
        //send the shard id to the outgoing peer
        let shard_id = self.shard_id as u32;
        let serialized_shard_id = shard_id.to_be_bytes();
        stream.write_all(&serialized_shard_id).await?; 
        //receive the shard id from incoming peer
        let mut buffer = [0u8; std::mem::size_of::<u32>()];
        let _ = stream.read_exact(&mut buffer).await?;
        let received_shard_id = u32::from_be_bytes(buffer);
        info!("Connecting node's shard id: {}", received_shard_id);

        // register the new peer
        self.register(stream, peer::Direction::Outgoing, ex, received_shard_id as usize).await
    }

    async fn accept(
        &mut self,
        mut stream: Async<net::TcpStream>,
        ex: Arc<Executor<'_>>,
    ) -> std::io::Result<()> {
        //receive the shard id from incoming peer
        let mut buffer = [0u8; std::mem::size_of::<u32>()];
        let _ = stream.read_exact(&mut buffer).await?;
        let received_shard_id = u32::from_be_bytes(buffer);
        //send the shard id to the outgoing peer
        let shard_id = self.shard_id as u32;
        let serialized_shard_id = shard_id.to_be_bytes();
        stream.write_all(&serialized_shard_id).await?; 
        info!("Coming node's shard id: {}", received_shard_id);

        self.register(stream, peer::Direction::Incoming, ex, received_shard_id as usize).await?;
        Ok(())
    }

    async fn register(
        &mut self,
        stream: Async<net::TcpStream>,
        _direction: peer::Direction,
        ex: Arc<Executor<'_>>,
        shard_id: usize,
    ) -> std::io::Result<peer::Handle> {
        let (mut write_queue, handle) = peer::new(&stream)?;

        let stream = AsyncArc::new(stream);
        let new_msg_chan = self.new_msg_chan.clone();
        let handle_copy = handle.clone();
        let control_chan = self.control_sender.clone();
        let addr = stream.get_ref().peer_addr()?;

        // insert the peer handle so that we can broadcast to this guy later
        self.peers.insert(addr.clone(), handle.clone());
        // update the peers by shard map
        match self.peers_by_shard.get(&shard_id) {
            Some(old_addrs) => {
                let mut new_addrs = old_addrs.clone();
                new_addrs.push(addr);
                self.peers_by_shard.insert(shard_id, new_addrs);
            }
            None => {
                self.peers_by_shard.insert(shard_id, vec![addr.clone()]);
            }
        }
        // start the reactor for this peer
        // first, start a task that keeps reading from this guy
        let mut reader = BufReader::new(stream.clone());
        ex.spawn(async move {
            // the buffer to store the frame header, which contains the length of the frame
            let mut size_buffer: [u8; 4] = [0; 4];
            // the buffer to store the message content
            let mut msg_buffer: Vec<u8> = vec![];
            loop {
                // first, read exactly 4 bytes to get the frame header
                let msg_size = match reader.read_exact(&mut size_buffer).await {
                    Ok(_) => u32::from_be_bytes(size_buffer),
                    Err(_) => {
                        break;
                    }
                };
                // then, read exactly msg_size bytes to get the whole message
                if msg_buffer.len() < msg_size as usize {
                    msg_buffer.resize(msg_size as usize, 0);
                }
                match reader
                    .read_exact(&mut msg_buffer[0..msg_size as usize])
                    .await
                {
                    Ok(_) => {
                        let new_payload: Vec<u8> = msg_buffer[0..msg_size as usize].to_vec();
                        new_msg_chan
                            .send((new_payload, handle_copy.clone()))
                            .await
                            .unwrap();
                    }
                    Err(_) => {
                        break;
                    }
                }
            }
            // the peer is disconnected
        })
            .detach();

        // second, start a task that keeps writing to this guy
        let mut writer = BufWriter::new(stream.clone());
        ex.spawn(async move {
            loop {
                // first, get a message to write from the queue
                let new_msg = write_queue.next().await.unwrap();

                // second, encode the length of the message
                let size_buffer = (new_msg.len() as u32).to_be_bytes();

                // third, write the frame header and the payload
                match writer.write_all(&size_buffer).await {
                    Ok(_) => {}
                    Err(_) => {
                        break;
                    }
                }
                match writer.write_all(&new_msg).await {
                    Ok(_) => {}
                    Err(_) => {
                        break;
                    }
                }
                match writer.flush().await {
                    Ok(_) => {}
                    Err(_) => {
                        break;
                    }
                }
            }
            // the peer is disconnected
            control_chan
                .send(ControlSignal::DroppedPeer(addr))
                .await
                .unwrap();
        })
            .detach();

        Ok(handle)
    }
}

#[derive(Clone)]
pub struct Handle {
    control_chan: smol::channel::Sender<ControlSignal>,
}
//#[cfg(any(test,test_utilities))]
//pub struct TestReceiver{
//    control_chan: smol::channel::Receiver<ControlSignal>,
//}
//#[cfg(any(test,test_utilities))]
//impl TestReceiver {
//    pub fn recv(&self) -> Option<message::Message> {
//        let sig = smol::block_on(self.control_chan.recv()).unwrap();
//        match sig {
//            // in this test, only return broadcast msg
//            ControlSignal::BroadcastMessage(msg) => Some(msg),
//            _ => None,
//        }
//    }
//}

impl Handle {
    pub fn connect(&self, addr: std::net::SocketAddr) -> std::io::Result<peer::Handle> {
        let (sender, receiver) = oneshot::channel();
        smol::block_on(
            self.control_chan
                .send(ControlSignal::ConnectNewPeer(addr, sender)),
        )
            .unwrap();
        smol::block_on(receiver).unwrap()
    }

    pub fn broadcast(&self, msg: Message) {
        smol::block_on(self.control_chan.send(ControlSignal::BroadcastMessage(msg))).unwrap();
    }

    pub fn broadcast_with_shard(&self, msg: Message, shard_id: usize) {
        smol::block_on(self.control_chan.send(ControlSignal::BroadcastMessageWithShard((msg, shard_id)))).unwrap();
    }

    pub fn send(&self, receiver: Address, msg: Message) {
        smol::block_on(self.control_chan.send(ControlSignal::SendToPeer((receiver, msg)))).unwrap();
    }

   // #[cfg(any(test,test_utilities))]
   // pub fn new_for_test() -> (Handle, TestReceiver) {
   //     let (s,r) = smol::channel::unbounded();
   //     let h = Handle {control_chan: s};
   //     let t = TestReceiver {control_chan: r};
   //     (h,t)
   // }
}

enum ControlSignal {
    ConnectNewPeer(
        std::net::SocketAddr,
        oneshot::Sender<std::io::Result<peer::Handle>>,
    ),
    BroadcastMessage(Message),
    BroadcastMessageWithShard((Message, usize)),
    GetNewPeer(Async<net::TcpStream>),
    DroppedPeer(std::net::SocketAddr),
    SendToPeer((Address,Message)),
}
