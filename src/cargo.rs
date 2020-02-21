use serde::Serialize;

use crate::errors::CargoPlayError;
use crate::opt::RustEdition;

#[derive(Clone, Debug, Serialize)]
struct CargoPackage {
    name: String,
    version: String,
    edition: String,
}

impl CargoPackage {
    fn new(name: String, edition: RustEdition) -> Self {
        Self {
            name: name.to_lowercase(),
            version: "0.1.0".into(),
            edition: edition.into(),
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct CargoManifest {
    package: CargoPackage,
    #[serde(flatten)]
    dependencies: toml::value::Table,
}

impl CargoManifest {
    pub(crate) fn new(
        name: String,
        dependencies: Vec<String>,
        edition: RustEdition,
    ) -> Result<Self, CargoPlayError> {
        let dependencies: toml::Value = format!("[dependencies]\n{}", dependencies.join("\n"))
            .parse()
            .map_err(CargoPlayError::from_serde)?;

        let dependencies = dependencies.try_into().unwrap();
        Ok(Self {
            package: CargoPackage::new(name, edition),
            dependencies,
        })
    }
}
