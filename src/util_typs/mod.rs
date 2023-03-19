pub mod refinement;

#[derive(Debug)]
pub enum RefinementError {
  RangedI64Overflow(i64, i64, i64),
  RangedI64Underflow(i64, i64, i64),
  RangedI64ArithmeticError(i64, char, i64),
}

pub type Result<T> = core::result::Result<T, RefinementError>;
