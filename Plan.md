# Plan: Convert `lowres.rs` to a Tauri v2 Module

This plan outlines the steps to integrate the existing `lowres.rs` image processing logic into your Tauri v2 application.

## 1. Backend (Rust/Tauri)

### 1.1. Add Dependencies

We need to add the image processing libraries used in `lowres.rs` to the Tauri project's `Cargo.toml`.

**Action:** Update `src-tauri/Cargo.toml` to include:

- `image`
- `rayon`
- `anyhow`
- `png`

### 1.2. Port `lowres.rs` Logic

We will move the core logic from `app/lowres.rs` into the Tauri project, refactoring it from a CLI tool to a library module.

**Action:** Create `src-tauri/src/lowres.rs`.

- Copy the logic from `app/lowres.rs`.
- Remove `clap` and CLI-specific code (`main`, `Args` struct derivation).
- Create a struct `LowresConfig` that derives `serde::Deserialize` to accept parameters from the frontend.
- Refactor `run` to accept `input_path`, `output_path`, and `config`.

### 1.3. Create Tauri Command

We need a command that the frontend can call to trigger the image processing.

**Action:** Define `#[tauri::command] fn process_image(...)` in `src-tauri/src/lib.rs` (or `lowres.rs`).

- This command will take the input file path and configuration options.
- It will determine an output path (e.g., `filename_lowres.png`).
- It will call the refactored `lowres` logic.
- It will return the output path to the frontend.

**Action:** Register the command in `src-tauri/src/lib.rs` inside the `tauri::generate_handler![]` macro.

### 1.4. Permissions (Optional but Recommended)

If the app needs to read/write files outside of standard locations, we might need to configure Tauri permissions. For now, we will assume the user selects files that the OS grants access to, or we use the `fs` scope if needed.

## 2. Frontend (Svelte)

### 2.1. Update UI

We will replace the default "Greet" UI with an image processor UI.

**Action:** Update `src/routes/+page.svelte`.

- Add a **Drag & Drop** zone.
- Add a **"Browse"** button (using `<input type="file">` or Tauri's dialog plugin if preferred, but standard input is easier for now).
- Add controls for `lowres` parameters (optional, or hardcode defaults for now).
- Display the **Original Image** and **Processed Image**.

### 2.2. Integrate with Tauri

**Action:** Implement the TypeScript logic in `+page.svelte`.

- Import `invoke` from `@tauri-apps/api/core`.
- Handle file selection.
- Call `invoke('process_image', { input: selectedFile, ... })`.
- Update the UI with the result.

## 3. Execution Steps

1.  **Modify `src-tauri/Cargo.toml`** to add dependencies.
2.  **Create `src-tauri/src/lowres.rs`** with the ported logic.
3.  **Update `src-tauri/src/lib.rs`** to register the module and command.
4.  **Update `src/routes/+page.svelte`** with the new UI and logic.
