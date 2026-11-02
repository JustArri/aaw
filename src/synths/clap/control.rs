use super::{AawHost, ParameterInfo};
use crate::synths::{
    Error,
    clap::{AawHostShared, host_info},
};
use anyhow::{Context, Result, anyhow};
use clack_extensions::params::{ParamInfoBuffer, PluginParams};
use clack_host::{
    plugin::PluginInstance,
    plugin::features,
    process::{PluginAudioConfiguration, StoppedPluginAudioProcessor},
};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::{Arc, LazyLock, atomic::Ordering, mpsc},
    thread,
    time::Duration,
};
use tokio::sync::oneshot;

pub type Processor = StoppedPluginAudioProcessor<AawHost>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct InstanceId(u64);

pub struct LoadedPlugin {
    pub id: InstanceId,
    pub processor: Processor,
    pub parameters: Vec<ParameterInfo>,
    pub name: Option<String>,
    pub version: Option<String>,
    pub requests: Arc<super::HostRequests>,
}

// Reconfiguration can fail before or after the old
// processor has been consumed.
pub enum ReconfigureResult {
    Ready(Processor),
    Rejected { error: anyhow::Error, processor: Processor },
    ActivationFailed(anyhow::Error),
}

enum Command {
    Load {
        path: PathBuf,
        plugin_id: String,
        config: PluginAudioConfiguration,
        reply: oneshot::Sender<Result<LoadedPlugin>>,
    },
    Reconfigure {
        id: InstanceId,
        processor: Processor,
        config: PluginAudioConfiguration,
        reply: oneshot::Sender<ReconfigureResult>,
    },
    Destroy {
        id: InstanceId,
        processor: Option<Processor>,
    },
}

#[derive(Clone)]
pub struct ClapControlThread {
    sender: mpsc::Sender<Command>,
}

impl ClapControlThread {
    pub fn shared() -> Result<Self> {
        static CONTROL: LazyLock<std::result::Result<ClapControlThread, String>> =
            LazyLock::new(|| ClapControlThread::spawn().map_err(|error| error.to_string()));

        CONTROL.as_ref().cloned().map_err(|error| anyhow!(error.clone()))
    }

    pub fn spawn() -> Result<Self> {
        let (sender, receiver) = mpsc::channel();

        thread::Builder::new()
            .name("clap-control".into())
            .spawn(move || {
                ControlState::new().run(receiver);
            })
            .context("Failed to spawn CLAP control thread")?;

        Ok(Self { sender })
    }

    pub async fn load(
        &self,
        path: PathBuf,
        plugin_id: String,
        config: PluginAudioConfiguration,
    ) -> Result<LoadedPlugin> {
        let (reply, receiver) = oneshot::channel();

        self.sender
            .send(Command::Load { path, plugin_id, config, reply })
            .map_err(|_| anyhow!("CLAP control thread unavailable"))?;

        receiver.await.context("CLAP control thread stopped during load")?
    }

    pub async fn reconfigure(
        &self,
        id: InstanceId,
        processor: Processor,
        config: PluginAudioConfiguration,
    ) -> ReconfigureResult {
        let (reply, receiver) = oneshot::channel();
        let command = Command::Reconfigure { id, processor, config, reply };

        if let Err(error) = self.sender.send(command) {
            let Command::Reconfigure { processor, .. } = error.0 else { unreachable!() };
            return ReconfigureResult::Rejected {
                error: anyhow::anyhow!("CLAP control thread unavailable"),
                processor,
            };
        }

        match receiver.await {
            Ok(result) => result,
            Err(_) => ReconfigureResult::ActivationFailed(anyhow!(
                "CLAP control thread terminated during reconfiguration"
            )),
        }
    }

    pub fn destroy(&self, id: InstanceId, processor: Option<Processor>) {
        let _ = self.sender.send(Command::Destroy { id, processor });
    }
}

// Never moved outside the control OS thread.
struct ControlState {
    next_id: u64,
    instances: HashMap<InstanceId, HostedInstance>,
}

struct HostedInstance {
    instance: PluginInstance<AawHost>,
    requests: Arc<super::HostRequests>,
}

impl ControlState {
    fn new() -> Self {
        Self { next_id: 1, instances: HashMap::new() }
    }

    fn run(&mut self, receiver: mpsc::Receiver<Command>) {
        loop {
            let command = match receiver.recv_timeout(Duration::from_millis(10)) {
                Ok(command) => command,
                Err(mpsc::RecvTimeoutError::Timeout) => {
                    self.service_callbacks();
                    continue;
                }
                Err(mpsc::RecvTimeoutError::Disconnected) => break,
            };
            match command {
                Command::Load { path, plugin_id, config, reply } => {
                    let result = self.load(path, plugin_id, config);

                    // If the HTTP request was cancelled after
                    // loading, deactivate the unused processor.
                    if let Err(result) = reply.send(result)
                        && let Ok(loaded) = result
                    {
                        self.cleanup_loaded(loaded);
                    }
                }

                Command::Reconfigure { id, processor, config, reply } => {
                    let result = self.reconfigure(id, processor, config);

                    if let Err(result) = reply.send(result) {
                        match result {
                            ReconfigureResult::Ready(processor) => {
                                self.cleanup_processor(id, processor);
                            }
                            ReconfigureResult::Rejected { processor, .. } => {
                                self.cleanup_processor(id, processor);
                            }
                            ReconfigureResult::ActivationFailed(_) => {
                                self.instances.remove(&id);
                            }
                        }
                    }
                }

                Command::Destroy { id, processor } => {
                    self.destroy(id, processor);
                }
            }

            self.service_callbacks();
        }
    }

    fn service_callbacks(&mut self) {
        for hosted in self.instances.values_mut() {
            if hosted.requests.callback.swap(false, Ordering::AcqRel) {
                // SAFETY: this is the persistent main/control thread for this instance.
                hosted.instance.call_on_main_thread_callback();
            }

            if hosted.requests.restart.swap(false, Ordering::AcqRel) {
                tracing::warn!("CLAP plugin requested a restart; waiting for host reconfiguration");
            }
        }
    }

    fn load(
        &mut self,
        path: PathBuf,
        plugin_id: String,
        config: PluginAudioConfiguration,
    ) -> Result<LoadedPlugin> {
        let requests = Arc::new(super::HostRequests::default());
        let loaded_instance = load_plugin_sync(&path, &plugin_id, Arc::clone(&requests))
            .context("Failed to create CLAP instance")?;
        let mut instance = loaded_instance.instance;

        let processor =
            instance.activate(|_, _| (), config).context("Failed to activate CLAP plugin")?;
        let parameters = read_parameters(&mut instance);

        let id = InstanceId(self.next_id);
        self.next_id = self.next_id.checked_add(1).expect("CLAP instance ID exhausted");

        self.instances.insert(id, HostedInstance { instance, requests: Arc::clone(&requests) });

        Ok(LoadedPlugin {
            id,
            processor,
            parameters,
            name: loaded_instance.name,
            version: loaded_instance.version,
            requests,
        })
    }

    fn reconfigure(
        &mut self,
        id: InstanceId,
        processor: Processor,
        config: PluginAudioConfiguration,
    ) -> ReconfigureResult {
        let Some(hosted) = self.instances.get_mut(&id) else {
            return ReconfigureResult::Rejected {
                error: anyhow!("Unknown CLAP instance"),
                processor,
            };
        };

        if !processor.matches(&hosted.instance) {
            return ReconfigureResult::Rejected {
                error: anyhow!("Wrong CLAP processor"),
                processor,
            };
        }

        hosted.instance.deactivate(processor);

        match hosted.instance.activate(|_, _| (), config) {
            Ok(processor) => ReconfigureResult::Ready(processor),
            Err(error) => ReconfigureResult::ActivationFailed(error.into()),
        }
    }

    fn destroy(&mut self, id: InstanceId, processor: Option<Processor>) {
        let Some(hosted) = self.instances.get_mut(&id) else {
            return;
        };

        match processor {
            Some(processor) if processor.matches(&hosted.instance) => {
                hosted.instance.deactivate(processor);
                self.instances.remove(&id);
            }
            Some(processor) => {
                tracing::error!("refusing to destroy CLAP instance with a mismatched processor");
                drop(processor);
            }
            None if !hosted.instance.is_active() => {
                self.instances.remove(&id);
            }
            None => match hosted.instance.try_deactivate() {
                Ok(()) => {
                    self.instances.remove(&id);
                }
                Err(error) => {
                    tracing::error!(%error, "could not destroy active CLAP instance");
                }
            },
        }
    }

    fn cleanup_loaded(&mut self, loaded: LoadedPlugin) {
        self.cleanup_processor(loaded.id, loaded.processor);
    }

    fn cleanup_processor(&mut self, id: InstanceId, processor: Processor) {
        if let Some(hosted) = self.instances.get_mut(&id)
            && processor.matches(&hosted.instance)
        {
            hosted.instance.deactivate(processor);
            self.instances.remove(&id);
            return;
        }

        // The processor must not be used after this point.
        // A mismatched/unknown processor indicates a host bug.
        drop(processor);
    }
}

fn read_parameters(instance: &mut PluginInstance<AawHost>) -> Vec<ParameterInfo> {
    let plugin = instance.plugin_handle();
    let Some(parameters) = plugin.get_extension::<PluginParams>() else {
        return Vec::new();
    };
    let mut info_buffer = ParamInfoBuffer::new();
    let mut text_buffer = [0_u8; 256];
    let parameter_count = parameters.count(&plugin);
    let mut result = Vec::with_capacity(parameter_count as usize);

    for index in 0..parameter_count {
        let Some(info) = parameters.get_info(&plugin, index, &mut info_buffer) else {
            continue;
        };
        let value = parameters.get_value(&plugin, info.id);
        let value_text = value.and_then(|value| {
            parameters
                .value_to_text(&plugin, info.id, value, &mut text_buffer)
                .ok()
                .map(|text| String::from_utf8_lossy(text).into_owned())
        });

        result.push(ParameterInfo {
            id: info.id.get(),
            name: String::from_utf8_lossy(info.name).into_owned(),
            module: String::from_utf8_lossy(info.module).into_owned(),
            min_value: info.min_value,
            max_value: info.max_value,
            default_value: info.default_value,
            value,
            value_text,
            flags: info.flags.bits(),
        });
    }

    result
}

struct LoadedInstance {
    instance: PluginInstance<AawHost>,
    name: Option<String>,
    version: Option<String>,
}

fn load_plugin_sync(
    path: impl AsRef<Path>,
    plugin_id: &str,
    requests: Arc<super::HostRequests>,
) -> Result<LoadedInstance> {
    let entry = unsafe {
        clack_host::entry::PluginEntry::load(path.as_ref()).map_err(|e| Error::ClapPluginLoad {
            plugin_path: path.as_ref().to_owned(),
            plugin_id: plugin_id.to_string(),
            source: e,
        })?
    };

    let plugin_factory = entry.get_plugin_factory().ok_or_else(|| Error::InvalidClapPlugin {
        plugin_path: path.as_ref().to_owned(),
        plugin_id: plugin_id.to_string(),
    })?;

    let plugin_descriptor = plugin_factory
        .plugin_descriptors()
        .find(|d| d.id().is_some_and(|id| id.to_bytes() == plugin_id.as_bytes()))
        .ok_or_else(|| Error::MissingClapPlugin {
            plugin_path: path.as_ref().to_owned(),
            plugin_id: plugin_id.to_string(),
        })?;

    if !plugin_descriptor.features().any(|feature| feature == features::INSTRUMENT) {
        return Err(Error::ClapPluginNotInstrument.into());
    }
    let name = plugin_descriptor.name().and_then(|name| name.to_str().ok()).map(str::to_owned);
    let version =
        plugin_descriptor.version().and_then(|version| version.to_str().ok()).map(str::to_owned);

    let main_requests = Arc::clone(&requests);
    let instance = PluginInstance::<AawHost>::new(
        |_| AawHostShared { requests },
        |_| super::AawHostMainThread { requests: main_requests },
        &entry,
        plugin_descriptor.id().unwrap(),
        &host_info(),
    )
    .map_err(|e| Error::ClapPluginInstance {
        plugin_path: path.as_ref().to_owned(),
        plugin_id: plugin_id.to_string(),
        source: e,
    })?;

    Ok(LoadedInstance { instance, name, version })
}
