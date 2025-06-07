use smbclient::SmbClient;

fn main() {
    let mut client = SmbClient::new("SERVER_IP", "username", "NTLM_HASH").unwrap();
    client.connect().unwrap();
    let shares = client.list_shares().unwrap();
    for s in shares { println!("{}", s); }
}
