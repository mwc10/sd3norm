use units::{ConcUnit, VolUnit, to_si};
use failure::Error;
use traits::SI;

#[derive(Debug, Deserialize)]
pub struct SD3 {
    #[serde(rename = "Chip ID")]
    id: String,
    #[serde(rename = "Assay Plate ID")]
    assay_plate_id: Option<String>,
    #[serde(rename = "Assay Well ID")]
    assay_well_id: Option<String>,
    #[serde(rename = "Method/Kit")]
    method: String,
    #[serde(rename = "Target/Analyte")]
    target: String,
    #[serde(rename = "Subtarget")]
    subtarget: Option<String>,
    #[serde(rename = "Sample Location")]
    sample_loc: String,
    #[serde(rename = "Day")]
    day: f64,
    #[serde(rename = "Hour")]
    hour: f64,
    #[serde(rename = "Minute")]
    min: f64,
    #[serde(rename = "Value")]
    value: f64,
    #[serde(rename = "Value Unit")]
    value_unit: ConcUnit, 
    #[serde(rename = "Caution Flag")]
    flag: Option<String>,
    #[serde(rename = "Exclude")]
    exclude: Option<String>,
    #[serde(rename = "Notes")]
    notes: Option<String>,
    #[serde(rename = "Replicate")]
    replicate: Option<f32>,
    #[serde(rename = "Cross Reference")]
    xref: Option<String>,
    #[serde(flatten)]
    normal_info: Option<Normalization>,
}

impl SD3 {
    pub fn into_normalized(mut self) -> Result<Self, Error> {
        if let Some(ref f) = self.exclude {
            if f != "" { bail!("Excluded row") }
        }
        {
        let info = if let Some(i) = self.normal_info
            { i } else {
                bail!("Missing necessary information for normalization:\n {:#?}", self)
            };

        let sample_time = info.sample_days 
                          + (info.sample_hours/24.0) 
                          + (info.sample_minutes/(24.0*60.0));
        let norm_val = to_ngday_millioncells(
            self.value, self.value_unit, 
            info.sample_volume, info.sample_vol_unit, 
            sample_time, info.cell_count);
        
        self.value = norm_val;
        }
        self.value_unit = ConcUnit::ng_day_millioncells;
        //Add to notes field?
        self.normal_info = None;


        Ok(self)
    }
}

#[derive(Debug, Deserialize)]
struct Normalization {
    #[serde(rename = "Duration Sample Collection (days)")]
    sample_days: f64,
    #[serde(rename = "Duration Sample Collection (hours)")]
    sample_hours: f64,
    #[serde(rename = "Duration Sample Collection (minutes)")]
    sample_minutes: f64,
    #[serde(rename = "Sample Volume")]
    sample_volume: f64,
    #[serde(rename = "Sample Volume Unit")]
    sample_vol_unit: VolUnit,
    #[serde(rename = "Estimated Cell Number")]
    cell_count: f64,
}

fn to_ngday_millioncells<V, S>(
    val: f64, val_unit: V, 
    vol: f64, vol_unit: S, 
    sample_days: f64, cells: f64
) -> f64
where V: SI, S: SI
{
    let si_val = to_si(val, val_unit);
    let si_vol = to_si(vol, vol_unit);
    println!("val {}\nvol {}", si_val, si_vol);

    // first go from the concentration (g/L) and sample volume (L) 
    // into grams/day/cell
    let gdaycell = si_val * si_vol / sample_days / cells;
    // now, convert to the output ng/day/10^6 cells 
    gdaycell * 1_000_000_000.0 * 1_000_000.0
}




