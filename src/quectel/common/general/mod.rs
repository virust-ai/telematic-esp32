//! ### 4 - General Commands
pub mod responses;
pub mod urc;

use core::str::FromStr;

use atat::{atat_derive::AtatCmd, AtatCmd};
use heapless::String;
use log::info;
use responses::*;

use super::NoResponse;

/// Disable echo mode E0
///
/// Disable echo mode
#[derive(Clone, AtatCmd)]
#[at_cmd("E0", NoResponse)]
pub struct DisableEchoMode;

/// 4.1 Manufacturer identification +CGMI
///
/// Text string identifying the manufacturer.
#[derive(Clone, AtatCmd)]
#[at_cmd("+CGMI", ManufacturerId)]
pub struct GetManufacturerId;

/// Model identification +CGMM
///
/// Read a text string that identifies the device model.
#[derive(Clone, AtatCmd)]
#[at_cmd("+CGMM", ModelId)]
pub struct GetModelId;

/// Software version identification +CGMR
///
/// Read a text string that identifies the software version of the module
#[derive(Clone, AtatCmd)]
#[at_cmd("+CGMR", SoftwareVersion)]
pub struct GetSoftwareVersion;

/// Full-Functionality Mode +CFUN=1
///
/// Set to Full-Functionality Mode
#[derive(Clone, AtatCmd)]
#[at_cmd("+CFUN=1", CommonResponse)]
pub struct SetFullFuncMode;

/// SIM card status +CPIN?
///
/// Check SIM card status
#[derive(Clone, AtatCmd)]
#[at_cmd("+CPIN?", SimCardStatus)]
pub struct GetSimCardStatus;

/// Network Registration Status +CREG?
///
/// Check Network Registration Status
#[derive(Clone, AtatCmd)]
#[at_cmd("+CREG?", NetworkRegisStatus)]
pub struct GetNetworkRegisStatus;

/// Network Signal Quality +CSQ
///
/// Check Network Signal Quality
#[derive(Clone, AtatCmd)]
#[at_cmd("+CSQ", NetworkSignalQuality)]
pub struct GetNetworkSignalQuality;

/// Network Operator Name +COPS?
///
/// Check Network Operator Name
#[derive(Clone, AtatCmd)]
#[at_cmd("+COPS?", NetworkOperatorName)]
pub struct GetNetworkOperatorName;

/// Packet-Switched Network +CGATT=1
///
/// Attach to the Packet-Switched Network
#[derive(Clone, AtatCmd)]
#[at_cmd("+CGATT=1", NoResponse)]
pub struct AttachPacketSwitch;

/// Enable GPS Functionality +QGPS=1
///
/// Enable GPS Functionality
#[derive(Clone, AtatCmd)]
#[at_cmd("+QGPS=1", NoResponse, timeout_ms = 1000, response_code = false)]
pub struct EnableGpsFunc;

/// Enable Assisted GPS Functionality +QAGPS=1
///
/// Enable Assisted GPS
#[derive(Clone, AtatCmd)]
#[at_cmd("+QAGPS=1", NoResponse, response_code = false)]
pub struct EnableAssistGpsFunc;

/// Enable GNSS Functionality +QGPSCFG="gnssconfig",1
///
/// Enable GNSS Functionality
#[derive(Clone, AtatCmd)]
#[at_cmd("+QGPSCFG=\"gnssconfig\",1", NoResponse)]
pub struct EnableGnssFunc;

/// GPS Position Data +QGPSGNMEA="RMC"
///
/// Retrieve GPS Position Data
#[derive(Clone)]
pub struct RetrieveGpsData;

impl AtatCmd for RetrieveGpsData {
    type Response = GpsData;

    const MAX_LEN: usize = 1000;

    fn write(&self, buf: &mut [u8]) -> usize {
        info!("write AT+QGPSGNMEA=\"RMC\"");
        let cmd = b"AT+QGPSGNMEA=\"RMC\"\r\n";
        let len = cmd.len();
        buf[..len].copy_from_slice(cmd);
        len
    }

    fn parse(
        &self,
        resp: Result<&[u8], atat::InternalError>,
    ) -> Result<Self::Response, atat::Error> {
        if let Ok(gps) = resp {
            let str = core::str::from_utf8(gps).unwrap();
            let a: String<100> = String::from_str(str).unwrap();
            info!("{a}");
        }
        Ok(GpsData::default())
    }
}
