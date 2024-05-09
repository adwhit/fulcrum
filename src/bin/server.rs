use std::{
    error::Error,
    ffi::OsString,
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    os::unix::{ffi::OsStringExt, fs::MetadataExt},
    path::PathBuf,
};

type Result<T> = std::result::Result<T, Box<dyn Error>>;

fn main() -> Result<()> {
    let mut iter = std::env::args().skip(1);
    let Some(static_dir) = iter.next() else {
        return Err("Usage: server <STATICDIR> [HOST] [PORT]".into());
    };
    let host = iter.next().unwrap_or("localhost:3333".into());
    let port = iter.next().unwrap_or("3333".into());
    let Ok(port) = port.parse::<u16>() else {
        return Err(format!("Invalid port {port}").into());
    };
    let static_dir = PathBuf::from(static_dir).canonicalize()?;
    if !static_dir.is_dir() {
        return Err(format!("{} is not a directory", static_dir.display()).into());
    }
    serve(&host, port, static_dir)?;
    Ok(())
}

fn serve(host: &str, port: u16, static_dir: PathBuf) -> Result<()> {
    println!(
        "Serving directory {} on {host}:{port}",
        static_dir.display()
    );
    let listener = TcpListener::bind((host, port))?;
    for conn in listener.incoming() {
        match conn {
            Ok(conn) => {
                println!("Connection received");
                std::thread::spawn({
                    let static_dir = static_dir.clone();
                    move || {
                        if let Err(e) = serve_connection(conn, static_dir) {
                            eprintln!("Transfer failed: {e}");
                        }
                    }
                });
            }
            Err(e) => {
                eprintln!("Failed to connect: {e}");
            }
        }
    }
    unreachable!()
}

// first two bytes: length of file path
// next n bytes: file path

fn serve_connection(mut conn: TcpStream, static_dir: PathBuf) -> Result<()> {
    let mut msglen = [0, 0];
    conn.read_exact(&mut msglen)?;
    let msglen = u16::from_le_bytes(msglen);
    let mut raw_path: Vec<u8> = vec![0; msglen as usize];
    conn.read_exact(&mut raw_path)?;
    // Note - won't compile on windows. Handy because we are assuming utf8 paths
    let file_path = OsString::from_vec(raw_path);
    let full_path = static_dir.join(&file_path).canonicalize()?;
    if !full_path.starts_with(static_dir) {
        let p = String::from_utf8_lossy(file_path.as_encoded_bytes());
        return Err(format!("invalid path {}", p).into());
    }
    let mut file = std::fs::File::open(&full_path)?;

    println!("Transferring file '{}'", full_path.display());
    let size = file.metadata().unwrap().size(); // can this really fail?
    let mut buffer = vec![0; 10 * 1024 * 1024]; // read in 10MB chunks

    // write filesize to socket
    conn.write_all(&size.to_le_bytes())?;

    // now write the actual file
    loop {
        let nbytes = file.read(&mut buffer)?;
        if nbytes == 0 {
            break;
        }
        conn.write_all(&buffer[..nbytes])?;
    }
    println!("Transfer completed");
    Ok(())
}
