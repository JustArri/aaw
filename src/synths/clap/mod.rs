use crate::APP_VERSION;
use clack_extensions::params::{
    HostParams, HostParamsImplMainThread, HostParamsImplShared, ParamClearFlags, ParamRescanFlags,
};
use clack_host::host::{HostExtensions, HostHandlers, HostInfo, MainThreadHandler, SharedHandler};
use serde::{Deserialize, Serialize};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use utoipa::ToSchema;

pub mod control;
pub mod synth;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ParameterInfo {
    pub id: u32,
    pub name: String,
    pub module: String,
    pub min_value: f64,
    pub max_value: f64,
    pub default_value: f64,

    #[schema(required)]
    pub value: Option<f64>,

    #[schema(required)]
    pub value_text: Option<String>,

    pub flags: u32,
}

impl ParameterInfo {
    const READ_ONLY_FLAG: u32 = 1 << 3;

    pub fn set_value(&mut self, value: f64) -> crate::synths::Result<()> {
        if self.flags & Self::READ_ONLY_FLAG != 0 {
            return Err(crate::synths::Error::InvalidClapParameter {
                parameter_id: self.id,
                reason: "parameter is read-only",
            });
        }
        if !value.is_finite() {
            return Err(crate::synths::Error::InvalidClapParameter {
                parameter_id: self.id,
                reason: "value is not finite",
            });
        }
        if !(self.min_value..=self.max_value).contains(&value) {
            return Err(crate::synths::Error::InvalidClapParameter {
                parameter_id: self.id,
                reason: "value is outside the parameter range",
            });
        }

        self.value = Some(value);
        self.value_text = None;
        Ok(())
    }
}

#[derive(Default)]
pub(super) struct HostRequests {
    restart: AtomicBool,
    process: AtomicBool,
    callback: AtomicBool,
    parameter_flush: AtomicBool,
}

pub(super) struct AawHostShared {
    requests: Arc<HostRequests>,
}

impl<'a> SharedHandler<'a> for AawHostShared {
    fn request_restart(&self) {
        self.requests.restart.store(true, Ordering::Release);
    }
    fn request_process(&self) {
        self.requests.process.store(true, Ordering::Release);
    }
    fn request_callback(&self) {
        self.requests.callback.store(true, Ordering::Release);
    }
}

impl HostParamsImplShared for AawHostShared {
    fn request_flush(&self) {
        self.requests.parameter_flush.store(true, Ordering::Release);
        self.requests.process.store(true, Ordering::Release);
    }
}

pub(super) struct AawHostMainThread {
    requests: Arc<HostRequests>,
}

impl MainThreadHandler<'_> for AawHostMainThread {}

impl HostParamsImplMainThread for AawHostMainThread {
    fn rescan(&self, _flags: ParamRescanFlags) {
        self.requests.restart.store(true, Ordering::Release);
    }

    fn clear(&self, _parameter_id: clack_host::utils::ClapId, _flags: ParamClearFlags) {}
}

pub(super) struct AawHost;

impl HostHandlers for AawHost {
    type Shared<'a> = AawHostShared;

    type MainThread<'a> = AawHostMainThread;
    type AudioProcessor<'a> = ();

    fn declare_extensions(builder: &mut HostExtensions<Self>, _shared: &Self::Shared<'_>) {
        builder.register::<HostParams>();
    }
}

fn host_info() -> HostInfo {
    HostInfo::new("Arri's Audio Workstation", "Arri", "", APP_VERSION)
        .expect("HostInfo expected not to contain any NULL characters. This is a bug.")
}
