#[allow(non_snake_case)]
pub mod Manifest {
    use std::{fs, io};

    use crate::sky::CloudPlugin;
    use crate::utils::format_js;
    use densky_adapter::utils::join_paths;
    use densky_adapter::{CloudManifestUpdate, CompileContext};
    use optimized_tree::OptimizedTreeContainer;

    /// Generate TS code for this node and children
    fn build_node(
        id: u64,
        plugin: &CloudPlugin,
        container: &OptimizedTreeContainer,
    ) -> CloudManifestUpdate {
        let mut updates = CloudManifestUpdate::new();
        let node = container.nodes.get_reader(id).unwrap();

        let mut static_children = String::new();
        let mut children = String::new();

        for (pathname, id) in node.static_children.iter() {
            let children_update = build_node(*id, plugin, container);
            let children_content = &String::new();
            let children_content = match children_update.content() {
                Some(c) => c,
                None => children_content,
            };

            const EXTRA_LEN: usize = "\"".len() + "\": () => {".len() + "},".len();

            static_children.reserve_exact(pathname.len() + children_content.len() + EXTRA_LEN);
            static_children.push('"');
            static_children += pathname;
            static_children.push_str("\": () => {");
            static_children += children_content;
            static_children.push('}');
            static_children.push(',');

            updates += children_update;
        }

        for (_, id) in node.dynamic_children.iter() {
            let children_update = build_node(*id, plugin, container);
            children += &children_update.content().unwrap_or(&String::new());

            updates += children_update;
        }

        let dynamic_child = if let Some((id, varname)) = node.dynamic.as_ref() {
            let mut child = container.nodes.get_writer(*id).unwrap();
            child.varname = Some(varname.clone());
            drop(child);

            build_node(*id, plugin, container)
        } else {
            CloudManifestUpdate::new()
        };

        updates += &dynamic_child;

        let binding = String::new();
        let dynamic_child = dynamic_child.content().unwrap_or(&binding);

        let leaf = node.into_leaf(container);
        let plugin_updates = unsafe {
            plugin
                .cloud_optimized_manifest_call(
                    leaf,
                    static_children,
                    children,
                    dynamic_child.to_string(),
                )
                .unwrap()
        };

        updates += &plugin_updates;

        updates = if let Some(content) = &plugin_updates.content() {
            updates.set_content(content.to_string())
        } else {
            updates
        };

        updates
    }

    /// Generate a manifest file from a container.
    fn build(plugin: &CloudPlugin, container: &OptimizedTreeContainer) -> String {
        let mut imports = String::new();
        let mut args = String::new();
        let mut content = String::new();

        let before_updates = unsafe { plugin.cloud_before_manifest() };

        if let Ok(before_updates) = before_updates {
            before_updates.build_update(&mut imports, &mut args, &mut content)
        }

        let root = build_node(container.get_root_id().unwrap(), plugin, &container);
        root.build_update(&mut imports, &mut args, &mut content);

        format_js(format!(
            "
// This file is generated by Densky-Framework
// manifest.ts
{imports}

export default function({args}) {{
    {content}
}}"
        ))
    }

    /// Generate and write a manifest file from a container
    pub fn update(
        container: &OptimizedTreeContainer,
        plugin: &CloudPlugin,
        context: &CompileContext,
    ) -> io::Result<()> {
        let manifest = build(plugin, container);

        fs::write(join_paths("manifest.ts", &context.output_dir), manifest)
    }
}
