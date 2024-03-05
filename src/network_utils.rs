use tokio::io::{self, AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
/// Returns the length of `data` as a big-endian 4 bytes array.
pub fn get_length(data: &[u8]) -> [u8; 4] {
    (data.len() as u32).to_be_bytes()
}

/// Attempts to write `packet` to `socket`. Returns `tokio::io:Error`` upon failure.
pub async fn send_packet(packet: &[u8], socket: &mut TcpStream) -> io::Result<()> {
    let packet_len = get_length(packet);

    // Send the length prefix
    socket.write_all(&packet_len).await?;

    // Send the packet
    socket.write_all(packet).await?;

    Ok(())
}

/// Reads a packet containing a length prefix from a TcpStream and returns it as usize.
pub async fn read_length_prefix(socket: &mut TcpStream) -> Result<usize, io::Error> {
    let mut buffer_length = [0u8; 4];

    // Read 4 bytes from the TcpStream
    socket
        .read_exact(&mut buffer_length)
        .await
        .expect("error: failed to read data from socket\n");

    let packet_length = u32::from_be_bytes(buffer_length) as usize;
    Ok(packet_length)
}

/// Allocates a buffer of size `length` initialized with zeros.
fn alloc_buffer(length: usize) -> Vec<u8> {
    vec![0u8; length]
}

/// Reads a packet from a TcpStream `socket` and returns it as a String.
pub async fn recv_packet(socket: &mut TcpStream) -> Result<Vec<u8>, io::Error> {
    // Read the 4 bytes length prefix
    let packet_length = read_length_prefix(socket).await?;

    // Allocate a buffer with the same length as the incoming packet
    let mut buffer = alloc_buffer(packet_length);

    // Read the packet into the buffer
    socket.read_exact(&mut buffer).await?;

    Ok(buffer)
}

/// Attempts to establish a connection to `address`:`port` and return the
/// connection object if successful.
pub async fn connect(address: &str, port: usize) -> Result<TcpStream, io::Error> {
    TcpStream::connect(format!("{}:{}", address, port,)).await
}
