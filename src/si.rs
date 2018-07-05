use std::fmt;
use std::str::FromStr;
use serde::de::{self, Visitor, Deserialize, Deserializer};
use serde::ser::{Serialize, Serializer};

#[derive(Debug, Fail)]
pub enum SIError {
    #[fail(display = "Cannot convert from {:?} to {:?}", _0, _1)]
    IncompatibleTypes(UnitType, UnitType),
    #[fail(display = "Unknown SI unit <{}>", _0)]
    UnkType(String),
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[allow(non_camel_case_types)]
pub enum SIUnit {
    pg_ml,
    ng_ml,
    mg_ml,
    mg_dl,
    g_l,

    ml,
    ul,
    dl,
    l,

    ng,
    g,

    g_day,
    ng_day,

    g_day_cell,
    ng_day_cell,
    ng_day_millioncells,
}

impl SIUnit {
    fn unit_type(&self) -> UnitType {
        use self::SIUnit::*;
        use self::UnitType::*;
        match self {
            pg_ml | ng_ml | mg_ml | mg_dl | g_l
                => Concentration,
            ul | ml | dl | l 
                => Volume,
            g | ng
                => Mass,
            ng_day | g_day
                => Rate,
            ng_day_cell | ng_day_millioncells | g_day_cell
                => CellNormalized,
        }
    }
    /*
    fn si_base(&self) -> Self {
        use self::SIUnit::*;
        use self::UnitType::*;

        match self.unit_type() {
            Concentration => g_l,
            Volume => l,
            Rate => g_day,
            CellNormalized => g_day_cell,
            Mass => g,
        }
    }*/

    fn as_str(&self) -> &'static str {
        use self::SIUnit::*;

        match self {
            pg_ml => "pg/mL",
            ng_ml => "ng/mL",
            mg_ml => "mg/mL",
            mg_dl => "mg/dL",
            g_l => "g/L",

            ml => "mL",
            ul => "µL",
            dl => "dL",
            l => "L",

            g => "g",
            ng => "ng",

            g_day => "g/day",
            ng_day => "ng/day",

            g_day_cell => "g/day/cell",
            ng_day_cell => "ng/day/cell",
            ng_day_millioncells => "g/day/10^6 cells",
        }
    }
    /// Factor to put this unit into base SI unit
    fn si_factor(&self) -> f64 {
        use self::SIUnit::*;

        match self {
            pg_ml => 1.0 / 1_000_000_000.0,
            ng_ml => 1.0 / 1_000_000.0,
            mg_ml => 1.0,
            mg_dl => 1.0 / 100.0,
            g_l => 1.0,

            ml => 1.0 / 1_000.0,
            ul => 1.0 / 1_000_000.0,
            dl => 1.0 / 10.0,
            l => 1.0,

            g => 1.0,
            ng => 1.0 / 1_000_000_000.0,

            g_day => 1.0,
            ng_day => 1.0 / 1_000_000_000.0,
            /* These seem off... */
            g_day_cell => 1.0,
            ng_day_cell => 1.0 / 1_000_000_000.0,
            ng_day_millioncells => 1.0 / (1_000_000_000.0 * 1_000_000_000.0),
        }
    }
}

impl fmt::Display for SIUnit {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for SIUnit {
    type Err = SIError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use self::SIUnit::*;

        match s {
            "pg/mL" | "pg/ml" => Ok(pg_ml),
            "ng/mL" | "ng/ml" => Ok(ng_ml),
            "mg/mL" | "mg/ml" => Ok(mg_ml),
            "mg/dL" | "mg/dl" => Ok(mg_dl),
            "g/L" | "g/l" => Ok(g_l),

            "mL" | "ml" => Ok(ml),
            "µL" | "µl" | "ul" | "uL" => Ok(ul),
            "dL" | "dl" => Ok(dl),
            "L" | "l" => Ok(l),

            "g" => Ok(g),
            "ng" => Ok(ng),

            "g/day" => Ok(g_day),
            "ng/day" => Ok(ng_day),

            "g/day/cell" => Ok(g_day_cell),
            "ng/day/cell" => Ok(ng_day_cell),
            "g/day/10^6 cells" | "g/day/10^6cells" => Ok(ng_day_millioncells),
            
            _ => Err(SIError::UnkType(s.to_string())),
        }
    }
}

impl Serialize for SIUnit {
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
    where S: Serializer
    {
        s.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for SIUnit {
    fn deserialize<D>(d: D) -> Result<Self, D::Error>
    where D: Deserializer<'de>
    {
        d.deserialize_str(SIUnitVisitor)
    }
}
struct SIUnitVisitor;

impl<'de> Visitor<'de> for SIUnitVisitor {
    type Value = SIUnit;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "A typical SI concentration unit")
    }

    fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
    where E: de::Error
    {
        match Self::Value::from_str(s) {
            Ok(v) => Ok(v),
            Err(e) => Err(E::custom(format!("{}",e))),
        }
    }
}


#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum UnitType {
    Concentration,
    Volume,
    CellNormalized,
    Rate,
    Mass,
}

pub fn convert((val, unit): (f64, SIUnit), to: SIUnit) -> Result<f64, SIError> {
    let from_type = unit.unit_type();
    let to_type = to.unit_type();
    if from_type != to_type {
        return Err(SIError::IncompatibleTypes(from_type, to_type));
    }
    let from_fact = unit.si_factor();
    let to_fact = to.si_factor().recip();
    
    Ok(val * (from_fact * to_fact))
}
