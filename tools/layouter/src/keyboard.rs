use hidapi::{HidApi, HidDevice};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Layer {
    Base,
    BaseShift,
    Lower,
    LowerShift,
    Raise,
    RaiseShift,
}

const VENDOR_ID: u16 = 0x4653;
const PRODUCT_ID: u16 = 0x1;
const USAGE_PAGE: u16 = 0xff60;

pub struct Keyboard {
    pub current_layer: Layer,
    device: HidDevice,
}

impl Keyboard {
    pub fn new() -> Result<Self, hidapi::HidError> {
        let api = HidApi::new()?;

        let device = api
            .device_list()
            .find(|device| {
                device.vendor_id() == VENDOR_ID
                    && device.product_id() == PRODUCT_ID
                    && device.usage_page() == USAGE_PAGE
            })
            .ok_or(hidapi::HidError::IoError {
                error: std::io::Error::new(std::io::ErrorKind::NotFound, "Could not find keyboard"),
            })?
            .open_device(&api)?;

        device.set_blocking_mode(false)?;
        Ok(Self {
            current_layer: Layer::Base,
            device,
        })
    }

    pub fn get_current_layer(&mut self) -> Result<Layer, hidapi::HidError> {
        let mut buf = [0u8; 4];
        self.device.read(&mut buf)?;

        if buf[0] != 0x80 {
            // is event type for layer change
            return Ok(self.current_layer);
        }

        let layer_id = buf[1];
        let is_shift_pressed = buf[2];

        let layer = match (layer_id, is_shift_pressed) {
            (0, 1) => Layer::BaseShift,
            (0, _) => Layer::Base,
            (1, 1) => Layer::LowerShift,
            (1, _) => Layer::Lower,
            (2, 1) => Layer::RaiseShift,
            (2, _) => Layer::Raise,
            _ => {
                return Err(hidapi::HidError::IoError {
                    error: std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid layer"),
                })
            }
        };

        self.current_layer = layer;

        Ok(layer)
    }
}
