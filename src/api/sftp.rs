use std::io::Write;

use std::net::TcpStream;
use ssh2::Session;
use std::path::Path;

pub fn ssh_connect() {
    let tcp = TcpStream::connect("127.0.0.1:22").unwrap();
    let mut sess = Session::new().unwrap();
    sess.set_tcp_stream(tcp);
    sess.handshake().unwrap();

    sess.userauth_password("username", "password").unwrap();

    let sftp = sess.sftp().unwrap();

    sftp.mkdir(Path::new("path/to/sftp/dir"), 0o777).ok();
    sftp.create(&Path::new("path/to/file/in/sftp/dir/file.json"))
        .unwrap()
        .write_all("text to be written to file".as_ref())
        .unwrap();
}