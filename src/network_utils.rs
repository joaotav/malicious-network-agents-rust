use tokio::io::{self, AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::spawn;

/// Returns the length of `message` as a big-endian 4 bytes array.
pub fn get_msg_len(message: &str) -> [u8; 4] {
    (message.len() as u32).to_be_bytes()
}

/// Attempts to write `message` to `socket`. Returns `tokio::io:Error`` upon failure.
pub async fn send_message(message: &str, socket: &mut TcpStream) -> io::Result<()> {
    let msg_len = get_msg_len(message);

    // Send the length prefix
    socket.write_all(&msg_len).await?;

    // Send the message
    socket.write_all(message.as_bytes()).await?;

    Ok(())
}

/// Reads a message containing a length prefix from a TcpStream and returns it as usize.
pub async fn read_length_prefix(socket: &mut TcpStream) -> Result<usize, io::Error> {
    let mut buffer_length = [0u8; 4];

    // Read 4 bytes from the TcpStream
    socket
        .read_exact(&mut buffer_length)
        .await
        .expect("error: failed to read data from socket\n");

    let message_length = u32::from_be_bytes(buffer_length) as usize;
    Ok(message_length)
}

/// Allocates a buffer of size `message_length` initialized with zeros.
fn alloc_msg_buffer(message_length: usize) -> Vec<u8> {
    vec![0u8; message_length]
}

/// Reads a message from a TcpStream `socket` and returns it as a String.
pub async fn recv_message(socket: &mut TcpStream) -> Result<String, io::Error> {
    // Read the 4 bytes, message length prefix
    let message_length = read_length_prefix(socket).await?;

    // Allocate a buffer with the same length as the incoming message
    let mut buffer = alloc_msg_buffer(message_length);

    // Read the message into the buffer
    socket.read_exact(&mut buffer).await?;

    let message = String::from_utf8_lossy(&buffer[..message_length]);
    Ok(message.to_string())
}

/// Attempts to establish a connection to `address`:`port` and return the
/// connection object if successful.
pub async fn connect(address: String, port: usize) -> TcpStream {
    TcpStream::connect(format!("{}:{}", address, port,))
        .await
        .expect("error: unable to establish connection with agent")
}
