use crate::error::GitronicsError;
use crate::model_config::{ConfigurationSource, ModelConfig};
use migjorn::Model;
use serde::Serialize;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs,
    io::Read,
    path::{Path, PathBuf},
};

#[derive(Debug)]
pub(crate) struct SourceFile {
    pub path: PathBuf,
    pub sha256: String,
    pub bytes: u64,
}

impl SourceFile {
    pub fn read(path: &Path) -> Result<(String, Self), GitronicsError> {
        let text =
            fs::read_to_string(path).map_err(|error| GitronicsError::io_path(path, error))?;
        let source = Self {
            path: path.to_path_buf(),
            sha256: format!("{:x}", Sha256::digest(text.as_bytes())),
            bytes: text.len() as u64,
        };
        Ok((text, source))
    }
}

#[derive(Debug, Serialize, Default)]
pub struct Evidence {
    pub inputs: Vec<InputFile>,
    pub configuration_chain: Vec<ConfigurationLayer>,
    pub assignment_origins: BTreeMap<String, String>,
    pub resolved_configuration: Value,
    pub full_commit: Option<String>,
    pub dirty: Option<bool>,
    pub content_fingerprint: String,
    pub inputs_unchanged: Option<bool>,
    pub output: Option<Artifact>,
    pub data_cards: Vec<DataCardEntry>,
    #[serde(skip)]
    paths: Vec<PathBuf>,
    #[serde(skip)]
    base: PathBuf,
}

#[derive(Debug, Serialize)]
pub struct InputFile {
    pub name: String,
    pub role: String,
    pub path: String,
    pub sha256: String,
    pub bytes: u64,
}

#[derive(Debug, Serialize)]
pub struct Artifact {
    pub path: String,
    pub sha256: String,
    pub bytes: u64,
}

#[derive(Debug, Serialize)]
pub struct ConfigurationLayer {
    pub path: String,
    pub values: Value,
}

#[derive(Debug, Serialize)]
pub struct DataCardEntry {
    pub name: String,
    pub category: &'static str,
    pub input: String,
    pub text: String,
    pub referenced_cell_runs: Vec<[i64; 2]>,
}

pub fn digest_file(path: &Path) -> Result<(String, u64), GitronicsError> {
    let mut file = fs::File::open(path).map_err(|error| GitronicsError::io_path(path, error))?;
    let mut digest = Sha256::new();
    let mut bytes = 0;
    let mut buffer = [0; 65536];
    loop {
        let count = file
            .read(&mut buffer)
            .map_err(|error| GitronicsError::io_path(path, error))?;
        if count == 0 {
            break;
        }
        digest.update(&buffer[..count]);
        bytes += count as u64;
    }
    Ok((format!("{:x}", digest.finalize()), bytes))
}

fn display_path(path: &Path, base: &Path) -> String {
    pathdiff::diff_paths(path, base)
        .unwrap_or_else(|| path.to_path_buf())
        .to_string_lossy()
        .replace('\\', "/")
}

impl Evidence {
    pub fn from_configuration(config: &ModelConfig, sources: Vec<ConfigurationSource>) -> Self {
        let base = sources[0].source.path.parent().unwrap().to_path_buf();
        let mut evidence = Self {
            base: base.clone(),
            ..Self::default()
        };
        for layer in sources {
            let relative = display_path(&layer.source.path, &base);
            evidence.record_input("configuration", &relative, layer.source);
            evidence.configuration_chain.push(ConfigurationLayer {
                path: relative,
                values: layer.values,
            });
        }
        evidence.configuration_chain.reverse();
        for layer in &evidence.configuration_chain {
            if let Some(envelopes) = layer.values.get("envelopes").and_then(Value::as_object) {
                for name in envelopes.keys() {
                    evidence
                        .assignment_origins
                        .insert(name.clone(), layer.path.clone());
                }
            }
        }
        evidence.resolved_configuration =
            serde_json::to_value(config).expect("configuration serializes");
        evidence.resolved_configuration["project_roots"] = Value::from(
            config
                .project_roots()
                .iter()
                .map(|root| {
                    display_path(
                        &dunce::canonicalize(root).unwrap_or_else(|_| root.clone()),
                        &base,
                    )
                })
                .collect::<Vec<_>>(),
        );
        if let Ok(repo) = git2::Repository::discover(&base) {
            evidence.full_commit = repo
                .head()
                .ok()
                .and_then(|head| head.peel_to_commit().ok())
                .map(|commit| commit.id().to_string());
            evidence.dirty = repo
                .statuses(None)
                .ok()
                .map(|statuses| !statuses.is_empty());
        }
        evidence
    }

    pub fn record_input(&mut self, role: &str, name: &str, source: SourceFile) {
        self.inputs.push(InputFile {
            name: name.into(),
            role: role.into(),
            path: display_path(&source.path, &self.base),
            sha256: source.sha256,
            bytes: source.bytes,
        });
        self.paths.push(source.path);
    }

    pub fn record_data_cards(&mut self, path: &Path, text: &str) {
        let model = Model::parse(&format!("data cards\n\n\n{text}\n"));
        for card in model.data_cards() {
            let Some(name) = card.name() else {
                continue;
            };
            let mnemonic = name
                .trim_end_matches(|character: char| character.is_ascii_digit())
                .to_ascii_lowercase();
            let category = match mnemonic.as_str() {
                "m" | "mt" | "mx" => "Materials",
                "tr" => "Transforms",
                "sdef" | "si" | "sp" | "sb" | "ds" | "sc" | "ssr" | "ssw" => "Source",
                "f" | "fm" | "fc" | "ft" | "fq" | "e" | "de" | "df" | "fmesh" | "fs" | "sd"
                | "fu" => "Tallies",
                "mode" | "nps" | "kcode" | "ksrc" | "cut" | "phys" | "rand" | "prdmp" | "print"
                | "lost" => "Run settings",
                _ => "Other",
            };
            self.data_cards.push(DataCardEntry {
                name: format!(
                    "{}{}{}",
                    if card.starred() { "*" } else { "" },
                    name.to_ascii_uppercase(),
                    card.particle()
                        .map(|particle| format!(":{particle}"))
                        .unwrap_or_default()
                ),
                category,
                input: display_path(path, &self.base),
                text: card.text().trim().to_owned(),
                referenced_cell_runs: vec![],
            });
        }
    }

    pub fn finish_inputs(&mut self, config: &ModelConfig) {
        let mut inputs: Vec<_> = std::mem::take(&mut self.inputs)
            .into_iter()
            .zip(std::mem::take(&mut self.paths))
            .collect();
        inputs.sort_by_cached_key(|(input, _)| {
            let rank = match input.role.as_str() {
                "configuration" => 0,
                "envelope_structure" => 1,
                "envelope_metadata" => 2,
                "filler" | "filler_metadata" => 3,
                "materials" => 4,
                "transforms" => 5,
                "tallies" => 6,
                _ => 7,
            };
            let names = match input.role.as_str() {
                "materials" => config.materials(),
                "transforms" => config.transformations(),
                "tallies" => config.tallies(),
                _ => &[],
            };
            (
                rank,
                if rank == 3 {
                    input.name.clone()
                } else {
                    String::new()
                },
                names
                    .iter()
                    .position(|name| **name == input.name)
                    .unwrap_or(0),
                input.role.clone(),
            )
        });
        (self.inputs, self.paths) = inputs.into_iter().unzip();
        self.data_cards.sort_by_cached_key(|card| {
            self.inputs
                .iter()
                .position(|input| input.path == card.input)
                .unwrap_or(usize::MAX)
        });
        let mut identities: Vec<_> = self
            .inputs
            .iter()
            .map(|input| (&input.role, &input.name, &input.path, &input.sha256))
            .collect();
        identities.sort();
        let content = serde_json::to_vec(&(
            env!("CARGO_PKG_VERSION"),
            identities,
            &self.resolved_configuration,
        ))
        .unwrap();
        self.content_fingerprint = format!("{:x}", Sha256::digest(&content));
    }

    pub fn verify_inputs(&mut self) -> bool {
        let unchanged = self.inputs.iter().zip(&self.paths).all(|(input, path)| {
            digest_file(path)
                .is_ok_and(|(hash, bytes)| hash == input.sha256 && bytes == input.bytes)
        });
        self.inputs_unchanged = Some(unchanged);
        unchanged
    }

    pub fn record_output(&mut self, path: &Path) -> Result<(), GitronicsError> {
        let (sha256, bytes) = digest_file(path)?;
        self.output = Some(Artifact {
            path: "assembled.mcnp".into(),
            sha256,
            bytes,
        });
        Ok(())
    }

    pub fn material_references(&mut self, model: &Model) {
        let mut references: BTreeMap<i64, Vec<i64>> = BTreeMap::new();
        for cell in model.cells() {
            if let (Some(material), Some(id)) = (cell.material(), cell.id()) {
                references.entry(material).or_default().push(id);
            }
        }
        for card in &mut self.data_cards {
            if let Some(id) = card
                .name
                .strip_prefix('M')
                .and_then(|id| id.parse::<i64>().ok())
            {
                card.referenced_cell_runs = crate::build_record::runs_from_ids(
                    references.get(&id).cloned().unwrap_or_default(),
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fingerprints_and_input_stability_use_loaded_contents() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("input");
        fs::write(&path, "abc").unwrap();
        let (text, source) = SourceFile::read(&path).unwrap();
        assert_eq!(
            source.sha256,
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
        assert_eq!(source.bytes, 3);
        assert_eq!(
            digest_file(&path).unwrap(),
            (source.sha256.clone(), source.bytes)
        );
        fs::write(&path, "abd").unwrap();
        let mut evidence = Evidence::default();
        evidence.record_input("test", "input", source);
        assert_eq!(text, "abc");
        assert!(!evidence.verify_inputs());
        fs::write(&path, "abc").unwrap();
        assert!(evidence.verify_inputs());
    }

    #[test]
    fn configuration_chain_records_loaded_assignment_origins_and_relative_paths() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join("base.yaml"),
            "envelope_structure: shell\nenvelopes:\n  inherited: null\n  replaced: null\n",
        )
        .unwrap();
        let path = dir.path().join("variant.yaml");
        fs::write(
            &path,
            "overrides: base.yaml\nenvelopes:\n  replaced: null\n",
        )
        .unwrap();
        let (config, sources) = ModelConfig::load_with_sources(&path).unwrap();
        fs::write(&path, "envelopes: {}\n").unwrap();
        let mut evidence = Evidence::from_configuration(&config, sources);
        evidence.finish_inputs(&config);
        assert_eq!(evidence.configuration_chain[0].path, "base.yaml");
        assert_eq!(evidence.configuration_chain[1].path, "variant.yaml");
        assert_eq!(evidence.assignment_origins["inherited"], "base.yaml");
        assert_eq!(evidence.assignment_origins["replaced"], "variant.yaml");
        assert!(!evidence.verify_inputs());
        assert!(
            evidence
                .inputs
                .iter()
                .all(|input| !Path::new(&input.path).is_absolute())
        );
    }
}
