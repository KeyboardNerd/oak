//
// Copyright 2024 The Project Oak Authors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
//

use anyhow::Context;
use oak_attestation_verification_types::policy::Policy;
use oak_proto_rust::oak::attestation::v1::{
    ContainerLayerData, ContainerLayerEndorsements, ContainerLayerReferenceValues,
    EventAttestationResults,
};
use prost_types::Any;

use crate::{
    compare::compare_container_layer_measurement_digests,
    expect::get_container_layer_expected_values,
    util::{decode_endorsement_proto, decode_event_proto},
};

pub struct ContainerPolicy {
    reference_values: ContainerLayerReferenceValues,
}

impl ContainerPolicy {
    pub fn new(reference_values: &ContainerLayerReferenceValues) -> Self {
        Self { reference_values: reference_values.clone() }
    }
}

// We have to use [`Policy<[u8], Any>`] instead of [`EventPolicy`], because
// Rust doesn't yet support implementing trait aliases.
// <https://github.com/rust-lang/rfcs/blob/master/text/1733-trait-alias.md>
impl Policy<[u8], Any> for ContainerPolicy {
    fn verify(
        &self,
        encoded_event: &[u8],
        encoded_event_endorsement: &Any,
        milliseconds_since_epoch: i64,
    ) -> anyhow::Result<EventAttestationResults> {
        let event = decode_event_proto::<ContainerLayerData>(
            "type.googleapis.com/oak.attestation.v1.ContainerLayerData",
            encoded_event,
        )?;
        let event_endorsement = decode_endorsement_proto::<ContainerLayerEndorsements>(
            "type.googleapis.com/oak.attestation.v1.ContainerLayerEndorsements",
            encoded_event_endorsement,
        )?;

        let expected_values = get_container_layer_expected_values(
            milliseconds_since_epoch,
            Some(&event_endorsement),
            &self.reference_values,
        )
        .context("couldn't verify container endosements")?;
        compare_container_layer_measurement_digests(&event, &expected_values)
            .context("couldn't verify container event")?;

        // TODO: b/356631062 - Return detailed attestation results.
        Ok(EventAttestationResults {})
    }
}