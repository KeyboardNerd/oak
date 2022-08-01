//
// Copyright 2022 The Project Oak Authors
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

use crate::Client;
use bmrng::unbounded::UnboundedRequestSender;
use futures::Future;
use oak_baremetal_communication_channel::schema;
use oak_remote_attestation_sessions::{SessionId, SESSION_ID_LENGTH};
use std::net::SocketAddr;
use tonic::{transport::Server, Request, Response};

pub mod proto {
    tonic::include_proto!("oak.session.unary.v1");
}

use proto::{
    unary_session_server::{UnarySession, UnarySessionServer},
    UnaryRequest, UnaryResponse,
};

fn encode_request(unary_request: UnaryRequest) -> Result<Vec<u8>, oak_idl::Status> {
    let mut session_id: SessionId = [0; SESSION_ID_LENGTH];
    if unary_request.session_id.len() != SESSION_ID_LENGTH {
        return Err(oak_idl::Status::new_with_message(
            oak_idl::StatusCode::Internal,
            format!(
                "session_id must be {} bytes in length, found a length of {} bytes instead",
                SESSION_ID_LENGTH,
                unary_request.session_id.len()
            ),
        ));
    }
    session_id.copy_from_slice(&unary_request.session_id);

    // Create the flatbuffer (containing an implicit lifetime)
    let owned_request_flatbuffer = {
        let mut builder = oak_idl::utils::OwnedFlatbufferBuilder::default();
        let session_id = &schema::SessionId::new(&session_id);
        let body = builder.create_vector::<u8>(&unary_request.body);
        let flatbuffer = schema::UserRequest::create(
            &mut builder,
            &schema::UserRequestArgs {
                session_id: Some(session_id),
                body: Some(body),
            },
        );
        builder.finish(flatbuffer).map_err(|err| {
            oak_idl::Status::new_with_message(oak_idl::StatusCode::Internal, err.to_string())
        })?
    };

    // Return the underlying owned buffer
    Ok(owned_request_flatbuffer.into_vec())
}

fn decode_response(encoded_response: Vec<u8>) -> Result<UnaryResponse, tonic::Status> {
    let response =
        oak_idl::utils::OwnedFlatbuffer::<schema::UserRequestResponse>::from_vec(encoded_response)
            .map_err(|err| tonic::Status::internal(err.to_string()))?;

    let response_body = response
        .get()
        .body()
        .ok_or_else(|| tonic::Status::internal(""))?;

    Ok(UnaryResponse {
        body: response_body.to_vec(),
    })
}

pub struct EchoImpl {
    request_dispatcher: UnboundedRequestSender<oak_idl::Request, Result<Vec<u8>, oak_idl::Status>>,
}

#[tonic::async_trait]
impl UnarySession for EchoImpl {
    async fn message(
        &self,
        request: Request<UnaryRequest>,
    ) -> Result<Response<UnaryResponse>, tonic::Status> {
        let request = request.into_inner();
        let encoded_request = encode_request(request)
            .map_err(|err| tonic::Status::invalid_argument(format!("{:?}", err)))?;

        let mut client = schema::TrustedRuntimeAsyncClient::new(Client {
            request_dispatcher: self.request_dispatcher.clone(),
        });

        let encoded_response = client
            .handle_user_request(encoded_request)
            .await
            .map_err(|err| tonic::Status::internal(format!("{:?}", err)))?;
        let response = decode_response(encoded_response.into_vec())?;

        Ok(Response::new(response))
    }
}

pub fn server(
    addr: SocketAddr,
    request_dispatcher: UnboundedRequestSender<oak_idl::Request, Result<Vec<u8>, oak_idl::Status>>,
) -> impl Future<Output = Result<(), tonic::transport::Error>> {
    let server_impl = EchoImpl { request_dispatcher };
    Server::builder()
        .add_service(UnarySessionServer::new(server_impl))
        .serve(addr)
}