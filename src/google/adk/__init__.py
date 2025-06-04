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

from . import version

# MIGRATION NOTICE:
# The Google ADK is transitioning from Python to Rust for better performance,
# type safety, and modern web server capabilities.
#
# Core functionality (agents, models, tools, CLI, web server) has been migrated to Rust.
# Advanced features (evaluation, code execution, memory services) remain in Python.
#
# For new projects, use the Rust implementation:
#   cargo run -- --help
#
# For existing projects using advanced features, continue using Python modules:
#   - Code execution: google.adk.code_executors
#   - Evaluation: google.adk.evaluation
#   - Memory services: google.adk.memory
#   - Advanced tools: google.adk.tools.*
#   - Authentication: google.adk.auth
#
# See MIGRATION_STATUS.md for detailed migration status.

from .runners import Runner

__version__ = version.__version__
__all__ = ["Runner"]

# Legacy imports for backward compatibility (deprecated)
# Use Rust implementation for new development
