pub mod delta_xor;
pub mod adaptive_linear;
pub mod selector;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum PredictorMode {
    Raw = 0,
    Delta = 1,
    Xor = 2,
    AdaptiveLinear = 3,
    Constant = 4,
    DeltaOfDelta = 5,
    LinearRamp = 6,
    RepeatHistory = 7,
    DecimalFloat = 8,
}

impl PredictorMode {
    pub fn from_u8(val: u8) -> Option<Self> {
        match val {
            0 => Some(Self::Raw),
            1 => Some(Self::Delta),
            2 => Some(Self::Xor),
            3 => Some(Self::AdaptiveLinear),
            4 => Some(Self::Constant),
            5 => Some(Self::DeltaOfDelta),
            6 => Some(Self::LinearRamp),
            7 => Some(Self::RepeatHistory),
            8 => Some(Self::DecimalFloat),
            _ => None,
        }
    }
}
