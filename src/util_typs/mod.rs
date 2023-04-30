pub mod refinement;
pub mod percent;

#[derive(Debug, Clone)]
pub enum RefinementError {
  RangedI64Overflow(i64, i64, i64),
  RangedI64Underflow(i64, i64, i64),
  RangedI64ArithmeticError(i64, char, i64),
  FailedConversionToI64(String),
}

#[derive(Debug)]
pub enum PercentError {
  PercentF32Overflow(f32),
}

pub type RefineResult<T> = core::result::Result<T, RefinementError>;
