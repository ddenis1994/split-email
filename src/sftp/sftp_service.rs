use std::{env, fs};
use std::io::Write;

use std::net::TcpStream;
use ssh2::{Error, Session, Sftp};
use std::path::Path;

#[derive(Clone)]
pub struct SftpService {
    pub session: Option<Session>,
}

impl SftpService {
    pub fn dry_check(&self) -> Option<()> {
        match self.session {
            Some(_) => Some(()),
            _ => None
        }
    }

    pub fn live_check(&mut self) -> Result<(), &str> {
        if let Some(_) = self.session {
            return Ok(());
        }
        match self.connect() {
            Ok(_) => {
                if let Err(e) = self.disconnect() {
                    println!("Error: {:?}", e);
                    return Err("Couldn't disconnect");
                }
                Ok(())
            }
            Err(e) => {
                println!("Error: {:?}", e);
                Err("Couldn't connect to server")
            }
        }
    }
    fn connect(&mut self) -> Result<Sftp, &str> {
        let sftp_host = env::var("SFTP_HOST").expect("SFTP_HOST missing on env");
        let sftp_port = env::var("SFTP_PORT").expect("SFTP_PORT missing on env");
        let sftp_username = env::var("SFTP_USERNAME").expect("SFTP_USERNAME missing on env");
        let sftp_password = env::var("SFTP_PASSWORD").expect("SFTP_PASSWORD missing on env");

        if let Ok(tcp) = TcpStream::connect(format!("{}:{}", sftp_host, sftp_port)) {
            if let Ok(mut sess) = Session::new() {
                sess.set_tcp_stream(tcp);


                if let Err(e) = sess.handshake() {
                    println!("Error: {:?}", e);
                    return Err("Couldn't handshake");
                }

                if let Err(e) = sess.userauth_password(&sftp_username, &sftp_password) {
                    println!("Error: {:?}", e);
                    return Err("Couldn't authenticate");
                }

                match sess.sftp() {
                    Ok(sftp) => {
                        self.session = Some(sess);
                        Ok(sftp)
                    }
                    Err(e) => {
                        println!("Error: {:?}", e);
                        Err("Couldn't create sftp")
                    }
                }
            } else {
                println!("Couldn't create session");
                Err("Couldn't create session")
            }
        } else {
            println!("Couldn't connect to server");
            Err("Couldn't connect to server...")
        }
    }

    fn disconnect(&mut self) -> Result<(), Error> {
        if let Some(session) = &self.session {
            session.disconnect(None, "", None)?;
            self.session = None;
        }
        Ok(())
    }

    pub async fn send_file(&mut self, file_path: &str, temp_local_file: &str) -> Result<(), &str> {
        if let Ok(sftp) = self.connect() {
            if let Ok(mut remote_file) = sftp.create(Path::new(file_path)) {
                let body = match fs::read(temp_local_file) {
                    Ok(body) => body,
                    _ => return Err("Could not read file!"),
                };
                if let Err(e) = remote_file.write_all(body.as_slice()) {
                    println!("Error: {:?}", e);
                    return Err("Couldn't write file");
                }
                self.disconnect();
            }
        };


        Ok(())
    }
}