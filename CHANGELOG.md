# Changelog

<!-- markdownlint-disable-file MD024 -->

This changelog is manually updated. While an effort will be made to keep the [Unreleased](#unreleased-yyyy-mm-dd) changes up to date, it may not be fully representative of the current state of the project.

<!-- If you are updating this file, make sure you copy the unreleased section and change the version and date. Do not remove it. -->

## Unreleased (YYYY-MM-DD)

### Added

#### Headline features

- **API**: Guest code can now **create and interact with UI**.
- **Client**: The client can now **run on the web**. <!-- Remove this if not done by release -->

#### Other

- **API**: Kinematic bodies are now exposed. This is used by the minigolf example to provide its moving obstacles.
- **API**: Added `physics::move_character` function to correctly move character controllers. This is used by the third-person camera example.
- **API**: `UVec2`/`UVec3`/`UVec4` can now be used for component values.
- **Client**: The client's window title is now automatically changed to the name of the project running on the server. Thanks to [@MavethGH](https://github.com/MavethGH) for implementing this in [#178](https://github.com/AmbientRun/Ambient/pull/178).
- **Client**: Added a basic headless mode to enable automatic CI testing of projects.
- **Client**: Added `Dump UI World` button to inspect the state of the UI. Thanks to [@owenpalmer](https://github.com/owenpalmer) for implementing this in [#216](https://github.com/AmbientRun/Ambient/pull/216).

### Changed

#### Breaking

- **Client**: `--debug` is now `--debugger`, but it can also be accessed through `AMBIENT_DEBUGGER`.
- **API**: `player_camera` has been removed, and the components it instantiated are now directly exposed. See the `multiplayer` example to see what's changed.

#### Non-breaking

- **Ambient**: Ambient is now dual-licensed MIT/Apache2, in accordance with the rest of the Rust ecosystem.
- **Ambient**: The default logging settings now better communicate what Ambient is doing at any given moment.
- **Project**: Concept definitions in projects now support namespaces. Thanks to [@ArberSephirotheca](https://github.com/ArberSephirotheca) for implementing this in [#212](https://github.com/AmbientRun/Ambient/pull/212).
- **API**: Concepts now include the components they use in their doc comments.

### Fixed

- **Ambient**: Various stability and performance fixes.
- **Ambient**: Added attributions for external code.
- **Ambient**: Typo fixes. Thanks for the following!
  - [#159: fix: docs broken links to gh-pages](https://github.com/AmbientRun/Ambient/pull/159) by [@daniellavoie](https://github.com/daniellavoie)
  - [#172: chore: fix typo in gpu.rs](https://github.com/AmbientRun/Ambient/pull/172) by [@eltociear](https://github.com/eltociear)
- **Examples**: The Minigolf example now has several gameplay tweaks (including camera movement on right-click) to improve the experience.
- **Examples**: The examples no longer occasionally use non-one alpha colours, which led to them rendering black objects.
- **Server**: The server no longer shuts down automatically after a period of inactivity.
- **ECS**: A bug with ECS component versioning that led to certain components not updating has been fixed. Fixes [#113](https://github.com/AmbientRun/Ambient/issues/113).

### Community PRs to internals

These PRs are not directly user-facing, but improve the development experience. They're just as appreciated!

- **CI**: Linux CI builds now output the tree of their target to assist in debugging CI cache blow-up. Thanks to [@daniellavoie](https://github.com/daniellavoie) for implementing this in [#170](https://github.com/AmbientRun/Ambient/pull/170).
- **ECS**: `Entity::assert_all` can be used to ensure all components for an `Entity` on the host have an attribute. Thanks to [@MavethGH](https://github.com/MavethGH) for implementing this in [#211](https://github.com/AmbientRun/Ambient/pull/211).
- **App**: `ambient new` uses the correct path for relative API when creating a project in `guest/rust/examples`. Thanks to [@owenpalmer](https://github.com/owenpalmer) for implementing this in [#218](https://github.com/AmbientRun/Ambient/pull/218).
- **Ambient**: The presentation of the license in the repository was improved. Thanks to [@C-BJ](https://github.com/C-BJ) for [#201](https://github.com/AmbientRun/Ambient/pull/201) and [#203](https://github.com/AmbientRun/Ambient/pull/203).
- **Ambient**: The book and build CI workflows now only run when relevant files are updated. Thanks to [@C-BJ](https://github.com/C-BJ) for implementing this in [#202](https://github.com/AmbientRun/Ambient/pull/202).

<!-- ### Removed -->

## Version 0.1.1 (2023-02-22)

### Added

- A [minigolf example](guest/rust/examples/minigolf) by [SK83RJOSH](https://github.com/SK83RJOSH).
- Examples are now bundled into a downloadable `examples.zip` for each release.

### Fixed

- macOS ARM64 builds are now available after enabling the execution of unsigned executable memory (as required for wasmtime execution).
- The debugging configuration for VSCode was updated to use the new CLI.
- Minor documentation updates.

## Version 0.1.0 (2023-02-22)

Initial release. See the [announcement blog post](https://www.ambient.run/post/introducing-ambient).
