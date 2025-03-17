use tokio::net::{TcpListener, TcpStream};
use tokio::io::{self, AsyncReadExt, AsyncWriteExt};
use tokio::sync::mpsc;
use tokio::task;

// Function to handle an individual client connection
async fn handle_client(mut stream: TcpStream, tx: mpsc::Sender<String>) -> io::Result<()> {
    let (mut reader, mut writer) = stream.split();
    let (msg_tx, mut msg_rx) = mpsc::channel(100);[^1^][1]

    // Spawn a task to read data from the client
    let read_task = task::spawn(async move {
        let mut buffer = vec![0; 4096];
        loop {
            let n = reader.read(&mut buffer).await?;
            if n == 0 {
                break;
            }
            let received = String::from_utf8_lossy(&buffer[..n]);
            println!("Received: {}", received);
            tx.send(received.to_string()).await.unwrap();
        }
        Ok::<_, io::Error>(())
    });

    // Spawn a task to write data to the client
    let write_task = task::spawn(async move {
        while let Some(msg) = msg_rx.recv().await {
            writer.write_all(msg.as_bytes()).await?;
        }
        Ok::<_, io::Error>(())
    });

    // Read commands from stdin and send them to the client
    let stdin = io::stdin(); // Use a single instance of stdin
    let mut cmd = String::new();
    loop {
        cmd.clear();
        stdin.read_line(&mut cmd).await?;
        if cmd.trim().eq_ignore_ascii_case("exit") {
            break;
        }
        msg_tx.send(cmd.clone()).await.unwrap();
    }

    // Wait for read and write tasks to complete
    let _ = tokio::try_join!(read_task, write_task)?;
    Ok(())
}

#[tokio::main]
async fn main() -> io::Result<()> {
    // Set up a TCP listener on port 9999[^2^][2]
    let listener = TcpListener::bind("0.0.0.0:9999").await?;
    println!("[*] Listening on 0.0.0.0:9999");

    // Create a channel for sending command results
    let (tx, mut rx) = mpsc::channel(100);[^1^][1]

    // Spawn a task to print command results
    tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            println!("Command Result: {}", msg);
        }
    });

    // Accept incoming connections and spawn a task to handle each client[^3^][3]
    loop {
        let (stream, addr) = listener.accept().await?;[^3^][3]
        println!("[*] Accepted connection from {}", addr);

        let tx_clone = tx.clone();[^4^][4]
        tokio::spawn(async move {
            if let Err(e) = handle_client(stream, tx_clone).await {[^4^][4]
                eprintln!("Error handling client: {}", e);
            }
        });
    }
}
