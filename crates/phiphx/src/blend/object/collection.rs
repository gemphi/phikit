//! `Scene.master_collection` → `Collection.children` / `gobject` walk.

use crate::blend::file::types::*;

impl BlendFile {
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
        report: &dyn Fn(&Vec<BlendSceneObject>, &str) -> bool,
    ) -> Option<BlendCollection> {
        if addr == 0 {
            return None;
        }
        if depth > 64 || self.cancel_requested.load(std::sync::atomic::Ordering::Relaxed) {
            return None;
        }
        let block = self.block_at(addr)?;

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
            block_address: addr,
            exclude: false,
            hide_viewport: false,
            visibility: self.collection_visibility(block, dna),
        };

        // Read children
        if let Some(children_offset) =
            Self::struct_field_offset("Collection", "children", dna, ptr_size)
        {
            let mut curr_child_addr =
                Self::read_ptr(&block.data, children_offset, ptr_size, little_endian);
            while curr_child_addr != 0 {
                if let Some(child_block) = self.block_at(curr_child_addr)
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
                            report,
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
                if let Some(obj_block) = self.block_at(curr_obj_addr)
                {
                    if let Some(ob_ptr_offset) =
                        Self::struct_field_offset("CollectionObject", "ob", dna, ptr_size)
                    {
                        let actual_ob_addr =
                            Self::read_ptr(&obj_block.data, ob_ptr_offset, ptr_size, little_endian);
                        if let Some(ob_block) =
                            self.block_at(actual_ob_addr)
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
                                        if !report(flat_objects, &ob_name) {
                                            self.cancel_requested
                                                .store(true, std::sync::atomic::Ordering::Relaxed);
                                            return Some(collection);
                                        }
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
}
