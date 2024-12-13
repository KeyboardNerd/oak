/*
 * Copyright 2024 The Project Oak Authors
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */
#include "cc/containers/hello_world_enclave_app/app_service.h"

#include <string>

#include "absl/log/log.h"
#include "absl/strings/str_cat.h"
#include "absl/strings/string_view.h"
#include "cc/containers/sdk/oak_session.h"
#include "cc/crypto/common.h"
#include "cc/crypto/server_encryptor.h"
#include "cc/transport/grpc_sync_session_server_transport.h"
#include "grpcpp/server_context.h"
#include "grpcpp/support/status.h"
#include "oak_containers/examples/hello_world/proto/hello_world.grpc.pb.h"
#include "oak_containers/examples/hello_world/proto/hello_world.pb.h"
#include "proto/crypto/crypto.pb.h"
#include "proto/session/service_streaming.pb.h"

namespace oak::containers::hello_world_enclave_app {

using ::oak::session::v1::RequestWrapper;
using ::oak::session::v1::ResponseWrapper;
using ::oak::session::v1::SessionRequest;
using ::oak::session::v1::SessionResponse;

grpc::Status FromAbsl(const absl::Status& status) {
  return grpc::Status(static_cast<grpc::StatusCode>(status.code()),
                      std::string(status.message()));
}

std::string EnclaveApplicationImpl::HandleRequest(absl::string_view request) {
  return absl::StrCat("Hello from the enclave, ", request,
                      "! Btw, the app has a config with a length of ",
                      application_config_.size(), " bytes.");
}

grpc::Status EnclaveApplicationImpl::LegacySession(
    grpc::ServerContext* context,
    grpc::ServerReaderWriter<ResponseWrapper, RequestWrapper>* stream) {
  absl::Status status = oak_session_context_.OakSession(
      stream, [this](absl::string_view request) -> absl::StatusOr<std::string> {
        return HandleRequest(request);
      });

  return FromAbsl(status);
}

grpc::Status EnclaveApplicationImpl::OakSession(
    grpc::ServerContext* context,
    grpc::ServerReaderWriter<SessionResponse, SessionRequest>* stream) {
  auto channel = session_server_.NewChannel(
      std::make_unique<transport::GrpcSyncSessionServerTransport>(stream));
  while (true) {
    absl::StatusOr<std::string> request = (*channel)->Receive();
    if (!request.ok()) {
      return FromAbsl(request.status());
    }
    std::string response = HandleRequest(*request);
    absl::Status send_result = (*channel)->Send(response);
    if (!send_result.ok()) {
      return FromAbsl(send_result);
    }
  }
  return grpc::Status();
}

}  // namespace oak::containers::hello_world_enclave_app