//! Scene + collection hierarchy extraction and name decoding.

use super::types::*;
use crate::phx::Result;

impl BlendFile {
    /// Extract scene information from the parsed file
    pub fn extract_scene(&self) -> Result<BlendScene> {
        let mut scene = BlendScene {
            name: "Scene".to_string(),
            collections: Vec::new(),
            objects: Vec::new(),
            active_object: None,
            selected: Vec::new(),
        };

        let dna = match &self.dna {
            Some(d) => d,
            None => return Ok(scene),
        };
        let ptr_size = self.header.pointer_size as usize;
        let little_endian = self.header.little_endian;

        // Name tracking for uniqueness
        let mut name_counts = std::collections::HashMap::new();
        let mut unique_objs = Vec::new();

        // Find the SC scene block
        if let Some(scene_block) = self.blocks.iter().find(|b| b.code_str == "SC") {
            let sc_name = self
                .extract_name_from_block(scene_block, "id.name")
                .unwrap_or_else(|_| "Scene".to_string());
            if !sc_name.is_empty() {
                scene.name = sc_name;
            }

            if let Some(master_offset) =
                Self::struct_field_offset("Scene", "master_collection", dna, ptr_size)
            {
                let master_addr =
                    Self::read_ptr(&scene_block.data, master_offset, ptr_size, little_endian);
                if let Some(master_collection) = self.extract_collection_hierarchy(
                    master_addr,
                    dna,
                    ptr_size,
                    little_endian,
                    &mut unique_objs,
                    0,
                ) {
                    scene.collections.push(master_collection);
                }
            }

            // Fallback for older Blender or if master_collection failed: Extract all OB blocks
            if unique_objs.is_empty() {
                for block in self.blocks.iter().filter(|b| b.code_str == "OB") {
                    if let Ok(Some(obj)) = self.extract_object(block) {
                        unique_objs.push(obj);
                    }
                }
            }
        }

        // Apply unique names with suffixes (e.g. Cube.001)
        for mut obj in unique_objs {
            let count = name_counts.entry(obj.name.clone()).or_insert(0);
            if *count > 0 {
                obj.name = format!("{}.{:03}", obj.name, count);
            }
            *count += 1;
            scene.objects.push(obj);
        }

        Ok(scene)
    }

    /// Recursively extract collection hierarchy.
    ///
    /// `depth` guards against malformed/cyclic pointers causing unbounded
    /// recursion and stack overflow on large files.
    pub(crate) fn extract_collection_hierarchy(
        &self,
        addr: u64,
        dna: &DnaType,
        ptr_size: usize,
        little_endian: bool,
        flat_objects: &mut Vec<BlendSceneObject>,
        depth: u32,
    ) -> Option<BlendCollection> {
        if addr == 0 {
            return None;
        }
        if depth > 64 {
            return None;
        }
        let block = self.blocks.iter().find(|b| b.old_address == addr)?;

        let mut col_name = self
            .extract_name_from_block(block, "id.name")
            .unwrap_or_else(|_| "Unknown".to_string());
        if col_name.is_empty() {
            col_name = "Scene Collection".to_string();
        }

        let mut collection = BlendCollection {
            name: col_name,
            object_names: Vec::new(),
            object_indices: Vec::new(),
            children: Vec::new(),
            is_instance: false,
            instance_targets: Vec::new(),
        };

        // Read children
        if let Some(children_offset) =
            Self::struct_field_offset("Collection", "children", dna, ptr_size)
        {
            let mut curr_child_addr =
                Self::read_ptr(&block.data, children_offset, ptr_size, little_endian);
            while curr_child_addr != 0 {
                if let Some(child_block) = self
                    .blocks
                    .iter()
                    .find(|b| b.old_address == curr_child_addr)
                {
                    if let Some(col_ptr_offset) =
                        Self::struct_field_offset("CollectionChild", "collection", dna, ptr_size)
                    {
                        let next_col_addr = Self::read_ptr(
                            &child_block.data,
                            col_ptr_offset,
                            ptr_size,
                            little_endian,
                        );
                        if let Some(child_col) = self.extract_collection_hierarchy(
                            next_col_addr,
                            dna,
                            ptr_size,
                            little_endian,
                            flat_objects,
                            depth + 1,
                        ) {
                            collection.children.push(child_col);
                        }
                    }
                    if let Some(next_ptr_offset) =
                        Self::struct_field_offset("CollectionChild", "next", dna, ptr_size)
                    {
                        curr_child_addr = Self::read_ptr(
                            &child_block.data,
                            next_ptr_offset,
                            ptr_size,
                            little_endian,
                        );
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            }
        }

        // Read objects
        if let Some(gobject_offset) =
            Self::struct_field_offset("Collection", "gobject", dna, ptr_size)
        {
            let mut curr_obj_addr =
                Self::read_ptr(&block.data, gobject_offset, ptr_size, little_endian);
            while curr_obj_addr != 0 {
                if let Some(obj_block) = self.blocks.iter().find(|b| b.old_address == curr_obj_addr)
                {
                    if let Some(ob_ptr_offset) =
                        Self::struct_field_offset("CollectionObject", "ob", dna, ptr_size)
                    {
                        let actual_ob_addr =
                            Self::read_ptr(&obj_block.data, ob_ptr_offset, ptr_size, little_endian);
                        if let Some(ob_block) =
                            self.blocks.iter().find(|b| b.old_address == actual_ob_addr)
                        {
                            let ob_name = self
                                .extract_name_from_block(ob_block, "id.name")
                                .unwrap_or_default();

                            // Add object name to collection
                            if !ob_name.is_empty() {
                                collection.object_names.push(ob_name.clone());
                            }

                            // Extract full object data and add to flat list if not already there,
                            // then record its flat index against this collection.
                            let idx = match flat_objects.iter().position(|o| o.name == ob_name) {
                                Some(i) => Some(i),
                                None => {
                                    if let Ok(Some(extracted_obj)) =
                                        self.extract_object_simple(ob_block)
                                    {
                                        flat_objects.push(extracted_obj);
                                        Some(flat_objects.len() - 1)
                                    } else {
                                        None
                                    }
                                }
                            };
                            if let Some(i) = idx {
                                collection.object_indices.push(i);
                            }
                        }
                    }
                    if let Some(next_ptr_offset) =
                        Self::struct_field_offset("CollectionObject", "next", dna, ptr_size)
                    {
                        curr_obj_addr = Self::read_ptr(
                            &obj_block.data,
                            next_ptr_offset,
                            ptr_size,
                            little_endian,
                        );
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            }
        }

        Some(collection)
    }

    /// Extract a name field from a block (ID structure name[66] in Blender 4.x)
    pub(crate) fn extract_name_from_block(
        &self,
        block: &BlendBlock,
        _field_name: &str,
    ) -> Result<String> {
        // Blender ID structure starts with name field (66 bytes in 4.x, 64 in older versions)
        if block.data.len() < 66 {
            return Ok("Unknown".to_string());
        }

        let mut name = String::new();

        // Try first 66 bytes (Blender 4.x format)
        let name_bytes = &block.data[0..66];
        let end = name_bytes.iter().position(|&b| b == 0).unwrap_or(66);
        let extracted = String::from_utf8_lossy(&name_bytes[0..end]).to_string();

        if !extracted.is_empty() {
            name = extracted;
        } else {
            // Fallback: scan for first printable name in first 200 bytes
            let search_len = block.data.len().min(200);
            for i in 0..search_len.saturating_sub(2) {
                // Look for pattern: printable, printable, null
                let b0 = block.data[i];
                let b1 = block.data[i + 1];
                let b2 = block.data[i + 2];
                if b0 >= 32 && b0 <= 126 && b1 >= 32 && b1 <= 126 && b2 == 0 {
                    let name_end = i + 2;
                    let name_start = i.saturating_sub(62); // 64 max length
                    let mut actual_start = name_start;
                    for j in (name_start..=i).rev() {
                        if block.data[j] == 0 {
                            actual_start = j + 1;
                            break;
                        }
                    }
                    if actual_start < name_end {
                        let candidate =
                            String::from_utf8_lossy(&block.data[actual_start..name_end])
                                .to_string();
                        if candidate.len() > 1
                            && candidate
                                .chars()
                                .all(|c| c.is_ascii_alphanumeric() || " _.-()[]{}".contains(c))
                        {
                            name = candidate;
                            break;
                        }
                    }
                }
            }
        }

        // Strip Blender's internal 2-character ID prefix (OB, GR, SC, ME, MA)
        if name.len() > 2
            && (name.starts_with("OB")
                || name.starts_with("GR")
                || name.starts_with("SC")
                || name.starts_with("MA")
                || name.starts_with("ME"))
        {
            Ok(name[2..].to_string())
        } else if !name.is_empty() {
            Ok(name)
        } else {
            Ok("Unknown".to_string())
        }
    }
}
