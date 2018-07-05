use std::fmt;
use std::str::FromStr;
use serde::de::{self, Visitor, Deserialize, Deserializer};
use serde::ser::{Serialize, Serializer};
use failure::Error;
use traits::SI;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[allow(non_camel_case_types)]
pub enum VolUnit {
    ml,
    ul,
    dl,
    l,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[allow(non_camel_case_types)]
pub enum ConcUnit {
    ng_ml,
    mg_dl,
    g_l,
    //Had to shoe-horn ng/day/10^6 cells into here; not a normal concentration, though.
    ng_day_millioncells,
}

/****** Volume Unit Implementations ******/

impl SI for VolUnit {
    type Unit = Self;

    fn si_base() -> Self::Unit {VolUnit::l}

    fn si_factor(&self) -> f64 {
        use self::VolUnit::*;
        // SI base unit are liters
        match self {
            ml => 1.0/1_000.0,
            ul => 1.0/1_000_000.0,
            dl => 1.0/10.0,
            l  => 1.0,
        }
    }
}

impl FromStr for VolUnit {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use self::VolUnit::*;
        match s {
            "uL" | "ul" | "µL" | "µl" => Ok(ul),
            "ml" | "mL" => Ok(ml),
            "dl" | "dL" => Ok(dl),
            "l" | "L" => Ok(l),
            _ => bail!("Unknown volume unit string <{}>", s)
        }
    }
}

impl fmt::Display for VolUnit {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::VolUnit::*;
        let display = match self {
            ml => "mL",
            ul => "µL",
            dl => "dL",
            l  => "L",
        };
        write!(f, "{}", display)
    }
}

impl Serialize for VolUnit {
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
    where S: Serializer
    {
        s.serialize_str(&format!("{}", self))
    }
}

impl<'de> Deserialize<'de> for VolUnit 
{
    fn deserialize<D>(d: D) -> Result<Self, D::Error>
    where D: Deserializer<'de>
    {
        d.deserialize_str(VolUnitVisitor)
    }

}

struct VolUnitVisitor;

impl<'de> Visitor<'de> for VolUnitVisitor {
    type Value = VolUnit;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "A typical SI volume unit")
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


/****** Concentration Unit Implementations ******/
impl SI for ConcUnit {
    type Unit = Self;

    fn si_factor(&self) -> f64 {
        use self::ConcUnit::*;
        // SI base unit are grams per liter (equivalent to mg/mL)
        match self {
            ng_ml => 1.0/1_000_000.0,
            g_l => 1.0,
            mg_dl => 1.0 / 100.0,
            ng_day_millioncells => 1.0/1_000_000_000.0,
        }
    }

    fn si_base() -> Self::Unit { ConcUnit::g_l }
}

impl FromStr for ConcUnit {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use self::ConcUnit::*;
        match s {
            "ng/mL" | "ng/ml" => Ok(ng_ml),
            "g/l" | "g/L" => Ok(g_l),
            "mg/dl" | "mg/dL" => Ok(mg_dl),
            "ng/day/10^6 cells" | "ng/day/10^6cells" => Ok(ng_day_millioncells),
            _ => bail!("Unknown concentration unit string <{}>", s)
        }
    }
}

impl fmt::Display for ConcUnit {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::ConcUnit::*;
        let display = match self {
            ng_ml => "ng/mL",
            g_l => "g/L",
            mg_dl => "mg/dL",
            ng_day_millioncells => "ng/day/10^6 cells",
        };
        write!(f, "{}", display)
    }
}

impl<'de> Deserialize<'de> for ConcUnit 
{
    fn deserialize<D>(d: D) -> Result<Self, D::Error>
    where D: Deserializer<'de>
    {
        d.deserialize_str(ConcUnitVisitor)
    }

}

impl Serialize for ConcUnit {
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
    where S: Serializer
    {
        s.serialize_str(&format!("{}", self))
    }
}

struct ConcUnitVisitor;

impl<'de> Visitor<'de> for ConcUnitVisitor {
    type Value = ConcUnit;

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

#[inline]
pub fn to_si<U>(value: f64, unit: U) -> f64
where U: SI
{
    value * unit.si_factor()
}
/*
#[inline]
pub fn from_si<U>(value: f64, to_unit: U) -> f64
where U: SI
{
    value / to_unit.si_factor()
}
*/

//TODO: add conversion tests

