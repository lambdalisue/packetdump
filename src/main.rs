use std::sync::mpsc;
use std::thread;
use std::time::SystemTime;

use pnet::datalink;
use pnet::datalink::Channel::Ethernet;
use pnet::datalink::NetworkInterface;
use pnet::packet::ethernet::EthernetPacket;

fn handle_ethernet_frame(
    interface: &NetworkInterface,
    time: &SystemTime,
    ethernet: &EthernetPacket,
) {
    println!("{}: {:?}: {:?}", interface.name, time, ethernet,);
}

fn main() {
    let (tx, rx) = mpsc::channel();
    let mut handles: Vec<_> = datalink::interfaces()
        .into_iter()
        .filter(|iface| !iface.mac_address().is_zero())
        .map(|iface| {
            let tx = tx.clone();
            thread::spawn(move || {
                let mut rx = match datalink::channel(&iface, Default::default()) {
                    Ok(Ethernet(_, rx)) => rx,
                    Ok(_) => panic!("unhandled channel type"),
                    Err(e) => panic!("An error occured when creating the datalink channel: {}", e),
                };
                loop {
                    match rx.next() {
                        Ok(packet) => tx
                            .send((iface.clone(), SystemTime::now(), packet.to_owned()))
                            .unwrap(),
                        Err(_) => continue,
                    }
                }
            })
        })
        .collect();
    handles.push(thread::spawn(move || loop {
        match rx.recv() {
            Ok((iface, time, packet)) => match EthernetPacket::new(&packet) {
                Some(packet) => handle_ethernet_frame(&iface, &time, &packet),
                _ => continue,
            },
            Err(_) => continue,
        }
    }));

    for h in handles {
        h.join().unwrap();
    }
}
