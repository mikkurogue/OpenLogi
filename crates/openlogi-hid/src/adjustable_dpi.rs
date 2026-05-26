//! Bare-minimum HID++ `AdjustableDpi` (feature `0x2201`) wrapper.
//!
//! `hidpp 0.2` ships an entry in the feature registry but no typed
//! implementation. We re-implement the few functions OpenLogi needs:
//! `getSensorCount` (probe), `getSensorDpi` (read current), and
//! `setSensorDpi` (write a new value).
//!
//! Follows the same shape as the typed wrappers `hidpp` ships
//! (`DeviceInformationFeatureV0`, `UnifiedBatteryFeatureV0`, …).

use std::sync::Arc;

use hidpp::{
    channel::HidppChannel,
    feature::{CreatableFeature, Feature},
    nibble::U4,
    protocol::v20::{self, Hidpp20Error},
};

/// `AdjustableDpi` / `0x2201` feature, version 0+.
#[derive(Clone)]
pub struct AdjustableDpiFeatureV0 {
    chan: Arc<HidppChannel>,
    device_index: u8,
    feature_index: u8,
}

impl CreatableFeature for AdjustableDpiFeatureV0 {
    const ID: u16 = 0x2201;
    const STARTING_VERSION: u8 = 0;

    fn new(chan: Arc<HidppChannel>, device_index: u8, feature_index: u8) -> Self {
        Self {
            chan,
            device_index,
            feature_index,
        }
    }
}

impl Feature for AdjustableDpiFeatureV0 {}

impl AdjustableDpiFeatureV0 {
    /// Number of sensors the device exposes. Mice almost always report
    /// `1`; the API still indexes the rest of the functions by sensor.
    pub async fn get_sensor_count(&self) -> Result<u8, Hidpp20Error> {
        let response = self
            .chan
            .send_v20(v20::Message::Short(
                v20::MessageHeader {
                    device_index: self.device_index,
                    feature_index: self.feature_index,
                    function_id: U4::from_lo(0),
                    software_id: self.chan.get_sw_id(),
                },
                [0x00, 0x00, 0x00],
            ))
            .await?;
        Ok(response.extend_payload()[0])
    }

    /// Currently-configured DPI for `sensor_index`. Returned tuple is
    /// `(current, default)` per the HID++ spec; we return only the
    /// current value since `default` isn't useful for the GUI today.
    pub async fn get_sensor_dpi(&self, sensor_index: u8) -> Result<u16, Hidpp20Error> {
        let response = self
            .chan
            .send_v20(v20::Message::Short(
                v20::MessageHeader {
                    device_index: self.device_index,
                    feature_index: self.feature_index,
                    function_id: U4::from_lo(2),
                    software_id: self.chan.get_sw_id(),
                },
                [sensor_index, 0x00, 0x00],
            ))
            .await?;
        let payload = response.extend_payload();
        Ok(u16::from_be_bytes([payload[1], payload[2]]))
    }

    /// Write a new DPI value. The device echoes the request back as
    /// the response; we discard the echo and report success.
    pub async fn set_sensor_dpi(&self, sensor_index: u8, dpi: u16) -> Result<(), Hidpp20Error> {
        let [dpi_hi, dpi_lo] = dpi.to_be_bytes();
        let _ = self
            .chan
            .send_v20(v20::Message::Short(
                v20::MessageHeader {
                    device_index: self.device_index,
                    feature_index: self.feature_index,
                    function_id: U4::from_lo(3),
                    software_id: self.chan.get_sw_id(),
                },
                [sensor_index, dpi_hi, dpi_lo],
            ))
            .await?;
        Ok(())
    }
}
