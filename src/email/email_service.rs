use std::{env, fs};
use lettre::message::header::ContentType;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};
use lettre::message::{Attachment, header, Mailboxes, MultiPart};
use lettre::transport::smtp::{PoolConfig};
use lettre::transport::smtp::response::Response;

#[derive(Clone)]
pub struct EmailService {
    pub mailer: Option<SmtpTransport>,
}

impl EmailService {
    pub fn init(&mut self) -> Result<(), &str> {
        let smtp_host = env::var("SMTP_HOST").expect("SMTP_HOST missing on env");
        let smtp_port = env::var("SMTP_PORT").expect("SMTP_PORT missing on env");
        let smtp_username = env::var("SMTP_USERNAME").expect("SMTP_USERNAME missing on env");
        let smtp_password = env::var("SMTP_PASSWORD").expect("SMTP_PASSWORD missing on env");

        let creds = Credentials::new(smtp_username, smtp_password);

        let pool_config = PoolConfig::new()
            .min_idle(1)
            .max_size(3)
            .idle_timeout(std::time::Duration::from_secs(600));

        let mailer = SmtpTransport::relay(smtp_host.as_str())
            .unwrap()
            .pool_config(pool_config)
            .credentials(creds)
            .port(smtp_port.parse::<u16>().unwrap())
            .build();

        match mailer.test_connection() {
            Ok(_) => {
                println!("Connected to email server!");
                self.mailer = Some(mailer);
                Ok(())
            }
            Err(e) => {
                println!("Could not connect to email server: {:?}", e);
                Err("Could not connect to email server!")
            }
        }
    }

    pub fn send_file(&self, email_addresses: Vec<String>, file_path: &str) -> Result<Response, &str> {
        let content_type = self.find_file_mime(file_path)?;

        let body = match fs::read(file_path) {
            Ok(body) => body,
            _ => return Err("Could not read file!"),
        };

        let mut mailboxes = Mailboxes::new();
        for address in email_addresses {
            match address.parse() {
                Ok(address) => mailboxes.push(address),
                _ => {
                    println!("Could not parse email address: {}", address);
                    return Err("Could not parse email address!");
                }
            };
        }
        let to_header: header::To = mailboxes.into();

        let mut file_ending: Vec<&str> = file_path.split(".").collect();
        let ending = match file_ending.pop() {
            Some(ending) => ending,
            _ => return Err("Could not determine file ending!"),
        };
        let attachment = Attachment::new(format!("data.{}", &ending)).body(body, content_type);

        let email = match Message::builder()
            .from("MDclone <nobody@domain.tld>".parse().unwrap())
            .mailbox(to_header)
            .subject("Happy new year")
            .multipart(
                MultiPart::mixed()
                    .singlepart(attachment)
            )
        {
            Ok(email) => email,
            _ => return Err("Could not create email!"),
        };

        self.send_email(&email)
    }

    pub fn find_file_mime(&self, file_path: &str) -> Result<ContentType, &str> {
        let guess = match mime_guess::from_path(&file_path).first() {
            Some(mime) => mime.to_string(),
            _ => return Err("Could not determine mime type!"),
        };

        match ContentType::parse(&guess) {
            Ok(content_type) => Ok(content_type),
            _ => return Err("Could not parse mime type!"),
        }
    }

    pub fn send_email(&self, message: &Message) -> Result<Response, &str> {
        if let Some(mailer) = &self.mailer {
            match mailer.send(&message) {
                Ok(response) => Ok(response),
                Err(e) => {
                    println!("Could not send email: {:?}", e);
                    Err("Could not send email!")
                }
            }
        } else {
            Err("Could not send email!")
        }
    }
}