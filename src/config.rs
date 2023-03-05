pub struct Transport<'a> {
    pub id: &'a str,
    pub family: &'a str,
    pub port: u16,
    pub updown: Option<&'a str>,
    pub address: Option<&'a str>,
    pub fwmark: Option<&'a str>,
}

pub struct Endpoint<'a> {
    pub id: &'a str,
    pub family: &'a str,
    pub port: u16,
    pub address: Option<&'a str>,
}
