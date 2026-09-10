# CLI starter notice

SPDX-License-Identifier: MIT

Copyright (c) 2026 tauri-remote-app-builder contributors.

Keep this notice and `LICENSE` with copies scaffolded from this directory.

This code is a typed command-line adapter template. It does not implement a
second authority, store mutable product state, authenticate principals, or
provide a generic Tauri-command forwarding channel. Wire its `AdminAdapter`
methods to the same running Rust service that owns the desktop application's
state, authorization, generation, and revision counters.

The template is not a security certification. Review the concrete transport,
authentication, authorization, audit, packaging, and update paths in every
application that incorporates it.

