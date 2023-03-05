pub struct Transport {
    pub id: String,
    pub family: String,
    pub port: u16,
    pub updown: Option<String>,
    pub address: Option<String>,
    pub fwmark: Option<String>,
}

pub struct Endpoint {
    pub id: String,
    pub family: String,
    pub port: u16,
    pub address: Option<String>,
}
