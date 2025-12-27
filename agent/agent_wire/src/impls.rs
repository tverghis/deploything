use bollard::secret::{ContainerSummary, ContainerSummaryStateEnum};

use crate::deploything::v1::{ContainerState, ContainerStatus};

impl From<&ContainerSummary> for ContainerStatus {
    fn from(summary: &ContainerSummary) -> Self {
        let summary = summary.clone();
        let name = summary.names.map(|names| names.first().unwrap().clone());
        let state = summary
            .state
            .map(|state| ContainerState::from(state) as i32);

        ContainerStatus {
            id: summary.id,
            name: name,
            image_id: summary.image_id,
            container_state: state,
        }
    }
}

impl From<ContainerSummaryStateEnum> for ContainerState {
    fn from(state: ContainerSummaryStateEnum) -> Self {
        match state {
            ContainerSummaryStateEnum::RUNNING => ContainerState::Running,
            ContainerSummaryStateEnum::EXITED => ContainerState::Exited,
            _ => ContainerState::Unspecified,
        }
    }
}
