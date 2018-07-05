use si::{SIUnit};

#[derive(Debug, Serialize, Deserialize)]
pub struct MIFC {
    #[serde(rename = "Chip ID")]
    pub id: String,
    #[serde(rename = "Assay Plate ID")]
    pub assay_plate_id: Option<String>,
    #[serde(rename = "Assay Well ID")]
    pub assay_well_id: Option<String>,
    #[serde(rename = "Method/Kit")]
    pub method: String,
    #[serde(rename = "Target/Analyte")]
    pub target: String,
    #[serde(rename = "Subtarget")]
    pub subtarget: Option<String>,
    #[serde(rename = "Sample Location")]
    pub sample_loc: String,
    #[serde(rename = "Day")]
    pub day: f64,
    #[serde(rename = "Hour")]
    pub hour: f64,
    #[serde(rename = "Minute")]
    pub min: f64,
    #[serde(rename = "Value")]
    pub value: Option<f64>,
    #[serde(rename = "Value Unit")]
    pub value_unit: Option<SIUnit>, 
    #[serde(rename = "Caution Flag")]
    pub flag: Option<String>,
    #[serde(rename = "Exclude")]
    pub exclude: Option<String>,
    #[serde(rename = "Notes")]
    pub notes: Option<String>,
    #[serde(rename = "Replicate")]
    pub replicate: Option<f32>,
    #[serde(rename = "Cross Reference")]
    pub xref: Option<String>,
}
