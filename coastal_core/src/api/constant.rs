use std::io::Write;

use convert_case::{Case, Casing};
use serde::{Deserialize, Serialize};

use super::State;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Constant {
    pub name: String,
    pub value: ConstantValue,
}

impl Constant {
    pub fn c_header(&self, f: &mut impl Write) -> Result<(), std::io::Error> {
        let name = format!("PREFIX_{}", self.name.to_case(Case::ScreamingSnake));
        match &self.value {
            ConstantValue::CChar(val) => {
                writeln!(f, "#define {name} {:?}", char::from(*val))
            }
            ConstantValue::I8(val) => writeln!(f, "#define {name} ((int8_t){val})"),
            ConstantValue::I16(val) => writeln!(f, "#define {name} ((int16_t){val})"),
            ConstantValue::I32(val) => writeln!(f, "#define {name} {val}"),
            ConstantValue::I64(val) => writeln!(f, "#define {name} {val}ll"),
            ConstantValue::U8(val) => writeln!(f, "#define {name} ((uint8_t){val}u)"),
            ConstantValue::U16(val) => writeln!(f, "#define {name} ((uint16_t){val}u)"),
            ConstantValue::U32(val) => writeln!(f, "#define {name} {val}u"),
            ConstantValue::U64(val) => writeln!(f, "#define {name} {val}ull"),
            ConstantValue::F32(val) => writeln!(f, "#define {name} {val}f"),
            ConstantValue::F64(val) => writeln!(f, "#define {name} {val}"),
            ConstantValue::Str(val) => writeln!(f, "#define {name} {val:?}"),
            ConstantValue::Bytes(val) => {
                let string_constant =
                    String::from_utf8(val.iter().copied().flat_map(u8::escape_ascii).collect())
                        .expect("valid utf8");
                writeln!(f, "#define {name} \"{string_constant}\"")
            }
        }
    }
}

impl State for Constant {
    const TYPE_NAME: &'static str = "coastal.constant";
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConstantValue {
    CChar(u8),
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    U8(i8),
    U16(i16),
    U32(i32),
    U64(i64),
    F32(f32),
    F64(f64),
    Str(String),
    Bytes(Vec<u8>),
}
