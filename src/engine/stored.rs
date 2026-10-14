use crate::engine::{ctr, dummy_graph, fluid_graph, sfizz_graph, synth_graph};
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub synth_graph: synth_graph::stored::Graph,
    pub dummy_graph: dummy_graph::stored::Graph,
    pub fluid_graph: fluid_graph::stored::Graph,
    pub sfizz_graph: sfizz_graph::stored::Graph,
}

impl From<&ctr::Project> for Project {
    fn from(value: &ctr::Project) -> Self {
        Self {
            synth_graph: (&value.synth_graph).into(),
            dummy_graph: (&value.dummy_graph).into(),
            fluid_graph: (&value.fluid_graph).into(),
            sfizz_graph: (&value.sfizz_graph).into(),
        }
    }
}

impl From<ctr::Project> for Project {
    fn from(value: ctr::Project) -> Self {
        Self {
            synth_graph: value.synth_graph.into(),
            dummy_graph: value.dummy_graph.into(),
            fluid_graph: value.fluid_graph.into(),
            sfizz_graph: value.sfizz_graph.into(),
        }
    }
}
