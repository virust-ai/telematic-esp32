//! Responses for General Commands
use atat::atat_derive::AtatResp;
use atat::heapless::String;

/// Common response
/// Text string that just return "OK".
#[allow(dead_code)]
#[derive(Clone, Debug, AtatResp)]
pub struct CommonResponse {
    pub res: String<64>,
}

/// 4.1 Manufacturer identification
/// Text string identifying the manufacturer.
#[allow(dead_code)]
#[derive(Clone, Debug, AtatResp)]
pub struct ManufacturerId {
    pub id: String<64>,
}

/// Model identification
/// Text string identifying the manufacturer.
#[allow(dead_code)]
#[derive(Clone, Debug, AtatResp)]
pub struct ModelId {
    pub id: String<64>,
}

/// Software version identification
/// Read a text string that identifies the software version of the module.
#[allow(dead_code)]
#[derive(Clone, Debug, AtatResp)]
pub struct SoftwareVersion {
    pub id: String<64>,
}

/// Get SIM card status
#[allow(dead_code)]
#[derive(Clone, Debug, AtatResp)]
pub struct SimCardStatus {
    pub status: String<64>,
}

/// Network Registration Status
#[allow(dead_code)]
#[derive(Clone, Debug, AtatResp)]
pub struct NetworkRegisStatus {
    pub status: String<64>,
}

/// Network Signal Quality
#[allow(dead_code)]
#[derive(Clone, Debug, AtatResp)]
pub struct NetworkSignalQuality {
    #[at_arg(position = 0)]
    pub rssi: u8,
    #[at_arg(position = 1)]
    pub ber: u8,
}

/// Network Operator Name
#[allow(dead_code)]
#[derive(Clone, Debug, AtatResp)]
pub struct NetworkOperatorName {
    #[at_arg(position = 0)]
    pub mode: u8,
    #[at_arg(position = 1)]
    pub format: u8,
    #[at_arg(position = 2)]
    pub oper: String<64>,
    #[at_arg(position = 3)]
    pub act: u8,
}

/// GPS data
#[allow(dead_code)]
#[derive(Default, Clone, Debug, AtatResp)]
pub struct GpsData {
    #[at_arg(position = 0)]
    pub rmc: atat::heapless_bytes::Bytes<6>,
    #[at_arg(position = 1)]
    pub utc: Option<f64>,
    #[at_arg(position = 2)]
    pub status: Option<char>,
    #[at_arg(position = 3)]
    pub latitude: Option<f64>,
    #[at_arg(position = 4)]
    pub latitude_direction: Option<char>,
    #[at_arg(position = 5)]
    pub longtitude: Option<f64>,
    #[at_arg(position = 6)]
    pub longtitude_direction: Option<char>,
    #[at_arg(position = 7)]
    pub spkm: Option<f64>,
    #[at_arg(position = 8)]
    pub heading: Option<f64>,
    #[at_arg(position = 9)]
    pub date: Option<u64>,
    #[at_arg(position = 10)]
    pub magnetic: Option<f64>,
    #[at_arg(position = 11)]
    pub magnetic_direction: Option<char>,
    #[at_arg(position = 12)]
    pub checksum: Option<u32>,
}
