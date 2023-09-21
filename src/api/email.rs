use std::{env, fs};
use lettre::message::header::ContentType;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};
use lettre::message::{Attachment, header, Mailboxes, MultiPart};
use lettre::transport::smtp::PoolConfig;


pub fn send_email(email_addresses: Vec<String>, file_path: &String) {
    let guess = mime_guess::from_path(&file_path).first().unwrap().to_string();

    let body = fs::read(file_path).unwrap();
    let mut mailboxes = Mailboxes::new();
    for address in email_addresses {
        mailboxes.push(address.parse().unwrap());
    }
    let to_header: header::To = mailboxes.into();
    let content_type = ContentType::parse(&guess).unwrap();
    let mut file_ending:Vec<&str>=file_path.split(".").collect();
    let ending = file_ending.pop().unwrap();
    let attachment = Attachment::new(format!("data.{}", &ending)).body(body, content_type);

    let email = Message::builder()
        .from("MDclone <nobody@domain.tld>".parse().unwrap())
        .mailbox(to_header)
        .subject("Happy new year")
        .multipart(
            MultiPart::mixed()
                .singlepart(attachment)
        )
        .unwrap();


    let creds = Credentials::new(env::var("SMTP_USERNAME").unwrap(), env::var("SMTP_PASSWORD").unwrap());

    let pool_config = PoolConfig::new()
        .min_idle(1)
        .max_size(3)
        .idle_timeout(std::time::Duration::from_secs(600));

// Open a remote connection to gmail
    let mailer = SmtpTransport::relay("smtp.gmail.com")
        .unwrap()
        .pool_config(pool_config)
        .credentials(creds)
        .port(env::var("SMTP_PORT").unwrap().parse::<u16>().unwrap())
        .build();


// Send the email
    match mailer.send(&email) {
        Ok(_) => println!("Email sent successfully!"),
        Err(_e) => panic!("Could not send email: {_e:?}"),
    }
}