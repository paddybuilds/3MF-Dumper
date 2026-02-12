# 3MF Discrepancy Report (Current `test.3mf`)

Comparison target:
- Baseline Bambu project extraction: `decompiled/`
- Generated file extraction: `decompiled_current/` (from current `test.3mf`)

Generated: 2026-02-11

## 1) What Was Decompiled

Current `test.3mf` contains only 3 entries:
- `[Content_Types].xml`
- `_rels/.rels`
- `3D/3dmodel.model`

Model summary:
- objects: 1
- mesh objects: 1
- build items: 1
- metadata entries: 1
- unit: `millimeter`

## 2) Missing Package Parts vs Bambu Project Baseline

These files exist in `decompiled/` but are absent in `decompiled_current/`:
- `3D/Objects/object_1.model`
- `3D/_rels/3dmodel.model.rels`
- `Auxiliaries/.thumbnails/thumbnail_3mf.png`
- `Auxiliaries/.thumbnails/thumbnail_middle.png`
- `Auxiliaries/.thumbnails/thumbnail_small.png`
- `Auxiliaries/Model Pictures/20241020_141740.webp`
- `Auxiliaries/Model Pictures/20241020_142056.webp`
- `Auxiliaries/Model Pictures/20241020_142236.webp`
- `Auxiliaries/Model Pictures/20250127_175444.webp`
- `Auxiliaries/Profile Pictures/20241020_141740.webp`
- `Metadata/cut_information.xml`
- `Metadata/model_settings.config`
- `Metadata/pick_1.png`
- `Metadata/plate_1.png`
- `Metadata/plate_no_light_1.png`
- `Metadata/project_settings.config`
- `Metadata/slice_info.config`
- `Metadata/top_1.png`
- `Metadata/_rels/model_settings.config.rels`

Interpretation:
- Your file is a minimal core 3MF model.
- It does not include the Bambu Studio project bundle (`Metadata/*`, previews, slicer configs, optional gcode relations).

## 3) XML-Level Compatibility Gaps

In `decompiled_current/3D/3dmodel.model`:
- Root attribute uses `lang="en-US"` instead of `xml:lang="en-US"`.
- `<metadata>` uses a `value` attribute: `<metadata name="Application" value="LithophaneLabs" />`.
  - Bambu baseline uses text content: `<metadata name="Application">BambuStudio-...</metadata>`.
- Missing Bambu namespaces/flags from baseline root:
  - `xmlns:BambuStudio="http://schemas.bambulab.com/package/2021"`
  - `xmlns:p="http://schemas.microsoft.com/3dmanufacturing/production/2015/06"`
  - `requiredextensions="p"`

## 4) Missing Bambu Metadata Keys

Baseline has 28 metadata keys; generated file has only `Application`.

Notable missing keys include:
- `BambuStudio:3mfVersion`
- `Thumbnail_Middle`
- `Thumbnail_Small`
- `Title`
- `CreationDate`
- `ModificationDate`
- `Designer`

Interpretation:
- Without these keys and companion files, Bambu Studio can still potentially import geometry, but it will not behave like a native Bambu project package.

## 5) What To Add for Bambu Studio Compatibility

If you only need model import:
- Keep current 3-file structure, but fix XML shape:
  - use `xml:lang`
  - write metadata values as element text, not `value=...`

If you need Bambu project-level compatibility (open with plate/settings/previews intact):
- Add `Metadata/model_settings.config`
- Add `Metadata/project_settings.config`
- Add `Metadata/slice_info.config`
- Add preview images (`Metadata/*.png` and/or `Auxiliaries/.thumbnails/*`)
- Add root relationships for thumbnails in `_rels/.rels`
- Add Bambu metadata keys in `3D/3dmodel.model` (at least `BambuStudio:3mfVersion`, thumbnail pointers, title/info)
- Optionally add `Metadata/_rels/model_settings.config.rels` if referencing gcode artifacts
