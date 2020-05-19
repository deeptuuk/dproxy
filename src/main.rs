use tokio::net::{TcpListener, TcpStream, lookup_host};
use tokio::prelude::*;
use tokio::net::tcp::ReadHalf;
use tokio::net::tcp::WriteHalf;

use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::str;


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut listener = TcpListener::bind("127.0.0.1:8080").await?;

    loop {
        let (socket, _) = listener.accept().await?;

        tokio::spawn(async move {
            match process_socket(socket).await {
                Ok(_) => return,
                Err(_) => return,
            };
        });
    }
}

async fn process_socket(mut stream: TcpStream) -> Result<(), Box<dyn std::error::Error>> {
    let mut buf:[u8; 257] = [0; 257];

    let _n = stream.read(&mut buf).await?;

    stream.write_all(&[5, 0]).await?;

    let mut buf:[u8; 1024] = [0; 1024];

    let n = stream.read(&mut buf).await?;

    if buf[1] != 0x01u8 {
        return Ok(())
    }

    let port:u16 = ((buf[n-2] as u16) << 8) + buf[n-1] as u16;

    let socket = match buf[3] {
        0x01 => {
            //println!("ipv4: {}.{}.{}.{}", buf[4], buf[5], buf[6], buf[7]);
            //SocketAddrV4::new(Ipv4Addr::new(buf[4], buf[5], buf[6], buf[7]), port)
            SocketAddr::new(IpAddr::V4(Ipv4Addr::new(buf[4], buf[5], buf[6], buf[7])), port)
        },
        0x03 => {
            let port = port.to_string();
            let num = buf[4] as usize; 
            //println!("domain: {}", str::from_utf8(&buf[5..(5+num)])?);
            let domain = String::from(str::from_utf8(&buf[5..(5+num)])?) + &":".to_string() + &String::from(port);
            //println!("full: {}", domain);
            let ips: Vec<_> = lookup_host(domain).await?.collect();
            //println!("ips: {:?}", ips);
            ips[0]
        },
           _ => return Ok(()),
    };

    stream.write_all(&[5, 0, 0, 1, 0, 0, 0, 0, 0, 0]).await?;

    let mut server_stream = TcpStream::connect(socket).await?;


    let (read_local, write_local) = stream.split();
    let (read_server, write_server) = server_stream.split();
    tokio::select! (
        r1 = process(read_local, write_server) => r1?,
        r2 = process(read_server, write_local) => r2?,
    );
    Ok(())
}

async fn process(mut mread: ReadHalf<'_>, mut mwrite: WriteHalf<'_>) -> Result<(), Box<dyn std::error::Error>> {
    io::copy(&mut mread, &mut mwrite).await?;
    Ok(())
}
