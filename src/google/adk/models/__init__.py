# Copyright 2025 Google LLC
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

"""Defines the interface to support a model."""

# Core model implementations have been migrated to Rust
# See: src/models/ for Rust implementations
# Remaining Python implementations for advanced features:

from .anthropic_llm import AnthropicLlm
from .lite_llm import LiteLlm
from .base_llm_connection import BaseLlmConnection
from .gemini_llm_connection import GeminiLlmConnection

__all__ = [
    'AnthropicLlm',
    'LiteLlm',
    'BaseLlmConnection',
    'GeminiLlmConnection',
]

# Migration Note:
# Basic model types (BaseLlm, GoogleLlm, LlmRequest, LlmResponse, Registry)
# have been migrated to Rust for better performance and type safety.
# Use the Rust implementation: `use google_adk::models::*;`
