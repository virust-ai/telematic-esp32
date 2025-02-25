use embedded_storage::{ReadStorage, Storage};
use esp_storage::FlashStorage;

const NVS_ADDR: u32 = 0x9000;
const CRT_PEM_ADDR: u32 = 0x9000;
const CRT_PEM_SIZE: usize = 2574;
const DVT_CRT_ADDR: u32 = 0xA000;
const DVT_CRT_SIZE: usize = 1268;
const DVT_KEY_ADDR: u32 = 0xB000;
const DVT_KEY_SIZE: usize = 1678;

pub enum NvsError {
    LenErr,
    Other
}

#[allow(dead_code)]
pub struct EspNvs {
    addr: u32,
    storage: FlashStorage,
}

#[allow(dead_code)]
impl EspNvs {
    pub fn init() -> Self {
        Self {
            addr: NVS_ADDR,
            storage: FlashStorage::new(),
        }
    }

    pub fn nvs_write_crt_pem(&mut self, buf: &[u8]) -> Result<(), NvsError> {
        if buf.len() != CRT_PEM_SIZE {
            return Err(NvsError::LenErr);
        } else {
            self.storage.write(CRT_PEM_ADDR, buf);
            Ok(())
        }
    }
    pub fn nvs_write_dvt_crt(&mut self, buf: &[u8]) -> Result<(), NvsError> {
        if buf.len() != DVT_CRT_SIZE {
            return Err(NvsError::LenErr);
        } else {
            self.storage.write(DVT_CRT_ADDR, buf);
            Ok(())
        }
    }
    pub fn nvs_write_dvt_key(&mut self, buf: &[u8]) -> Result<(), NvsError> {
        if buf.len() != DVT_KEY_SIZE {
            return Err(NvsError::LenErr);
        } else {
            self.storage.write(DVT_KEY_ADDR, buf);
            Ok(())
        }
    }
    pub fn nvs_read_crt_pem(&mut self, buf: &mut [u8]) -> Result<(), NvsError> {
        if buf.len() != CRT_PEM_SIZE {
            return Err(NvsError::LenErr);
        } else {
            self.storage.read(CRT_PEM_ADDR, buf);
            Ok(())
        }
    }
    pub fn nvs_read_dvt_crt(&mut self, buf: &mut [u8]) -> Result<(), NvsError> {
        if buf.len() != DVT_CRT_SIZE {
            return Err(NvsError::LenErr);
        } else {
            self.storage.read(DVT_CRT_ADDR, buf);
            Ok(())
        }
    }
    pub fn nvs_read_dvt_key(&mut self, buf: &mut [u8]) -> Result<(), NvsError> {
        if buf.len() != DVT_KEY_SIZE {
            return Err(NvsError::LenErr);
        } else {
            self.storage.read(DVT_KEY_ADDR, buf);
            Ok(())
        }
    }
}
