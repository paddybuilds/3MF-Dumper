# 3MF Structure and Slicer Settings (Generic + Bambu Studio Notes)

This document explains the typical 3MF file structure, where slicer settings are stored, and how to distinguish standard 3MF content from slicer-specific extensions. It is written to be vendor-neutral but includes notes for Bambu Studio 3MF files where relevant.

## 1) High-Level 3MF Container Layout

A 3MF file is an OPC (Open Packaging Convention) zip container. Common entries:

- `[Content_Types].xml`
  - Declares MIME/content types for parts inside the package.
- `_rels/.rels`
  - Root relationships for the package.
  - Typically points to the main model at `/3D/3dmodel.model` and optionally to thumbnails.
- `3D/3dmodel.model`
  - The primary 3MF XML document.
  - Contains `<model>`, `<resources>`, and `<build>`.
- `3D/_rels/3dmodel.model.rels`
  - Relationships for the main model file, often linking to external object models in `3D/Objects/`.
- `3D/Objects/*.model`
  - Mesh data in `<mesh>` with `<vertices>` and `<triangles>`.
- `Metadata/`
  - Vendor-specific slicer configuration, previews, or project info.
- `Auxiliaries/`
  - Thumbnails or auxiliary images.

## 2) Core 3MF XML Elements (Standard)

These are defined by the 3MF core specification and are generally portable:

- `<model>`
  - Root element. Has units (`unit="millimeter"`) and language (`xml:lang`).
- `<resources>`
  - Contains `<object>` elements.
- `<object id="…" type="model">`
  - References mesh data or sub-components.
- `<mesh>`
  - Mesh geometry: `<vertices>` and `<triangles>`.
- `<build>`
  - Defines build items and their placement.
- `<item objectid="…" transform="…">`
  - Placement and scale of each build item.

These are the parts you should treat as standard 3MF, regardless of slicer.

## 3) Where Slicer Settings Live (Generic Guidance)

The 3MF core spec does not prescribe slicer settings. Slicer configuration is typically stored in one (or more) of these patterns:

- **Custom metadata** embedded in `3dmodel.model` via `<metadata>` tags.
- **Vendor-defined XML or JSON files** under `Metadata/`.
- **Vendor namespaces** in the main model file (custom `xmlns:Vendor=`).

Because this is not standardized, each slicer defines its own layout and keys.

## 4) Bambu Studio 3MF: What’s Standard vs. Exclusive

The following are common markers for Bambu Studio 3MF files:

### A) Bambu Studio Identifiers (Slicer-Specific)

- Namespace on the root model:
  - `xmlns:BambuStudio="http://schemas.bambulab.com/package/2021"`
- Metadata keys indicating the app and internal versioning:
  - `Application = BambuStudio-<version>`
  - `BambuStudio:3mfVersion = 1`
- Additional per-profile metadata inside `<metadata>`.

These keys are slicer-specific and not part of the 3MF core spec.

### B) Slicer Settings (Bambu Studio)

Bambu Studio stores full slicer configuration in `Metadata/project_settings.config` (JSON). Keys and settings here are slicer-defined. Examples of commonly relevant groups:

- Layering:
  - `layer_height`, `initial_layer_print_height`
- Walls and shells:
  - `wall_loops`, `top_shell_layers`, `bottom_shell_layers`
- Infill:
  - `sparse_infill_density`, `sparse_infill_pattern`
- Supports:
  - `enable_support`, `support_type`, `support_threshold_angle`
- Brim/adhesion:
  - `brim_type`, `brim_width`, `brim_object_gap`
- Temperatures:
  - `nozzle_temperature`, `nozzle_temperature_initial_layer`, `*_plate_temp`
- Cooling:
  - `fan_min_speed`, `fan_max_speed`, `fan_cooling_layer_time`
- Filament material:
  - `filament_diameter`, `filament_flow_ratio`, `filament_density`, `filament_colour`
- G-code blocks (Bambu-specific):
  - `machine_start_gcode`, `machine_end_gcode`, `filament_start_gcode`, `filament_end_gcode`, `change_filament_gcode`

These settings are not standardized 3MF elements; they are Bambu Studio configuration data.

### C) Plate and Object Mapping (Bambu Studio)

`Metadata/model_settings.config` (XML) typically includes:

- Per-object metadata (names, extruders)
- Plate info (thumbnails, plate IDs)
- Assembly transforms for instances

This file is slicer-specific and not part of core 3MF.

## 5) Practical Rules of Thumb

- Treat `/3D/3dmodel.model`, `/3D/Objects/*.model`, and `/_rels/` files as **standard 3MF**.
- Treat `Metadata/` contents and any non-core namespaces as **slicer-specific**.
- When extracting slicer settings, prefer:
  - `Metadata/project_settings.config` (Bambu Studio)
  - `Metadata/model_settings.config` (Bambu Studio)
  - Any non-core `<metadata>` fields in `3dmodel.model`

## 6) Suggested Parsing Strategy (Generic)

1. Read `/3D/3dmodel.model` first for core geometry and build items.
2. Follow `3D/_rels/3dmodel.model.rels` to resolve object models.
3. Scan `Metadata/` for slicer-specific configs.
4. Parse vendor namespaces and metadata keys for slicer identification.

## 7) Notes About Portability

- Slicer-specific settings may be ignored by other slicers.
- The mesh geometry will generally load in any 3MF-capable tool.
- Vendor metadata is useful for reproducing prints in the original slicer but should not be assumed portable.

