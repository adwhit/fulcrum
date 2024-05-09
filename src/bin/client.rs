use std::{
    error::Error,
    io::{Read, Write},
    net::TcpStream,
    path::{Path, PathBuf},
};

type Result<T> = std::result::Result<T, Box<dyn Error>>;

fn main() -> Result<()> {
    let mut iter = std::env::args().skip(1);
    let usage = "Usage: client <URL> <REMOTE_PATH> <SAVE_PATH>";
    let (Some(url), Some(file_path), Some(save_path)) = (iter.next(), iter.next(), iter.next())
    else {
        return Err(usage.into());
    };

    let save_path = PathBuf::from(save_path);
    if save_path.exists() {
        return Err("save path already exists".into());
    }
    if let Err(e) = run(&url, &file_path, &save_path) {
        if save_path.exists() {
            eprintln!("Error occurred, removing saved file");
            std::fs::remove_file(save_path)?;
        }
        return Err(e);
    }
    Ok(())
}

fn run(url: &str, file_path: &str, save_path: &Path) -> Result<()> {
    let mut conn = TcpStream::connect(url)?;
    let pathlen = file_path.len() as u16;
    conn.write_all(&pathlen.to_le_bytes())?;
    conn.write_all(file_path.as_bytes())?;

    let mut file_len = [0; 8];
    conn.read_exact(&mut file_len)?;
    let file_len = u64::from_le_bytes(file_len) as usize;
    let mut file = std::fs::File::create(save_path)?;

    let mut buffer = vec![0; 10 * 1024 * 1024]; // read in 10MB chunks
    let mut bytes_received = 0;
    loop {
        let nbytes = conn.read(&mut buffer)?;
        bytes_received += nbytes;
        if nbytes == 0 {
            break;
        }
        file.write_all(&buffer[..nbytes])?;
    }
    if bytes_received != file_len {
        return Err("Incorrect number of bytes transferred - file corrupt".into());
    }

    Ok(())
}
