use si::{SIUnit, self};
use mifc::MIFC;

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
        let info = self.normal_info.ok_or(SD3Error::NoInfo)?;

        let sample_time = info.calc_sample_time();
        let norm_val = to_ngday_millioncells(value, value_unit, &info);

        let mut normalized_mifc = self.mifc;
        let note = format!("Normalized from {v:.4} {vu} by a {s} {su} sample over {d} {ds} with an estimated {c} cells ", 
            v = value, vu = value_unit,
            s = info.sample_volume, su = info.sample_vol_unit,
            d = sample_time, ds = if sample_time > 1.0 {"days"} else {"day"},
            c = info.cell_count
        );

        normalized_mifc.value = Some(norm_val);
        normalized_mifc.value_unit = Some(SIUnit::ng_day_millioncells);        
        normalized_mifc.notes = if let Some(mut n) = normalized_mifc.notes {
            if &n != "" { n.push_str(" || "); }
            n.push_str(&note);
            Some(n)
        } else {
            Some(note)
        };

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
    sample_vol_unit: SIUnit,
    #[serde(rename = "Estimated Cell Number")]
    cell_count: f64,
}

impl Normalization {
    /// Calculate the duration of the sample in terms of days
    #[inline]
    fn calc_sample_time(&self) -> f64 {
        self.sample_days 
        + (self.sample_hours/24.0) 
        + (self.sample_minutes/(24.0*60.0))
    }
}

fn to_ngday_millioncells(val: f64, val_unit: SIUnit, norm: &Normalization) -> f64
{
    use self::SIUnit::*;

    let &Normalization{cell_count: cells, sample_volume: vol, sample_vol_unit: vol_unit, ..} = norm;

    let days = norm.calc_sample_time();
    let si_val = si::convert((val, val_unit), g_l).unwrap();
    let si_vol = si::convert((vol, vol_unit), l).unwrap();
    trace!("conc: {:.5} {} to SI {:.5} {}", val, val_unit, si_val, g_l);
    trace!("vol: {:.5} {} to SI {:.5} {}",  vol, vol_unit, si_vol, l);

    // first go from the concentration (g/L) and sample volume (L) 
    // into nanograms/day/cell
    let made_ng = si::convert((si_val * si_vol, g), ng).unwrap();
    trace!("produced ng: {:.5} over {:.3} day(s)", made_ng, days);
    let ngdaycell = made_ng / days / cells;
    // now, multiple by 10^6 to make rate by million cells 
    ngdaycell * 1_000_000.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use utils::double_comparable;
    use si::SIUnit::*;

    struct Norm {
        val: f64,
        val_unit: SIUnit,
        info: Normalization,
    }

    static INPUTS: [Norm; 10] = [
        Norm {
            val: 153.914,
            val_unit: ng_ml,
            info: Normalization{
                sample_days: 1.0,
                sample_hours: 0.0,
                sample_minutes: 0.0,
                sample_volume: 200.0,
                sample_vol_unit: ul,
                cell_count: 16768.0,
            }
        },
        Norm {
            val: 1360.2953,
            val_unit: ng_ml,
            info: Normalization{
                sample_days: 1.0,
                sample_hours: 0.0,
                sample_minutes: 0.0,
                sample_volume: 200.0,
                sample_vol_unit: ul,
                cell_count: 16768.0,
            }
        },
        Norm {
            val: 1071.288,
            val_unit: ng_ml,
            info: Normalization{
                sample_days: 1.0,
                sample_hours: 0.0,
                sample_minutes: 0.0,
                sample_volume: 300.0,
                sample_vol_unit: ul,
                cell_count: 80000.0,
            }
        },
        Norm {
            val: 1543.054,
            val_unit: mg_dl,
            info: Normalization{
                sample_days: 1.0,
                sample_hours: 0.0,
                sample_minutes: 0.0,
                sample_volume: 200.0,
                sample_vol_unit: ul,
                cell_count: 50000.0,
            }
        },
        Norm {
            val: 484321.0,
            val_unit: mg_dl,
            info: Normalization{
                sample_days: 2.0,
                sample_hours: 5.0,
                sample_minutes: 0.0,
                sample_volume: 500.0,
                sample_vol_unit: ul,
                cell_count: 50000.0,
            }
        },
        Norm {
            val: 15.9,
            val_unit: mg_dl,
            info: Normalization{
                sample_days: 0.0,
                sample_hours: 20.0,
                sample_minutes: 2.0,
                sample_volume: 100.0,
                sample_vol_unit: ul,
                cell_count: 50000.0,
            }
        },
        Norm {
            val: 0.87,
            val_unit: mg_dl,
            info: Normalization{
                sample_days: 0.0,
                sample_hours: 10.0,
                sample_minutes: 30.0,
                sample_volume: 0.1,
                sample_vol_unit: ml,
                cell_count: 50000.0,
            }
        },
        Norm {
            val: 542.0,
            val_unit: pg_ml,
            info: Normalization{
                sample_days: 3.0,
                sample_hours: 15.0,
                sample_minutes: 1.0,
                sample_volume: 200.0,
                sample_vol_unit: ul,
                cell_count: 20000.0,
            }
        },
        Norm {
            val: 12.0556,
            val_unit: pg_ml,
            info: Normalization{
                sample_days: 1.0,
                sample_hours: 0.0,
                sample_minutes: 0.0,
                sample_volume: 0.1,
                sample_vol_unit: ml,
                cell_count: 20000.0,
            }
        },
        Norm {
            val: 0.00465,
            val_unit: pg_ml,
            info: Normalization{
                sample_days: 0.0,
                sample_hours: 2.0,
                sample_minutes: 30.0,
                sample_volume: 0.01,
                sample_vol_unit: l,
                cell_count: 20000.0,
            }
        },
    ];

    static OUTPUTS: [f64; 10] = [
        1835.801527,
        16224.89623,
        4017.33,
        61722160.0,
        21931516981.0,
        380965.0582,
        39771.42857,
        1.494886037,
        0.060278,
        0.02232,
    ];

    #[test]
    fn ng_day_cell_normalization() {
        const PERCENT_TOLERANCE: f64 = 0.001;

        let all_equal = INPUTS.iter()
            .map(|i| to_ngday_millioncells(i.val, i.val_unit, &i.info))
            .zip(OUTPUTS.iter())
            .enumerate()
            .inspect(|(i, (c, e))|
                println!("\nSet #{}\ncalculated: {} | expected: {}", i, c, e)
            )
            .all(|(_i,(c,e))|
                double_comparable(c, *e, PERCENT_TOLERANCE)
            );

        assert!(all_equal);
    }
}
