use units::{ConcUnit, VolUnit, to_si};
use traits::SI;

#[derive(Debug, Fail)]
pub enum SD3Error {
    #[fail(display = "Row had a non-empty Exclude column")]
    Excluded,
    #[fail(display = "Row did not have associated normalization info columns")]
    NoInfo,
    #[fail(display = "Row did not an entered Value")]
    NoValue,
    #[fail(display = "Row did not an entered Value Unit")]
    NoValueUnit,
}
//TODO: Deserialize optional string fields with a null || "" = None checking function
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
    value: Option<f64>,
    #[serde(rename = "Value Unit")]
    value_unit: Option<ConcUnit>, 
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
    pub fn into_normalized(mut self) -> Result<Self, SD3Error> {
        if let Some(ref f) = self.exclude {
            if f != "" { return Err(SD3Error::Excluded) }
        }
        let value = self.value.ok_or(SD3Error::NoValue)?;
        let value_unit = self.value_unit.ok_or(SD3Error::NoValueUnit)?;

        let norm_val = if let Some(info) = self.normal_info
        { 
            let sample_time = info.sample_days 
                          + (info.sample_hours/24.0) 
                          + (info.sample_minutes/(24.0*60.0));

            to_ngday_millioncells(
                value, value_unit, 
                info.sample_volume, info.sample_vol_unit, 
                sample_time, info.cell_count
            )
        } else {
            return Err(SD3Error::NoInfo)
        };

        self.value = Some(norm_val);
        self.value_unit = Some(ConcUnit::ng_day_millioncells);
        //Add to normalization info to the notes field?
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




