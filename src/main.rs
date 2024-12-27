use local_ip_address::local_ip;
use nanoid::nanoid;
use services::packet_service_client::PacketServiceClient;
use services::Packet;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::Request;

pub mod services {
    tonic::include_proto!("packet");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = PacketServiceClient::connect("http://[::1]:50051").await?;
    let (tx, rx) = mpsc::channel::<Packet>(10);

    let source_ip = local_ip()?;
    println!("Client IP Address: {}", source_ip);

    tokio::spawn(async move {
        let packets = vec![
            Packet {
                packet_id: nanoid!(),
                source_ip: source_ip.to_string(),
                destination_ip: "10.100.0.163".to_string(),
                source_port: 1234,
                destination_port: 80,
                payload: vec![1, 2, 3],
                timestamp: 1701234567890,
                protocol: "TCP".to_string(),
                packet_size: 512,
                payload_entropy: 0.85,
            },
            Packet {
                packet_id: nanoid!(),
                source_ip: source_ip.to_string(),
                destination_ip: "10.100.0.163".to_string(),
                source_port: 5678,
                destination_port: 443,
                payload: vec![4, 5, 6],
                timestamp: 1701234567891,
                protocol: "UDP".to_string(),
                packet_size: 256,
                payload_entropy: 0.65,
            },
        ];

        for packet in packets {
            if tx.send(packet).await.is_err() {
                eprintln!("Receiver dropped");
                break;
            }
        }
    });

    let request_stream = ReceiverStream::new(rx);

    let mut response_stream = client
        .stream_packet(Request::new(request_stream))
        .await?
        .into_inner();

    while let Some(response) = response_stream.message().await? {
        println!(
            "Received response: packet_id={}, message={}, timestamp={}",
            response.packet_id, response.message, response.timestamp
        );
    }

    Ok(())
}
