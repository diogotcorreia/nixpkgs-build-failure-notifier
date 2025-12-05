use anyhow::{Result, anyhow};
use brotli::Decompressor;
use struson::{
    json_path,
    reader::simple::{SimpleJsonReader, ValueReader, multi_json_path::multi_json_path},
};

pub fn fetch_packages_of_maintainers(maintainers: &[String]) -> Result<Vec<String>> {
    if maintainers.is_empty() {
        return Ok(vec![]);
    }

    let bytes =
        reqwest::blocking::get("https://channels.nixos.org/nixpkgs-unstable/packages.json.br")?;

    let decompressor = Decompressor::new(bytes, 4096);

    let deserializer = SimpleJsonReader::new(decompressor);

    // Not using serde because that saves everything to memory first
    let mut packages = vec![];
    deserializer
        .read_seeked(&json_path!["packages"], |value_reader| {
            value_reader.read_object_owned_names(|package_name, package_reader| {
                let mut include = false;
                package_reader.read_seeked_multi(
                    &multi_json_path![?"meta", ?"maintainers", [*], ?"github"],
                    false,
                    |reader| {
                        if maintainers.contains(&reader.read_string()?) {
                            include = true;
                        }
                        Ok(())
                    },
                )?;
                if include {
                    packages.push(package_name);
                }
                Ok(())
            })?;
            Ok(())
        })
        .map_err(|e| anyhow!("Failed to deserialize package metadata: {e}"))?;

    Ok(packages)
}
