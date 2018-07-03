use units::{ConcUnit, VolUnit, to_si};
use mifc::MIFC;
use traits::SI;

#[derive(Debug, Fail)]
pub enum SD3Error {
    #[fail(display = "row had a non-empty Exclude column")]
    Excluded,
    #[fail(display = "row did not have associated normalization info columns")]
    NoInfo,
    #[fail(display = "row did not have an entered Value")]
    NoValue,
    #[fail(display = "row did not have an entered Value Unit")]
    NoValueUnit,
}
//TODO: Deserialize optional string fields with a null || "" = None checking function
#[derive(Debug, Serialize, Deserialize)]
pub struct SD3 {
    #[serde(flatten)]
    mifc: MIFC,
    #[serde(flatten)]
    normal_info: Option<Normalization>,
}

impl SD3 {
    pub fn into_normalized(self) -> Result<MIFC, SD3Error> {
        if let Some(ref f) = self.mifc.exclude {
            if f != "" { return Err(SD3Error::Excluded) }
        }
        let value = self.mifc.value.ok_or(SD3Error::NoValue)?;
        let value_unit = self.mifc.value_unit.ok_or(SD3Error::NoValueUnit)?;
        // TODO: can self.normal_info be mapped like above? need fields to make note
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
        let mut normalized_mifc = self.mifc;

        normalized_mifc.value = Some(norm_val);
        normalized_mifc.value_unit = Some(ConcUnit::ng_day_millioncells);
        //TODO: Add to normalization info to the notes field?

        Ok(normalized_mifc)
    }
}

#[derive(Debug, Serialize, Deserialize)]
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

    // first go from the concentration (g/L) and sample volume (L) 
    // into grams/day/cell
    let gdaycell = si_val * si_vol / sample_days / cells;
    // now, convert to the output ng/day/10^6 cells 
    gdaycell * 1_000_000_000.0 * 1_000_000.0
}




