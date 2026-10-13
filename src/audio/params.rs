#[derive(Debug, Clone, Copy)]
pub enum HostSelection<'a> {
    Default,
    Id(&'a str),
}

#[derive(Debug, Clone, Copy)]
pub enum DeviceSelection<'a> {
    Default,
    Id(&'a str),
}

#[derive(Debug, Clone, Copy)]
pub enum LatencySelection {
    Automatic,
    Low,
    Stable,
    CustomBufferSize(u32),
}

#[derive(Debug, Clone, Copy)]
pub enum SampleRateSelection {
    Automatic,
    Fixed(u32),
}

#[derive(Debug, Clone, Copy)]
pub struct DeviceCfgSelection<'a> {
    pub host: HostSelection<'a>,
    pub device: DeviceSelection<'a>,
    pub latency: LatencySelection,
    pub sample_rate: SampleRateSelection,
}
