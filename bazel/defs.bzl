#
# Copyright 2024 The Project Oak Authors
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#     http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.
#
"""Forwarder for bazel rules and macros."""

load("//bazel/private:oci_runtime_bundle.bzl", _oci_runtime_bundle = "oci_runtime_bundle")

oci_runtime_bundle = _oci_runtime_bundle

RUST_NIGHTLY_VERSION = "nightly/2024-02-01"

RUST_VERSIONS = [
    "1.76.0",
    RUST_NIGHTLY_VERSION,
]
