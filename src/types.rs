/// SEN5x sensor data.
pub struct Sen5xData {
    /// Mass Concentration PM1.0 [μg/m³]
    pub pm1_0: f32,
    /// Mass Concentration PM2.5 [μg/m³]
    pub pm2_5: f32,
    /// Mass Concentration PM4.0 [μg/m³]
    pub pm4_0: f32,
    /// Mass Concentration PM10 [μg/m³]
    pub pm10_0: f32,
    /// Compensated Ambient Humidity [%RH]
    pub humidity: f32,
    /// Compensated Ambient Temperature [°C]
    pub temperature: f32,
    /// VOC Index
    pub voc_index: f32,
    /// NOx Index
    pub nox_index: f32,
}

/// SEN5x sensor raw data.
pub struct Sen5xDataRaw {
    /// Mass Concentration PM1.0 [μg/m³] [×10]
    pub pm1_0: u16,
    /// Mass Concentration PM2.5 [μg/m³] [×10]
    pub pm2_5: u16,
    /// Mass Concentration PM4.0 [μg/m³] [×10]
    pub pm4_0: u16,
    /// Mass Concentration PM10.0 [μg/m³] [×10]
    pub pm10_0: u16,
    /// Compensated Ambient Temperature [°C] [×200]
    pub temperature: u16,
    /// Compensated Ambient Humidity [%RH] [×100]
    pub humidity: u16,
    /// VOC Index [×10]
    pub voc_index: u16,
    /// NOx Index [×10]
    pub nox_index: u16,
}
